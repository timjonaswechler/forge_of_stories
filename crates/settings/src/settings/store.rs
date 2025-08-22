/// A set of strongly-typed setting values defined via multiple config files.
pub struct SettingsStore {
    setting_values: HashMap<TypeId, Box<dyn AnySettingValue>>,
    raw_default_settings: Value,
    raw_global_settings: Option<Value>,
    raw_user_settings: Value,
    raw_server_settings: Option<Value>,
    raw_extension_settings: Value,
    raw_local_settings: BTreeMap<(WorktreeId, Arc<Path>), Value>,
    raw_editorconfig_settings: BTreeMap<(WorktreeId, Arc<Path>), (String, Option<Editorconfig>)>,
    tab_size_callback: Option<(
        TypeId,
        Box<dyn Fn(&dyn Any) -> Option<usize> + Send + Sync + 'static>,
    )>,
    _setting_file_updates: Task<()>,
    setting_file_updates_tx:
        mpsc::UnboundedSender<Box<dyn FnOnce(AsyncApp) -> LocalBoxFuture<'static, Result<()>>>>,
}

impl Global for SettingsStore {}

impl Debug for SettingsStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SettingsStore")
            .field(
                "types",
                &self
                    .setting_values
                    .values()
                    .map(|value| value.setting_type_name())
                    .collect::<Vec<_>>(),
            )
            .field("default_settings", &self.raw_default_settings)
            .field("user_settings", &self.raw_user_settings)
            .field("local_settings", &self.raw_local_settings)
            .finish_non_exhaustive()
    }
}

impl SettingsStore {
    pub fn new(cx: &App) -> Self {
        let (setting_file_updates_tx, mut setting_file_updates_rx) = mpsc::unbounded();
        Self {
            setting_values: Default::default(),
            raw_default_settings: json!({}),
            raw_global_settings: None,
            raw_user_settings: json!({}),
            raw_server_settings: None,
            raw_extension_settings: json!({}),
            raw_local_settings: Default::default(),
            raw_editorconfig_settings: BTreeMap::default(),
            tab_size_callback: Default::default(),
            setting_file_updates_tx,
            _setting_file_updates: cx.spawn(async move |cx| {
                while let Some(setting_file_update) = setting_file_updates_rx.next().await {
                    (setting_file_update)(cx.clone()).await.log_err();
                }
            }),
        }
    }

    pub fn observe_active_settings_profile_name(cx: &mut App) -> gpui::Subscription {
        cx.observe_global::<ActiveSettingsProfileName>(|cx| {
            Self::update_global(cx, |store, cx| {
                store.recompute_values(None, cx).log_err();
            });
        })
    }

    pub fn update<C, R>(cx: &mut C, f: impl FnOnce(&mut Self, &mut C) -> R) -> R
    where
        C: BorrowAppContext,
    {
        cx.update_global(f)
    }

    /// Add a new type of setting to the store.
    pub fn register_setting<T: Settings>(&mut self, cx: &mut App) {
        let setting_type_id = TypeId::of::<T>();
        let entry = self.setting_values.entry(setting_type_id);

        if matches!(entry, hash_map::Entry::Occupied(_)) {
            return;
        }

        let setting_value = entry.or_insert(Box::new(SettingValue::<T> {
            global_value: None,
            local_values: Vec::new(),
        }));

        if let Some(default_settings) = setting_value
            .deserialize_setting(&self.raw_default_settings)
            .log_err()
        {
            let user_value = setting_value
                .deserialize_setting(&self.raw_user_settings)
                .log_err();

            let mut release_channel_value = None;
            if let Some(release_settings) = &self
                .raw_user_settings
                .get(release_channel::RELEASE_CHANNEL.dev_name())
            {
                release_channel_value = setting_value
                    .deserialize_setting(release_settings)
                    .log_err();
            }

            let mut os_settings_value = None;
            if let Some(os_settings) = &self.raw_user_settings.get(env::consts::OS) {
                os_settings_value = setting_value.deserialize_setting(os_settings).log_err();
            }

            let mut profile_value = None;
            if let Some(active_profile) = cx.try_global::<ActiveSettingsProfileName>() {
                if let Some(profiles) = self.raw_user_settings.get("profiles") {
                    if let Some(profile_settings) = profiles.get(&active_profile.0) {
                        profile_value = setting_value
                            .deserialize_setting(profile_settings)
                            .log_err();
                    }
                }
            }

            let server_value = self
                .raw_server_settings
                .as_ref()
                .and_then(|server_setting| {
                    setting_value.deserialize_setting(server_setting).log_err()
                });

            let extension_value = setting_value
                .deserialize_setting(&self.raw_extension_settings)
                .log_err();

            if let Some(setting) = setting_value
                .load_setting(
                    SettingsSources {
                        default: &default_settings,
                        global: None,
                        extensions: extension_value.as_ref(),
                        user: user_value.as_ref(),
                        release_channel: release_channel_value.as_ref(),
                        operating_system: os_settings_value.as_ref(),
                        profile: profile_value.as_ref(),
                        server: server_value.as_ref(),
                        project: &[],
                    },
                    cx,
                )
                .context("A default setting must be added to the `default.json` file")
                .log_err()
            {
                setting_value.set_global_value(setting);
            }
        }
    }

