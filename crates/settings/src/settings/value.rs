#[derive(Debug)]
struct SettingValue<T> {
    global_value: Option<T>,
    local_values: Vec<(WorktreeId, Arc<Path>, T)>,
}

trait AnySettingValue: 'static + Send + Sync {
    fn key(&self) -> Option<&'static str>;
    fn setting_type_name(&self) -> &'static str;
    fn deserialize_setting(&self, json: &Value) -> Result<DeserializedSetting> {
        self.deserialize_setting_with_key(json).1
    }
    fn deserialize_setting_with_key(
        &self,
        json: &Value,
    ) -> (Option<&'static str>, Result<DeserializedSetting>);
    fn load_setting(
        &self,
        sources: SettingsSources<DeserializedSetting>,
        cx: &mut App,
    ) -> Result<Box<dyn Any>>;
    fn value_for_path(&self, path: Option<SettingsLocation>) -> &dyn Any;
    fn all_local_values(&self) -> Vec<(WorktreeId, Arc<Path>, &dyn Any)>;
    fn set_global_value(&mut self, value: Box<dyn Any>);
    fn set_local_value(&mut self, root_id: WorktreeId, path: Arc<Path>, value: Box<dyn Any>);
    fn json_schema(&self, generator: &mut schemars::SchemaGenerator) -> schemars::Schema;
    fn edits_for_update(
        &self,
        raw_settings: &serde_json::Value,
        tab_size: usize,
        vscode_settings: &VsCodeSettings,
        text: &mut String,
        edits: &mut Vec<(Range<usize>, String)>,
    );
}

impl<T: Settings> AnySettingValue for SettingValue<T> {
    fn key(&self) -> Option<&'static str> {
        T::KEY
    }

    fn setting_type_name(&self) -> &'static str {
        type_name::<T>()
    }

    fn load_setting(
        &self,
        values: SettingsSources<DeserializedSetting>,
        cx: &mut App,
    ) -> Result<Box<dyn Any>> {
        Ok(Box::new(T::load(
            SettingsSources {
                default: values.default.0.downcast_ref::<T::FileContent>().unwrap(),
                global: values
                    .global
                    .map(|value| value.0.downcast_ref::<T::FileContent>().unwrap()),
                extensions: values
                    .extensions
                    .map(|value| value.0.downcast_ref::<T::FileContent>().unwrap()),
                user: values
                    .user
                    .map(|value| value.0.downcast_ref::<T::FileContent>().unwrap()),
                release_channel: values
                    .release_channel
                    .map(|value| value.0.downcast_ref::<T::FileContent>().unwrap()),
                operating_system: values
                    .operating_system
                    .map(|value| value.0.downcast_ref::<T::FileContent>().unwrap()),
                profile: values
                    .profile
                    .map(|value| value.0.downcast_ref::<T::FileContent>().unwrap()),
                server: values
                    .server
                    .map(|value| value.0.downcast_ref::<T::FileContent>().unwrap()),
                project: values
                    .project
                    .iter()
                    .map(|value| value.0.downcast_ref().unwrap())
                    .collect::<SmallVec<[_; 3]>>()
                    .as_slice(),
            },
            cx,
        )?))
    }

    fn deserialize_setting_with_key(
        &self,
        mut json: &Value,
    ) -> (Option<&'static str>, Result<DeserializedSetting>) {
        let mut key = None;
        if let Some(k) = T::KEY {
            if let Some(value) = json.get(k) {
                json = value;
                key = Some(k);
            } else if let Some((k, value)) = T::FALLBACK_KEY.and_then(|k| Some((k, json.get(k)?))) {
                json = value;
                key = Some(k);
            } else {
                let value = T::FileContent::default();
                return (T::KEY, Ok(DeserializedSetting(Box::new(value))));
            }
        }
        let value = T::FileContent::deserialize(json)
            .map(|value| DeserializedSetting(Box::new(value)))
            .map_err(anyhow::Error::from);
        (key, value)
    }

    fn all_local_values(&self) -> Vec<(WorktreeId, Arc<Path>, &dyn Any)> {
        self.local_values
            .iter()
            .map(|(id, path, value)| (*id, path.clone(), value as _))
            .collect()
    }

    fn value_for_path(&self, path: Option<SettingsLocation>) -> &dyn Any {
        if let Some(SettingsLocation { worktree_id, path }) = path {
            for (settings_root_id, settings_path, value) in self.local_values.iter().rev() {
                if worktree_id == *settings_root_id && path.starts_with(settings_path) {
                    return value;
                }
            }
        }
        self.global_value
            .as_ref()
            .unwrap_or_else(|| panic!("no default value for setting {}", self.setting_type_name()))
    }

    fn set_global_value(&mut self, value: Box<dyn Any>) {
        self.global_value = Some(*value.downcast().unwrap());
    }

    fn set_local_value(&mut self, root_id: WorktreeId, path: Arc<Path>, value: Box<dyn Any>) {
        let value = *value.downcast().unwrap();
        match self
            .local_values
            .binary_search_by_key(&(root_id, &path), |e| (e.0, &e.1))
        {
            Ok(ix) => self.local_values[ix].2 = value,
            Err(ix) => self.local_values.insert(ix, (root_id, path, value)),
        }
    }

    fn json_schema(&self, generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        T::FileContent::json_schema(generator)
    }

    fn edits_for_update(
        &self,
        raw_settings: &serde_json::Value,
        tab_size: usize,
        vscode_settings: &VsCodeSettings,
        text: &mut String,
        edits: &mut Vec<(Range<usize>, String)>,
    ) {
        let (key, deserialized_setting) = self.deserialize_setting_with_key(raw_settings);
        let old_content = match deserialized_setting {
            Ok(content) => content.0.downcast::<T::FileContent>().unwrap(),
            Err(_) => Box::<<T as Settings>::FileContent>::default(),
        };
        let mut new_content = old_content.clone();
        T::import_from_vscode(vscode_settings, &mut new_content);

        let old_value = serde_json::to_value(&old_content).unwrap();
        let new_value = serde_json::to_value(new_content).unwrap();

        let mut key_path = Vec::new();
        if let Some(key) = key {
            key_path.push(key);
        }

        update_value_in_json_text(
            text,
            &mut key_path,
            tab_size,
            &old_value,
            &new_value,
            T::PRESERVED_KEYS.unwrap_or_default(),
            edits,
        );
    }
}