    /// Get the value of a setting.
    ///
    /// Panics if the given setting type has not been registered, or if there is no
    /// value for this setting.
    pub fn get<T: Settings>(&self, path: Option<SettingsLocation>) -> &T {
        self.setting_values
            .get(&TypeId::of::<T>())
            .unwrap_or_else(|| panic!("unregistered setting type {}", type_name::<T>()))
            .value_for_path(path)
            .downcast_ref::<T>()
            .expect("no default value for setting type")
    }

    /// Get all values from project specific settings
    pub fn get_all_locals<T: Settings>(&self) -> Vec<(WorktreeId, Arc<Path>, &T)> {
        self.setting_values
            .get(&TypeId::of::<T>())
            .unwrap_or_else(|| panic!("unregistered setting type {}", type_name::<T>()))
            .all_local_values()
            .into_iter()
            .map(|(id, path, any)| {
                (
                    id,
                    path,
                    any.downcast_ref::<T>()
                        .expect("wrong value type for setting"),
                )
            })
            .collect()
    }

    /// Override the global value for a setting.
    ///
    /// The given value will be overwritten if the user settings file changes.
    pub fn override_global<T: Settings>(&mut self, value: T) {
        self.setting_values
            .get_mut(&TypeId::of::<T>())
            .unwrap_or_else(|| panic!("unregistered setting type {}", type_name::<T>()))
            .set_global_value(Box::new(value))
    }

    /// Get the user's settings as a raw JSON value.
    ///
    /// For user-facing functionality use the typed setting interface.
    /// (e.g. ProjectSettings::get_global(cx))
    pub fn raw_user_settings(&self) -> &Value {
        &self.raw_user_settings
    }

    /// Get the configured settings profile names.
    pub fn configured_settings_profiles(&self) -> impl Iterator<Item = &str> {
        self.raw_user_settings
            .get("profiles")
            .and_then(|v| v.as_object())
            .into_iter()
            .flat_map(|obj| obj.keys())
            .map(|s| s.as_str())
    }

    /// Access the raw JSON value of the global settings.
    pub fn raw_global_settings(&self) -> Option<&Value> {
        self.raw_global_settings.as_ref()
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn test(cx: &mut App) -> Self {
        let mut this = Self::new(cx);
        this.set_default_settings(&crate::test_settings(), cx)
            .unwrap();
        this.set_user_settings("{}", cx).unwrap();
        this
    }

    /// Updates the value of a setting in the user's global configuration.
    ///
    /// This is only for tests. Normally, settings are only loaded from
    /// JSON files.
    #[cfg(any(test, feature = "test-support"))]
    pub fn update_user_settings<T: Settings>(
        &mut self,
        cx: &mut App,
        update: impl FnOnce(&mut T::FileContent),
    ) {
        let old_text = serde_json::to_string(&self.raw_user_settings).unwrap();
        let new_text = self.new_text_for_update::<T>(old_text, update);
        self.set_user_settings(&new_text, cx).unwrap();
    }

    pub async fn load_settings(fs: &Arc<dyn Fs>) -> Result<String> {
        match fs.load(paths::settings_file()).await {
            result @ Ok(_) => result,
            Err(err) => {
                if let Some(e) = err.downcast_ref::<std::io::Error>() {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        return Ok(crate::initial_user_settings_content().to_string());
                    }
                }
                Err(err)
            }
        }
    }

    pub async fn load_global_settings(fs: &Arc<dyn Fs>) -> Result<String> {
        match fs.load(paths::global_settings_file()).await {
            result @ Ok(_) => result,
            Err(err) => {
                if let Some(e) = err.downcast_ref::<std::io::Error>() {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        return Ok("{}".to_string());
                    }
                }
                Err(err)
            }
        }
    }

    pub fn update_settings_file<T: Settings>(
        &self,
        fs: Arc<dyn Fs>,
        update: impl 'static + Send + FnOnce(&mut T::FileContent, &App),
    ) {
        self.setting_file_updates_tx
            .unbounded_send(Box::new(move |cx: AsyncApp| {
                async move {
                    let old_text = Self::load_settings(&fs).await?;
                    let new_text = cx.read_global(|store: &SettingsStore, cx| {
                        store.new_text_for_update::<T>(old_text, |content| update(content, cx))
                    })?;
                    let settings_path = paths::settings_file().as_path();
                    if fs.is_file(settings_path).await {
                        let resolved_path =
                            fs.canonicalize(settings_path).await.with_context(|| {
                                format!("Failed to canonicalize settings path {:?}", settings_path)
                            })?;

                        fs.atomic_write(resolved_path.clone(), new_text)
                            .await
                            .with_context(|| {
                                format!("Failed to write settings to file {:?}", resolved_path)
                            })?;
                    } else {
                        fs.atomic_write(settings_path.to_path_buf(), new_text)
                            .await
                            .with_context(|| {
                                format!("Failed to write settings to file {:?}", settings_path)
                            })?;
                    }

                    anyhow::Ok(())
                }
                .boxed_local()
            }))
            .ok();
    }

    pub fn import_vscode_settings(
        &self,
        fs: Arc<dyn Fs>,
        vscode_settings: VsCodeSettings,
    ) -> oneshot::Receiver<Result<()>> {
        let (tx, rx) = oneshot::channel::<Result<()>>();
        self.setting_file_updates_tx
            .unbounded_send(Box::new(move |cx: AsyncApp| {
                async move {
                    let res = async move {
                        let old_text = Self::load_settings(&fs).await?;
                        let new_text = cx.read_global(|store: &SettingsStore, _cx| {
                            store.get_vscode_edits(old_text, &vscode_settings)
                        })?;
                        let settings_path = paths::settings_file().as_path();
                        if fs.is_file(settings_path).await {
                            let resolved_path =
                                fs.canonicalize(settings_path).await.with_context(|| {
                                    format!(
                                        "Failed to canonicalize settings path {:?}",
                                        settings_path
                                    )
                                })?;

                            fs.atomic_write(resolved_path.clone(), new_text)
                                .await
                                .with_context(|| {
                                    format!("Failed to write settings to file {:?}", resolved_path)
                                })?;
                        } else {
                            fs.atomic_write(settings_path.to_path_buf(), new_text)
                                .await
                                .with_context(|| {
                                    format!("Failed to write settings to file {:?}", settings_path)
                                })?;
                        }

                        anyhow::Ok(())
                    }
                    .await;

                    let new_res = match &res {
                        Ok(_) => anyhow::Ok(()),
                        Err(e) => Err(anyhow::anyhow!("Failed to write settings to file {:?}", e)),
                    };

                    _ = tx.send(new_res);
                    res
                }
                .boxed_local()
            }))
            .ok();

        rx
    }
}

impl SettingsStore {
    /// Updates the value of a setting in a JSON file, returning the new text
    /// for that JSON file.
    pub fn new_text_for_update<T: Settings>(
        &self,
        old_text: String,
        update: impl FnOnce(&mut T::FileContent),
    ) -> String {
        let edits = self.edits_for_update::<T>(&old_text, update);
        let mut new_text = old_text;
        for (range, replacement) in edits.into_iter() {
            new_text.replace_range(range, &replacement);
        }
        new_text
    }

    pub fn get_vscode_edits(&self, mut old_text: String, vscode: &VsCodeSettings) -> String {
        let mut new_text = old_text.clone();
        let mut edits: Vec<(Range<usize>, String)> = Vec::new();
        let raw_settings = parse_json_with_comments::<Value>(&old_text).unwrap_or_default();
        let tab_size = self.json_tab_size();
        for v in self.setting_values.values() {
            v.edits_for_update(&raw_settings, tab_size, vscode, &mut old_text, &mut edits);
        }
        for (range, replacement) in edits.into_iter() {
            new_text.replace_range(range, &replacement);
        }
        new_text
    }

    /// Updates the value of a setting in a JSON file, returning a list
    /// of edits to apply to the JSON file.
    pub fn edits_for_update<T: Settings>(
        &self,
        text: &str,
        update: impl FnOnce(&mut T::FileContent),
    ) -> Vec<(Range<usize>, String)> {
        let setting_type_id = TypeId::of::<T>();

        let preserved_keys = T::PRESERVED_KEYS.unwrap_or_default();

        let setting = self
            .setting_values
            .get(&setting_type_id)
            .unwrap_or_else(|| panic!("unregistered setting type {}", type_name::<T>()));
        let raw_settings = parse_json_with_comments::<Value>(text).unwrap_or_default();
        let (key, deserialized_setting) = setting.deserialize_setting_with_key(&raw_settings);
        let old_content = match deserialized_setting {
            Ok(content) => content.0.downcast::<T::FileContent>().unwrap(),
            Err(_) => Box::<<T as Settings>::FileContent>::default(),
        };
        let mut new_content = old_content.clone();
        update(&mut new_content);

        let old_value = serde_json::to_value(&old_content).unwrap();
        let new_value = serde_json::to_value(new_content).unwrap();

        let mut key_path = Vec::new();
        if let Some(key) = key {
            key_path.push(key);
        }

        let mut edits = Vec::new();
        let tab_size = self.json_tab_size();
        let mut text = text.to_string();
        update_value_in_json_text(
            &mut text,
            &mut key_path,
            tab_size,
            &old_value,
            &new_value,
            preserved_keys,
            &mut edits,
        );
        edits
    }

    /// Configure the tab sized when updating JSON files.
    pub fn set_json_tab_size_callback<T: Settings>(
        &mut self,
        get_tab_size: fn(&T) -> Option<usize>,
    ) {
        self.tab_size_callback = Some((
            TypeId::of::<T>(),
            Box::new(move |value| get_tab_size(value.downcast_ref::<T>().unwrap())),
        ));
    }

    pub fn json_tab_size(&self) -> usize {
        const DEFAULT_JSON_TAB_SIZE: usize = 2;

        if let Some((setting_type_id, callback)) = &self.tab_size_callback {
            let setting_value = self.setting_values.get(setting_type_id).unwrap();
            let value = setting_value.value_for_path(None);
            if let Some(value) = callback(value) {
                return value;
            }
        }

        DEFAULT_JSON_TAB_SIZE
    }

    /// Sets the default settings via a JSON string.
    ///
    /// The string should contain a JSON object with a default value for every setting.
    pub fn set_default_settings(
        &mut self,
        default_settings_content: &str,
        cx: &mut App,
    ) -> Result<()> {
        let settings: Value = parse_json_with_comments(default_settings_content)?;
        anyhow::ensure!(settings.is_object(), "settings must be an object");
        self.raw_default_settings = settings;
        self.recompute_values(None, cx)?;
        Ok(())
    }

    /// Sets the user settings via a JSON string.
    pub fn set_user_settings(
        &mut self,
        user_settings_content: &str,
        cx: &mut App,
    ) -> Result<Value> {
        let settings: Value = if user_settings_content.is_empty() {
            parse_json_with_comments("{}")?
        } else {
            parse_json_with_comments(user_settings_content)?
        };

        anyhow::ensure!(settings.is_object(), "settings must be an object");
        self.raw_user_settings = settings.clone();
        self.recompute_values(None, cx)?;
        Ok(settings)
    }

    /// Sets the global settings via a JSON string.
    pub fn set_global_settings(
        &mut self,
        global_settings_content: &str,
        cx: &mut App,
    ) -> Result<Value> {
        let settings: Value = if global_settings_content.is_empty() {
            parse_json_with_comments("{}")?
        } else {
            parse_json_with_comments(global_settings_content)?
        };

        anyhow::ensure!(settings.is_object(), "settings must be an object");
        self.raw_global_settings = Some(settings.clone());
        self.recompute_values(None, cx)?;
        Ok(settings)
    }

    pub fn set_server_settings(
        &mut self,
        server_settings_content: &str,
        cx: &mut App,
    ) -> Result<()> {
        let settings: Option<Value> = if server_settings_content.is_empty() {
            None
        } else {
            parse_json_with_comments(server_settings_content)?
        };

        anyhow::ensure!(
            settings
                .as_ref()
                .map(|value| value.is_object())
                .unwrap_or(true),
            "settings must be an object"
        );
        self.raw_server_settings = settings;
        self.recompute_values(None, cx)?;
        Ok(())
    }

    /// Add or remove a set of local settings via a JSON string.
    pub fn set_local_settings(
        &mut self,
        root_id: WorktreeId,
        directory_path: Arc<Path>,
        kind: LocalSettingsKind,
        settings_content: Option<&str>,
        cx: &mut App,
    ) -> std::result::Result<(), InvalidSettingsError> {
        let mut zed_settings_changed = false;
        match (
            kind,
            settings_content
                .map(|content| content.trim())
                .filter(|content| !content.is_empty()),
        ) {
            (LocalSettingsKind::Tasks, _) => {
                return Err(InvalidSettingsError::Tasks {
                    message: "Attempted to submit tasks into the settings store".to_string(),
                    path: directory_path.join(task_file_name()),
                });
            }
            (LocalSettingsKind::Debug, _) => {
                return Err(InvalidSettingsError::Debug {
                    message: "Attempted to submit debugger config into the settings store"
                        .to_string(),
                    path: directory_path.join(task_file_name()),
                });
            }
            (LocalSettingsKind::Settings, None) => {
                zed_settings_changed = self
                    .raw_local_settings
                    .remove(&(root_id, directory_path.clone()))
                    .is_some()
            }
            (LocalSettingsKind::Editorconfig, None) => {
                self.raw_editorconfig_settings
                    .remove(&(root_id, directory_path.clone()));
            }
            (LocalSettingsKind::Settings, Some(settings_contents)) => {
                let new_settings =
                    parse_json_with_comments::<Value>(settings_contents).map_err(|e| {
                        InvalidSettingsError::LocalSettings {
                            path: directory_path.join(local_settings_file_relative_path()),
                            message: e.to_string(),
                        }
                    })?;
                match self
                    .raw_local_settings
                    .entry((root_id, directory_path.clone()))
                {
                    btree_map::Entry::Vacant(v) => {
                        v.insert(new_settings);
                        zed_settings_changed = true;
                    }
                    btree_map::Entry::Occupied(mut o) => {
                        if o.get() != &new_settings {
                            o.insert(new_settings);
                            zed_settings_changed = true;
                        }
                    }
                }
            }
            (LocalSettingsKind::Editorconfig, Some(editorconfig_contents)) => {
                match self
                    .raw_editorconfig_settings
                    .entry((root_id, directory_path.clone()))
                {
                    btree_map::Entry::Vacant(v) => match editorconfig_contents.parse() {
                        Ok(new_contents) => {
                            v.insert((editorconfig_contents.to_owned(), Some(new_contents)));
                        }
                        Err(e) => {
                            v.insert((editorconfig_contents.to_owned(), None));
                            return Err(InvalidSettingsError::Editorconfig {
                                message: e.to_string(),
                                path: directory_path.join(EDITORCONFIG_NAME),
                            });
                        }
                    },
                    btree_map::Entry::Occupied(mut o) => {
                        if o.get().0 != editorconfig_contents {
                            match editorconfig_contents.parse() {
                                Ok(new_contents) => {
                                    o.insert((
                                        editorconfig_contents.to_owned(),
                                        Some(new_contents),
                                    ));
                                }
                                Err(e) => {
                                    o.insert((editorconfig_contents.to_owned(), None));
                                    return Err(InvalidSettingsError::Editorconfig {
                                        message: e.to_string(),
                                        path: directory_path.join(EDITORCONFIG_NAME),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        };

        if zed_settings_changed {
            self.recompute_values(Some((root_id, &directory_path)), cx)?;
        }
        Ok(())
    }

    pub fn set_extension_settings<T: Serialize>(&mut self, content: T, cx: &mut App) -> Result<()> {
        let settings: Value = serde_json::to_value(content)?;
        anyhow::ensure!(settings.is_object(), "settings must be an object");
        self.raw_extension_settings = settings;
        self.recompute_values(None, cx)?;
        Ok(())
    }

    /// Add or remove a set of local settings via a JSON string.
    pub fn clear_local_settings(&mut self, root_id: WorktreeId, cx: &mut App) -> Result<()> {
        self.raw_local_settings
            .retain(|(worktree_id, _), _| worktree_id != &root_id);
        self.recompute_values(Some((root_id, "".as_ref())), cx)?;
        Ok(())
    }

    pub fn local_settings(
        &self,
        root_id: WorktreeId,
    ) -> impl '_ + Iterator<Item = (Arc<Path>, String)> {
        self.raw_local_settings
            .range(
                (root_id, Path::new("").into())
                    ..(
                        WorktreeId::from_usize(root_id.to_usize() + 1),
                        Path::new("").into(),
                    ),
            )
            .map(|((_, path), content)| (path.clone(), serde_json::to_string(content).unwrap()))
    }

    pub fn local_editorconfig_settings(
        &self,
        root_id: WorktreeId,
    ) -> impl '_ + Iterator<Item = (Arc<Path>, String, Option<Editorconfig>)> {
        self.raw_editorconfig_settings
            .range(
                (root_id, Path::new("").into())
                    ..(
                        WorktreeId::from_usize(root_id.to_usize() + 1),
                        Path::new("").into(),
                    ),
            )
            .map(|((_, path), (content, parsed_content))| {
                (path.clone(), content.clone(), parsed_content.clone())
            })
    }

    pub fn json_schema(&self, schema_params: &SettingsJsonSchemaParams, cx: &App) -> Value {
        let mut generator = schemars::generate::SchemaSettings::draft2019_09()
            .with_transform(DefaultDenyUnknownFields)
            .into_generator();
        let mut combined_schema = json!({
            "type": "object",
            "properties": {}
        });

        // Merge together settings schemas, similarly to json schema's "allOf". This merging is
        // recursive, though at time of writing this recursive nature isn't used very much. An
        // example of it is the schema for `jupyter` having contribution from both `EditorSettings`
        // and `JupyterSettings`.
        //
        // This logic could be removed in favor of "allOf", but then there isn't the opportunity to
        // validate and fully control the merge.
        for setting_value in self.setting_values.values() {
            let mut setting_schema = setting_value.json_schema(&mut generator);

            if let Some(key) = setting_value.key() {
                if let Some(properties) = combined_schema.get_mut("properties") {
                    if let Some(properties_obj) = properties.as_object_mut() {
                        if let Some(target) = properties_obj.get_mut(key) {
                            merge_schema(target, setting_schema.to_value());
                        } else {
                            properties_obj.insert(key.to_string(), setting_schema.to_value());
                        }
                    }
                }
            } else {
                setting_schema.remove("description");
                setting_schema.remove("additionalProperties");
                merge_schema(&mut combined_schema, setting_schema.to_value());
            }
        }

        fn merge_schema(target: &mut serde_json::Value, source: serde_json::Value) {
            let (Some(target_obj), serde_json::Value::Object(source_obj)) =
                (target.as_object_mut(), source)
            else {
                return;
            };

            for (source_key, source_value) in source_obj {
                match source_key.as_str() {
                    "properties" => {
                        let serde_json::Value::Object(source_properties) = source_value else {
                            log::error!(
                                "bug: expected object for `{}` json schema field, but got: {}",
                                source_key,
                                source_value
                            );
                            continue;
                        };
                        let target_properties =
                            target_obj.entry(source_key.clone()).or_insert(json!({}));
                        let Some(target_properties) = target_properties.as_object_mut() else {
                            log::error!(
                                "bug: expected object for `{}` json schema field, but got: {}",
                                source_key,
                                target_properties
                            );
                            continue;
                        };
                        for (key, value) in source_properties {
                            if let Some(existing) = target_properties.get_mut(&key) {
                                merge_schema(existing, value);
                            } else {
                                target_properties.insert(key, value);
                            }
                        }
                    }
                    "allOf" | "anyOf" | "oneOf" => {
                        let serde_json::Value::Array(source_array) = source_value else {
                            log::error!(
                                "bug: expected array for `{}` json schema field, but got: {}",
                                source_key,
                                source_value,
                            );
                            continue;
                        };
                        let target_array =
                            target_obj.entry(source_key.clone()).or_insert(json!([]));
                        let Some(target_array) = target_array.as_array_mut() else {
                            log::error!(
                                "bug: expected array for `{}` json schema field, but got: {}",
                                source_key,
                                target_array,
                            );
                            continue;
                        };
                        target_array.extend(source_array);
                    }
                    "type"
                    | "$ref"
                    | "enum"
                    | "minimum"
                    | "maximum"
                    | "pattern"
                    | "description"
                    | "additionalProperties" => {
                        if let Some(old_value) =
                            target_obj.insert(source_key.clone(), source_value.clone())
                        {
                            if old_value != source_value {
                                log::error!(
                                    "bug: while merging JSON schemas, \
                                    mismatch `\"{}\": {}` (before was `{}`)",
                                    source_key,
                                    old_value,
                                    source_value
                                );
                            }
                        }
                    }
                    _ => {
                        log::error!(
                            "bug: while merging settings JSON schemas, \
                            encountered unexpected `\"{}\": {}`",
                            source_key,
                            source_value
                        );
                    }
                }
            }
        }

        // add schemas which are determined at runtime
        for parameterized_json_schema in inventory::iter::<ParameterizedJsonSchema>() {
            (parameterized_json_schema.add_and_get_ref)(&mut generator, schema_params, cx);
        }

        // add merged settings schema to the definitions
        const ZED_SETTINGS: &str = "ZedSettings";
        let zed_settings_ref = add_new_subschema(&mut generator, ZED_SETTINGS, combined_schema);

        // add `ZedSettingsOverride` which is the same as `ZedSettings` except that unknown
        // fields are rejected. This is used for release stage settings and profiles.
        let mut zed_settings_override = zed_settings_ref.clone();
        zed_settings_override.insert("unevaluatedProperties".to_string(), false.into());
        let zed_settings_override_ref = add_new_subschema(
            &mut generator,
            "ZedSettingsOverride",
            zed_settings_override.to_value(),
        );

        // Remove `"additionalProperties": false` added by `DefaultDenyUnknownFields` so that
        // unknown fields can be handled by the root schema and `ZedSettingsOverride`.
        let mut definitions = generator.take_definitions(true);
        definitions
            .get_mut(ZED_SETTINGS)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove("additionalProperties");

        let meta_schema = generator
            .settings()
            .meta_schema
            .as_ref()
            .expect("meta_schema should be present in schemars settings")
            .to_string();

        json!({
            "$schema": meta_schema,
            "title": "Zed Settings",
            "unevaluatedProperties": false,
            // ZedSettings + settings overrides for each release stage / OS / profiles
            "allOf": [
                zed_settings_ref,
                {
                    "properties": {
                        "dev": zed_settings_override_ref,
                        "nightly": zed_settings_override_ref,
                        "stable": zed_settings_override_ref,
                        "preview": zed_settings_override_ref,
                        "linux": zed_settings_override_ref,
                        "macos": zed_settings_override_ref,
                        "windows": zed_settings_override_ref,
                        "profiles": {
                            "type": "object",
                            "description": "Configures any number of settings profiles.",
                            "additionalProperties": zed_settings_override_ref
                        }
                    }
                }
            ],
            "$defs": definitions,
        })
    }

    fn recompute_values(
        &mut self,
        changed_local_path: Option<(WorktreeId, &Path)>,
        cx: &mut App,
    ) -> std::result::Result<(), InvalidSettingsError> {
        // Reload the global and local values for every setting.
        let mut project_settings_stack = Vec::<DeserializedSetting>::new();
        let mut paths_stack = Vec::<Option<(WorktreeId, &Path)>>::new();
        for setting_value in self.setting_values.values_mut() {
            let default_settings = setting_value
                .deserialize_setting(&self.raw_default_settings)
                .map_err(|e| InvalidSettingsError::DefaultSettings {
                    message: e.to_string(),
                })?;

            let global_settings = self
                .raw_global_settings
                .as_ref()
                .and_then(|setting| setting_value.deserialize_setting(setting).log_err());

            let extension_settings = setting_value
                .deserialize_setting(&self.raw_extension_settings)
                .log_err();

            let user_settings = match setting_value.deserialize_setting(&self.raw_user_settings) {
                Ok(settings) => Some(settings),
                Err(error) => {
                    return Err(InvalidSettingsError::UserSettings {
                        message: error.to_string(),
                    });
                }
            };

            let server_settings = self
                .raw_server_settings
                .as_ref()
                .and_then(|setting| setting_value.deserialize_setting(setting).log_err());

            let mut release_channel_settings = None;
            if let Some(release_settings) = &self
                .raw_user_settings
                .get(release_channel::RELEASE_CHANNEL.dev_name())
            {
                if let Some(release_settings) = setting_value
                    .deserialize_setting(release_settings)
                    .log_err()
                {
                    release_channel_settings = Some(release_settings);
                }
            }

            let mut os_settings = None;
            if let Some(settings) = &self.raw_user_settings.get(env::consts::OS) {
                if let Some(settings) = setting_value.deserialize_setting(settings).log_err() {
                    os_settings = Some(settings);
                }
            }

            let mut profile_settings = None;
            if let Some(active_profile) = cx.try_global::<ActiveSettingsProfileName>() {
                if let Some(profiles) = self.raw_user_settings.get("profiles") {
                    if let Some(profile_json) = profiles.get(&active_profile.0) {
                        profile_settings =
                            setting_value.deserialize_setting(profile_json).log_err();
                    }
                }
            }

            // If the global settings file changed, reload the global value for the field.
            if changed_local_path.is_none() {
                if let Some(value) = setting_value
                    .load_setting(
                        SettingsSources {
                            default: &default_settings,
                            global: global_settings.as_ref(),
                            extensions: extension_settings.as_ref(),
                            user: user_settings.as_ref(),
                            release_channel: release_channel_settings.as_ref(),
                            operating_system: os_settings.as_ref(),
                            profile: profile_settings.as_ref(),
                            server: server_settings.as_ref(),
                            project: &[],
                        },
                        cx,
                    )
                    .log_err()
                {
                    setting_value.set_global_value(value);
                }
            }

            // Reload the local values for the setting.
            paths_stack.clear();
            project_settings_stack.clear();
            for ((root_id, directory_path), local_settings) in &self.raw_local_settings {
                // Build a stack of all of the local values for that setting.
                while let Some(prev_entry) = paths_stack.last() {
                    if let Some((prev_root_id, prev_path)) = prev_entry {
                        if root_id != prev_root_id || !directory_path.starts_with(prev_path) {
                            paths_stack.pop();
                            project_settings_stack.pop();
                            continue;
                        }
                    }
                    break;
                }

                match setting_value.deserialize_setting(local_settings) {
                    Ok(local_settings) => {
                        paths_stack.push(Some((*root_id, directory_path.as_ref())));
                        project_settings_stack.push(local_settings);

                        // If a local settings file changed, then avoid recomputing local
                        // settings for any path outside of that directory.
                        if changed_local_path.map_or(
                            false,
                            |(changed_root_id, changed_local_path)| {
                                *root_id != changed_root_id
                                    || !directory_path.starts_with(changed_local_path)
                            },
                        ) {
                            continue;
                        }

                        if let Some(value) = setting_value
                            .load_setting(
                                SettingsSources {
                                    default: &default_settings,
                                    global: global_settings.as_ref(),
                                    extensions: extension_settings.as_ref(),
                                    user: user_settings.as_ref(),
                                    release_channel: release_channel_settings.as_ref(),
                                    operating_system: os_settings.as_ref(),
                                    profile: profile_settings.as_ref(),
                                    server: server_settings.as_ref(),
                                    project: &project_settings_stack.iter().collect::<Vec<_>>(),
                                },
                                cx,
                            )
                            .log_err()
                        {
                            setting_value.set_local_value(*root_id, directory_path.clone(), value);
                        }
                    }
                    Err(error) => {
                        return Err(InvalidSettingsError::LocalSettings {
                            path: directory_path.join(local_settings_file_relative_path()),
                            message: error.to_string(),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    pub fn editorconfig_properties(
        &self,
        for_worktree: WorktreeId,
        for_path: &Path,
    ) -> Option<EditorconfigProperties> {
        let mut properties = EditorconfigProperties::new();

        for (directory_with_config, _, parsed_editorconfig) in
            self.local_editorconfig_settings(for_worktree)
        {
            if !for_path.starts_with(&directory_with_config) {
                properties.use_fallbacks();
                return Some(properties);
            }
            let parsed_editorconfig = parsed_editorconfig?;
            if parsed_editorconfig.is_root {
                properties = EditorconfigProperties::new();
            }
            for section in parsed_editorconfig.sections {
                section.apply_to(&mut properties, for_path).log_err()?;
            }
        }

        properties.use_fallbacks();
        Some(properties)
    }
}
