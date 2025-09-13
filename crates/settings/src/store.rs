use crate::{
    embedded::SettingsAssets,
    keymap::{DeviceFilter, DeviceKind, InputScheme, KeyChord, KeymapState, MergedKeymaps, Mods},
    settings::{Settings, SettingsError},
};
use paths::asset_str;
use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;
use toml::Value;

use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub enum LayerKind {
    // Settings:
    EmbeddedSettingText(String), // direkter TOML-Text (z. B. aus Assets)
    SettingsFile(PathBuf),       // Datei (optional erlaubt)
    EnvPrefix(String),           // APP__NETWORK__PORT -> [network].port

    // Keymaps:
    EmbeddedKeyMapText(String), // direkter TOML-Text (z. B. aus Assets)
    KeyMapFile(PathBuf),        // Datei (optional erlaubt)
}

#[derive(Clone, Debug)]
pub struct Layer {
    pub kind: LayerKind,
}

#[derive(Clone, Debug)]
struct RegisteredEntry {
    section: &'static str,
    rebuild: fn(&Value) -> Result<Box<dyn Any + Send + Sync>, SettingsError>,
}

#[derive(Clone, Copy, Debug)]
pub enum MergeArraysPolicy {
    Replace,
    Concat,
    Set,
}

pub struct SettingsStoreBuilder {
    watch_files: bool,
    layers: Vec<Layer>, // Reihenfolge = Priorität (letztes gewinnt)
    merge_arrays_policy: MergeArraysPolicy,
    env_layers_enabled: bool,
}

impl SettingsStoreBuilder {
    pub fn new() -> Self {
        Self {
            watch_files: false,
            layers: vec![],
            merge_arrays_policy: MergeArraysPolicy::Replace,
            env_layers_enabled: true,
        }
    }
    pub fn watch_files(mut self, yes: bool) -> Self {
        self.watch_files = yes;
        self
    }
    pub fn merge_arrays_policy(mut self, policy: MergeArraysPolicy) -> Self {
        self.merge_arrays_policy = policy;
        self
    }

    pub fn enable_env_layers(mut self, yes: bool) -> Self {
        self.env_layers_enabled = yes;
        self
    }

    /// Resolve platform-specific user config directory and register optional
    /// per-user settings and keymap files under `<config_dir>/<app_id>/`.
    ///
    /// Files:
    /// - settings: `<config_dir>/<app_id>/settings.toml`
    /// - keymap:   `<config_dir>/<app_id>/keymap.toml`
    ///
    /// Missing/empty files are treated as neutral layers.
    pub fn with_user_config_dir(mut self) -> Self {
        let app_dir = paths::config_dir().clone();
        self.layers.push(Layer {
            kind: LayerKind::SettingsFile(app_dir.join("settings.toml")),
        });
        self.layers.push(Layer {
            kind: LayerKind::KeyMapFile(app_dir.join("keymap.toml")),
        });
        self
    }

    // ---- Settings (Text/Asset/File) ----
    pub fn with_embedded_setting_text(mut self, toml_text: impl Into<String>) -> Self {
        self.layers.push(Layer {
            kind: LayerKind::EmbeddedSettingText(toml_text.into()),
        });
        self
    }
    pub fn with_embedded_setting_asset(mut self, asset_path: &'static str) -> Self {
        let txt = asset_str::<SettingsAssets>(asset_path).into_owned();
        self.layers.push(Layer {
            kind: LayerKind::EmbeddedSettingText(txt),
        });
        self
    }
    pub fn with_settings_file(mut self, path: PathBuf) -> Self {
        self.layers.push(Layer {
            kind: LayerKind::SettingsFile(path),
        });
        self
    }
    pub fn with_settings_file_optional(mut self, path: PathBuf) -> Self {
        // gleich wie with_settings_file: fehlende/leer wird beim Laden neutral behandelt
        self.layers.push(Layer {
            kind: LayerKind::SettingsFile(path),
        });
        self
    }
    pub fn with_env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.layers.push(Layer {
            kind: LayerKind::EnvPrefix(prefix.into()),
        });
        self
    }

    // ---- Keymaps (Text/Asset/File) ----
    pub fn with_embedded_keymap_text(mut self, toml_text: impl Into<String>) -> Self {
        self.layers.push(Layer {
            kind: LayerKind::EmbeddedKeyMapText(toml_text.into()),
        });
        self
    }
    pub fn with_embedded_keymap_asset(mut self, asset_path: &'static str) -> Self {
        let txt = asset_str::<SettingsAssets>(asset_path).into_owned();
        self.layers.push(Layer {
            kind: LayerKind::EmbeddedKeyMapText(txt),
        });
        self
    }
    pub fn with_keymap_file(mut self, path: PathBuf) -> Self {
        self.layers.push(Layer {
            kind: LayerKind::KeyMapFile(path),
        });
        self
    }
    pub fn with_keymap_file_optional(mut self, path: PathBuf) -> Self {
        self.layers.push(Layer {
            kind: LayerKind::KeyMapFile(path),
        });
        self
    }

    pub fn build(self) -> Result<SettingsStore, SettingsError> {
        SettingsStore::from_builder(self)
    }
}

#[derive(Debug)]
pub struct SettingsStore {
    layers: RwLock<Vec<Layer>>,

    // Effektive, bereits gemergte Sicht:
    effective_settings: RwLock<HashMap<String, Value>>, // section -> TOML
    effective_keymaps: RwLock<MergedKeymaps>,

    // Registrierte Abschnitte => Snapshots der Modelle
    snapshots: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
    registrations: RwLock<HashMap<TypeId, RegisteredEntry>>,
    // Merge policy for arrays in settings deep-merge
    merge_arrays_policy: MergeArraysPolicy,
    env_layers_enabled: bool,
    // TODO: optional Watcher/Events
    keymap_state: RwLock<KeymapState>,
}

impl SettingsStore {
    pub fn builder() -> SettingsStoreBuilder {
        SettingsStoreBuilder::new()
    }

    fn from_builder(b: SettingsStoreBuilder) -> Result<Self, SettingsError> {
        let store = Self {
            layers: RwLock::new(b.layers),
            effective_settings: RwLock::new(HashMap::new()),
            effective_keymaps: RwLock::new(MergedKeymaps::default()),
            snapshots: RwLock::new(HashMap::new()),
            registrations: RwLock::new(HashMap::new()),
            merge_arrays_policy: b.merge_arrays_policy,
            env_layers_enabled: b.env_layers_enabled,
            keymap_state: RwLock::new(KeymapState::default()),
        };
        store.reload_all()?; // initialer Load+Merge
        // TODO: watcher(b.watch_files)
        Ok(store)
    }

    pub fn effective_settings(&self) -> HashMap<String, Value> {
        self.effective_settings.read().unwrap().clone()
    }
    pub fn effective_keymaps(&self) -> MergedKeymaps {
        self.effective_keymaps.read().unwrap().clone()
    }

    pub fn push_layer(&self, layer: Layer) -> Result<(), SettingsError> {
        self.layers.write().unwrap().push(layer);
        self.reload_all()
    }

    /// Update a typed settings section by applying a mutation closure to the current model
    /// and persisting only the differences to the highest-priority writable settings file.
    ///
    /// Rules:
    /// - Writes to the last `LayerKind::SettingsFile` (ENV/Embedded layers are ignored for writing).
    /// - Creates the file if it does not exist (e.g., registered via `with_settings_file_optional`).
    /// - Only values that differ from the embedded defaults are written (minimal delta per section).
    pub fn update<S: Settings>(&self, f: impl FnOnce(&mut S::Model)) -> Result<(), SettingsError> {
        // 1) Resolve current merged section for S
        let merged_section = self
            .effective_settings
            .read()
            .unwrap()
            .get(S::SECTION)
            .cloned()
            .unwrap_or(Value::Table(Default::default()));

        // 2) Build current model with migration + validation
        let migrated = S::migrate(merged_section)?;
        let mut model: S::Model = toml::from_str(&toml::to_string(&migrated)?)?;
        // Apply mutation
        f(&mut model);
        // Validate updated model
        S::validate(&model)?;

        // 3) Serialize updated model back to TOML Value (section value)
        let updated_section_value: Value = toml_model_to_value(&model)?;

        // 4) Determine default section value from embedded defaults
        let default_section_value = self.default_section_value::<S>();

        // 5) Compute delta (only values differing from defaults)
        let delta_opt = toml_diff(&default_section_value, &updated_section_value);

        // 6) Determine highest-priority writable settings file
        let Some(target_path) = self.highest_priority_settings_file() else {
            return Err(SettingsError::Invalid(
                "no writable settings file registered",
            ));
        };

        // 7) Load existing file (if present) or start with empty table
        let root = match std::fs::read_to_string(&target_path) {
            Ok(txt) if !txt.trim().is_empty() => {
                toml::from_str::<Value>(&txt).unwrap_or(Value::Table(Default::default()))
            }
            _ => Value::Table(Default::default()),
        };
        let mut root_tbl = match root {
            Value::Table(t) => t,
            _ => Default::default(),
        };

        // 8) Apply delta for this section
        match delta_opt {
            Some(delta) => {
                root_tbl.insert(S::SECTION.to_string(), delta);
            }
            None => {
                // No differences to defaults; remove section if present.
                root_tbl.remove(S::SECTION);
            }
        }

        // 9) Persist atomically
        let new_text = toml::to_string_pretty(&Value::Table(root_tbl))?;
        write_atomic(&target_path, &new_text)?;

        // 10) Reload effective state and snapshots
        self.reload_all()
    }

    /// Returns the last SettingsFile layer as write target (highest priority), if any.
    fn highest_priority_settings_file(&self) -> Option<PathBuf> {
        let layers = self.layers.read().unwrap();
        for layer in layers.iter().rev() {
            if let LayerKind::SettingsFile(p) = &layer.kind {
                return Some(p.clone());
            }
        }
        None
    }

    /// Compute the default section value for a given `Settings` type by merging only
    /// embedded setting layers (read-only defaults).
    fn default_section_value<S: Settings>(&self) -> Value {
        let layers = self.layers.read().unwrap().clone();
        let mut stack: Vec<Value> = Vec::new();
        for layer in layers {
            match layer.kind {
                LayerKind::EmbeddedSettingText(ref s) => {
                    let v = if s.trim().is_empty() {
                        Value::Table(Default::default())
                    } else {
                        toml::from_str::<Value>(s).unwrap_or(Value::Table(Default::default()))
                    };
                    stack.push(v);
                }
                _ => {}
            }
        }
        let merged =
            deep_merge_settings_by_section(&stack, self.merge_arrays_policy).unwrap_or_default();
        merged
            .get(S::SECTION)
            .cloned()
            .unwrap_or(Value::Table(Default::default()))
    }
    pub fn reload_all(&self) -> Result<(), SettingsError> {
        // 1) Laden und mergen ohne Locks zu halten
        let layers = self.layers.read().unwrap().clone();
        let (new_settings, new_keymaps) =
            load_and_merge_layers(&layers, self.merge_arrays_policy, self.env_layers_enabled)?;

        // 2) Diff der Sections bestimmen, um nur geänderte Snapshots neu zu bauen
        let old_settings = self.effective_settings.read().unwrap().clone();
        let mut changed_sections = std::collections::BTreeSet::new();
        for (k, v_new) in &new_settings {
            match old_settings.get(k) {
                Some(v_old) if v_old == v_new => {}
                _ => {
                    changed_sections.insert(k.clone());
                }
            }
        }
        for k in old_settings.keys() {
            if !new_settings.contains_key(k) {
                changed_sections.insert(k.clone());
            }
        }

        // 3) Registrierte Einträge lesen und neue Snapshots für geänderte Sections vorbereiten
        let regs = self.registrations.read().unwrap().clone();
        let mut rebuilt: Vec<(TypeId, Box<dyn Any + Send + Sync>)> = Vec::new();
        for (type_id, entry) in regs.into_iter() {
            if changed_sections.contains(entry.section) {
                let merged = new_settings
                    .get(entry.section)
                    .cloned()
                    .unwrap_or(Value::Table(Default::default()));
                let snap = (entry.rebuild)(&merged)?;
                rebuilt.push((type_id, snap));
            }
        }

        // 3.5) Compile new TOML-based keymaps to resolved_bindings
        let new_resolved_bindings = self.compile_toml_keymaps_from_layers(&layers)?;
        // log::debug!(
        //     "settings: reload_all compiled {} resolved key bindings (layers={})",
        //     new_resolved_bindings.len(),
        //     layers.len()
        // );

        // 4) Unter konsistenter Lock-Reihenfolge effektiv tauschen und Snapshots aktualisieren
        {
            let mut es = self.effective_settings.write().unwrap();
            let mut ek = self.effective_keymaps.write().unwrap();
            let mut keymap_state = self.keymap_state.write().unwrap();
            let mut snaps = self.snapshots.write().unwrap();

            *es = new_settings;
            *ek = new_keymaps;

            // Update resolved bindings if we found new TOML-based keymaps
            if !new_resolved_bindings.is_empty() {
                keymap_state.resolved_bindings = new_resolved_bindings;
            }

            for (tid, snap) in rebuilt {
                snaps.insert(tid, snap);
            }
        }

        Ok(())
    }

    /// Compiles TOML-based keymaps from layers to resolved bindings
    fn compile_toml_keymaps_from_layers(
        &self,
        layers: &[Layer],
    ) -> Result<Vec<crate::keymap::ResolvedBinding>, SettingsError> {
        use crate::keymap::{KeymapFile, KeymapState};

        let mut combined_bindings = Vec::new();
        // log::debug!(
        //     "settings: compile_toml_keymaps_from_layers: scanning {} layers",
        //     layers.len()
        // );

        // Process layers in order (later layers have higher priority)
        for (idx, layer) in layers.iter().enumerate() {
            let toml_content = match &layer.kind {
                LayerKind::EmbeddedKeyMapText(s) => {
                    if s.trim().is_empty() {
                        // log::debug!(
                        //     "settings: keymap layer {} (embedded) empty/whitespace – skipping",
                        //     idx
                        // );
                        continue;
                    }
                    s.clone()
                }
                LayerKind::KeyMapFile(path) => match std::fs::read_to_string(path) {
                    Ok(content) if !content.trim().is_empty() => content,
                    Ok(_) => {
                        // log::debug!(
                        //     "settings: keymap layer {} file {} empty – skipping",
                        //     idx,
                        //     path.display()
                        // );
                        continue;
                    }
                    Err(err) => {
                        log::debug!(
                            "settings: keymap layer {} file {} not readable ({} ) – skipping",
                            idx,
                            path.display(),
                            err
                        );
                        continue;
                    }
                },
                _ => continue, // Not a keymap layer
            };

            // Try to parse as new TOML keymap format ([[bindings]] groups)
            match toml::from_str::<KeymapFile>(&toml_content) {
                Ok(keymap_file) => {
                    let group_count = keymap_file.bindings.len();
                    match KeymapState::compile_keymap(&keymap_file) {
                        Ok(mut resolved_bindings) => {
                            let added = resolved_bindings.len();
                            // log::debug!(
                            //     "settings: keymap layer {} parsed OK (groups={}, resolved_bindings={})",
                            //     idx,
                            //     group_count,
                            //     added
                            // );
                            combined_bindings.append(&mut resolved_bindings);
                        }
                        Err(e) => {
                            log::warn!(
                                "settings: keymap layer {} compile_keymap failed: {}",
                                idx,
                                e
                            );
                        }
                    }
                }
                Err(parse_err) => {
                    // Not the structured KeymapFile format; we silently ignore here (legacy formats
                    // would be handled by deep_merge_keymaps for effective_keymaps, but that's orthogonal).
                    log::debug!(
                        "settings: keymap layer {} not KeymapFile format (parse error: {}) – ignoring for resolved bindings",
                        idx,
                        parse_err
                    );
                }
            }
        }

        // log::debug!(
        //     "settings: compile_toml_keymaps_from_layers: total resolved bindings = {}",
        //     combined_bindings.len()
        // );
        Ok(combined_bindings)
    }

    // Registrierung+Zugriff
    pub fn register<S: Settings>(&self) -> Result<(), SettingsError>
    where
        S::Model: Send + Sync,
    {
        // aktuelles Merged-Section holen
        let merged = self
            .effective_settings
            .read()
            .unwrap()
            .get(S::SECTION)
            .cloned()
            .unwrap_or(Value::Table(Default::default()));

        // Model bauen und validieren
        let migrated = S::migrate(merged)?;
        let model: S::Model = toml::from_str(&toml::to_string(&migrated)?)?;
        S::validate(&model)?;

        // Registrierung mit Rebuild-Funktion ablegen
        let type_id = TypeId::of::<S::Model>();
        let rebuild_fn: fn(&Value) -> Result<Box<dyn Any + Send + Sync>, SettingsError> =
            build_registered_model::<S>;
        self.registrations.write().unwrap().insert(
            type_id,
            RegisteredEntry {
                section: S::SECTION,
                rebuild: rebuild_fn,
            },
        );

        // Snapshot ablegen
        self.snapshots
            .write()
            .unwrap()
            .insert(type_id, Box::new(Arc::new(model)));
        Ok(())
    }
    pub fn get<S: Settings>(&self) -> Result<Arc<S::Model>, SettingsError> {
        self.snapshots
            .read()
            .unwrap()
            .get(&TypeId::of::<S::Model>())
            .and_then(|b| b.downcast_ref::<Arc<S::Model>>())
            .cloned()
            .ok_or(SettingsError::NotRegistered)
    }

    /// Resolves an action for a key chord and active contexts using the ActionRegistry
    pub fn resolve_action_for_key<R: crate::keymap::ActionRegistry>(
        &self,
        key_chord: &str,
        active_contexts: &[String],
        registry: &R,
    ) -> Option<R::Action> {
        let keymap_state = self.keymap_state.read().unwrap();
        keymap_state.resolve_action_for_key(key_chord, active_contexts, registry)
    }

    /// Debug helper: returns a short summary of the currently compiled (TOML) keymap bindings.
    pub fn debug_keymap_state_summary(&self) -> String {
        let state = self.keymap_state.read().unwrap();
        let total = state.resolved_bindings.len();
        // Show a few of the highest-priority (front of vector after our reverse) bindings for quick inspection
        let sample: Vec<String> = state
            .resolved_bindings
            .iter()
            .take(5)
            .map(|b| format!("{}@{:?}:{}", b.key_chord, b.device, b.action_name))
            .collect();
        format!(
            "resolved_bindings={}, sample=[{}]",
            total,
            sample.join(", ")
        )
    }

    // ---- Keymap: (optionale) High-Level-API Platzhalter ----
    pub fn keymap_set_input_scheme(&self, scheme: InputScheme) {
        self.keymap_state.write().unwrap().scheme = scheme;
    }

    /// Exportiere (global + aktiver Kontext) für eine Eingabeart als Action -> ["chord", ...]. Kontext last-wins pro Gerät.
    pub fn export_keymap_for(
        &self,
        device: DeviceFilter,
        context: &str, // z.B. "login"
    ) -> BTreeMap<String, Vec<String>> {
        // Atomare Sicht: ein Clone aus dem RwLock
        let km = self.effective_keymaps.read().unwrap().clone();

        // Welche buckets wollen wir?
        let want_device = match device {
            DeviceFilter::Keyboard => DeviceKind::Keyboard,
            DeviceFilter::Mouse => DeviceKind::Mouse,
            DeviceFilter::GamepadAny | DeviceFilter::GamepadKind(_) => DeviceKind::Gamepad,
        };

        // Reihenfolge: global (Basis) -> context (überschreibt)
        let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();

        // Fallback: Wenn das Legacy-Kontextmodell leer ist (km.contexts.is_empty()),
        // bauen wir eine Mapping-Ansicht direkt aus den kompilierten resolved_bindings
        // des neuen [[bindings]]-Systems. Damit funktioniert die Help-/Export-Anzeige
        // auch ohne legacy keymap Struktur.
        if km.contexts.is_empty() {
            let state = self.keymap_state.read().unwrap();
            let mut seen = std::collections::HashSet::<(String, String)>::new();
            // Basis-Kontextliste für Evaluierung
            let base_contexts: Vec<&str> = if context == "global" {
                vec!["global"]
            } else {
                vec!["global", context]
            };
            for binding in &state.resolved_bindings {
                if binding.device != want_device {
                    continue;
                }
                // Evaluieren, ob das Binding im (global | global+context) aktiv wäre
                if !binding.context.eval(&base_contexts) {
                    continue;
                }
                let chord = binding.key_chord.clone();
                let key_id = (binding.action_name.clone(), chord.clone());
                if seen.insert(key_id) {
                    out.entry(binding.action_name.clone())
                        .or_default()
                        .push(chord);
                }
            }
            return out;
        }

        // Helfer: Action-Map eines Kontextes in out mergen (per Gerät last-wins)
        let mut merge_ctx = |ctx_name: &str| {
            if let Some(actions) = km.contexts.get(ctx_name) {
                for (action, device_map) in actions {
                    if let Some(list) = device_map.get(&want_device) {
                        // ggf. für Gamepad nach Kind filtern
                        // Dedupliziere stabil in Einfügereihenfolge nach String-Repräsentation
                        let mut seen = std::collections::HashSet::new();
                        let mut chords: Vec<String> = Vec::new();
                        for s in list
                            .iter()
                            .filter(|ch| match &device {
                                DeviceFilter::GamepadKind(kind) => ch
                                    .origin_prefix
                                    .as_deref()
                                    .map(|p| p == kind || p == "gp" || p == "gamepad")
                                    .unwrap_or(true),
                                _ => true,
                            })
                            .map(stringify_chord)
                        {
                            if seen.insert(s.clone()) {
                                chords.push(s);
                            }
                        }

                        if !chords.is_empty() {
                            // last-wins: spätere Kontexte überschreiben die Action
                            out.insert(action.clone(), chords);
                        }
                    }
                }
            }
        };

        merge_ctx("global");
        merge_ctx(context);
        out
    }
}

// Helfer: generische Rebuild-Funktion für registrierte Settings
fn build_registered_model<S: Settings>(
    merged: &Value,
) -> Result<Box<dyn Any + Send + Sync>, SettingsError>
where
    S::Model: Send + Sync,
{
    let migrated = S::migrate(merged.clone())?;
    let model: S::Model = toml::from_str(&toml::to_string(&migrated)?)?;
    S::validate(&model)?;
    Ok(Box::new(Arc::new(model)))
}

// ---------- Laden & Mergen ----------

// ---- atomarer Write (einfach & portabel) ----
fn write_atomic(path: &PathBuf, contents: &str) -> Result<(), SettingsError> {
    use std::fs::create_dir_all;
    use std::io::Write;

    let parent = path
        .parent()
        .ok_or(SettingsError::Invalid("invalid path"))?;
    create_dir_all(parent)?;

    // Unique temp file next to the destination to ensure same-filesystem rename/replace.
    let tmp = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name().unwrap().to_string_lossy(),
        std::process::id()
    ));

    // 1) Write file contents fully and durably to the temp file.
    {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true) // never clobber any pre-existing temp file
            .open(&tmp)?;
        f.write_all(contents.as_bytes())?;
        f.sync_all()?; // flush data + metadata of the temp file
    }

    // 2) Atomically install it and ensure durability of the directory entry.
    #[cfg(target_family = "unix")]
    {
        std::fs::rename(&tmp, path)?; // atomic replace on POSIX
        // fsync the containing directory to persist the rename
        fsync_dir(parent)?;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        // If target exists, use ReplaceFileW for atomic overwrite; otherwise, rename is fine.
        if path.exists() {
            replace_file_atomic(path.as_path(), tmp.as_path())?;
        } else {
            std::fs::rename(&tmp, path)?; // atomic move when dest absent
        }
        return Ok(());
    }

    // Fallback for other targets: best-effort atomic move.
    #[allow(unreachable_code)]
    {
        std::fs::rename(&tmp, path)?;
        Ok(())
    }
}

#[cfg(target_family = "unix")]
fn fsync_dir(dir: &std::path::Path) -> std::io::Result<()> {
    // On Unix, opening a directory and calling sync_all() fsyncs the directory entry updates.
    use std::fs::File;
    let f = File::open(dir)?;
    f.sync_all()
}

#[cfg(target_os = "windows")]
fn replace_file_atomic(dest: &std::path::Path, tmp: &std::path::Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    fn to_wide(os: &std::ffi::OsStr) -> Vec<u16> {
        let mut v: Vec<u16> = os.encode_wide().collect();
        v.push(0);
        v
    }

    // REPLACEFILE_WRITE_THROUGH ensures the operation is flushed to disk.
    const REPLACEFILE_WRITE_THROUGH: u32 = 0x00000001;

    #[link(name = "kernel32")]
    extern "system" {
        fn ReplaceFileW(
            lpReplacedFileName: *const u16,
            lpReplacementFileName: *const u16,
            lpBackupFileName: *const u16,
            dwReplaceFlags: u32,
            lpExclude: *mut std::ffi::c_void,
            lpReserved: *mut std::ffi::c_void,
        ) -> i32;
    }

    let dest_w = to_wide(dest.as_os_str());
    let tmp_w = to_wide(tmp.as_os_str());
    let ok = unsafe {
        ReplaceFileW(
            dest_w.as_ptr(),
            tmp_w.as_ptr(),
            std::ptr::null(),
            REPLACEFILE_WRITE_THROUGH,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if ok == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

/// Load all layers (embedded text/assets, files, env) and merge them.
/// Layer order is priority-ordered: later layers override earlier ones (last-wins).
/// Missing or empty files are treated as neutral (empty table).
/// Returns:
/// - Settings merged per section using deep-merge: tables recurse, arrays/scalars replace (last-wins).
/// - Keymaps merged with context/action device-bucket semantics (see `deep_merge_keymaps`).
fn load_and_merge_layers(
    layers: &[Layer],
    arrays_policy: MergeArraysPolicy,
    env_layers_enabled: bool,
) -> Result<(HashMap<String, Value>, MergedKeymaps), SettingsError> {
    let mut settings_stack: Vec<Value> = vec![];
    let mut keymap_stack: Vec<Value> = vec![];

    for (idx, layer) in layers.iter().enumerate() {
        use LayerKind::*;
        let v = match &layer.kind {
            EmbeddedSettingText(s) | EmbeddedKeyMapText(s) => {
                if s.trim().is_empty() {
                    Value::Table(Default::default())
                } else {
                    match toml::from_str::<Value>(s) {
                        Ok(v) => v,
                        Err(e) => {
                            log::warn!("settings: layer {} embedded TOML parse error: {}", idx, e);
                            Value::Table(Default::default())
                        }
                    }
                }
            }
            SettingsFile(p) | KeyMapFile(p) => match std::fs::read_to_string(p) {
                Ok(txt) if !txt.trim().is_empty() => match toml::from_str::<Value>(&txt) {
                    Ok(v) => v,
                    Err(e) => {
                        log::warn!(
                            "settings: layer {} file {} TOML parse error: {}",
                            idx,
                            p.display(),
                            e
                        );
                        Value::Table(Default::default())
                    }
                },
                Ok(_) => Value::Table(Default::default()),
                Err(e) => {
                    log::warn!(
                        "settings: layer {} file {} read error: {}",
                        idx,
                        p.display(),
                        e
                    );
                    Value::Table(Default::default())
                }
            },
            EnvPrefix(prefix) => {
                if env_layers_enabled {
                    env_to_toml(prefix)
                } else {
                    log::warn!(
                        "settings: layer {} env disabled; ignoring prefix '{}'",
                        idx,
                        prefix
                    );
                    Value::Table(Default::default())
                }
            }
        };

        match &layer.kind {
            EmbeddedSettingText(_) | SettingsFile(_) | EnvPrefix(_) => settings_stack.push(v),
            EmbeddedKeyMapText(_) | KeyMapFile(_) => keymap_stack.push(v),
        }
    }

    Ok((
        deep_merge_settings_by_section(&settings_stack, arrays_policy)?,
        deep_merge_keymaps(&keymap_stack)?,
    ))
}

// ---- Settings: deep-merge nach Section (last-wins) ----

/// Deep-merge settings by top-level section (table).
/// Rules:
/// - Tables are merged recursively (last-wins per key).
/// - Arrays and scalars are replaced entirely (last-wins).
/// Only top-level tables are considered sections; non-table top-level entries are ignored.
fn deep_merge_settings_by_section(
    stack: &[Value],
    arrays_policy: MergeArraysPolicy,
) -> Result<HashMap<String, Value>, SettingsError> {
    let mut out: HashMap<String, Value> = HashMap::new();
    for v in stack {
        let tbl = match v {
            Value::Table(t) => t,
            _ => continue, // nur Tabellen auf Top-Level erlaubt
        };
        for (k, val) in tbl {
            let entry = out
                .entry(k.clone())
                .or_insert(Value::Table(Default::default()));
            *entry = deep_merge(entry.clone(), val.clone(), arrays_policy);
        }
    }
    Ok(out)
}

/// Merge keymaps across layers with last-wins semantics:
/// - Meta fields (devices, gamepad, mouse_enabled) are simple last-wins replacements.
/// - Contexts: for each action, parse chord strings into device buckets; for each device bucket, later
///   layers replace earlier ones (last-wins per device). Non-table/non-array entries are ignored.
fn deep_merge_keymaps(stack: &[Value]) -> Result<MergedKeymaps, SettingsError> {
    let mut out = MergedKeymaps::default();

    for v in stack {
        let tbl = match v {
            Value::Table(t) => t,
            _ => continue,
        };
        // 1) Meta
        if let Some(Value::Table(meta)) = tbl.get("meta") {
            // last-wins: einfache Felder ersetzen
            if let Some(Value::Array(devs)) = meta.get("devices") {
                out.meta.devices = devs.iter().filter_map(|x| parse_device(x)).collect();
            }
            if let Some(Value::String(s)) = meta.get("gamepad") {
                out.meta.gamepad_profile = Some(s.clone());
            }
            if let Some(Value::Boolean(b)) = meta.get("mouse_enabled") {
                out.meta.mouse_enabled = Some(*b);
            }
        }
        // 2) Kontexte (alles außer "meta")
        for (ctx, val) in tbl {
            if ctx == "meta" {
                continue;
            }
            let ctx_tbl = match val {
                Value::Table(t) => t,
                _ => continue,
            };
            let ctx_entry = out.contexts.entry(ctx.clone()).or_default();

            for (action, chords_val) in ctx_tbl {
                // Liste von Strings: pro Gerät-Bucket last-wins
                let chords = match chords_val {
                    Value::Array(a) => a,
                    _ => continue,
                };
                // Sammle neue Buckets pro Gerät (mit stabiler Deduplizierung pro Gerät)
                let mut new_buckets: HashMap<DeviceKind, Vec<KeyChord>> = HashMap::new();
                let mut seen = std::collections::HashSet::<(DeviceKind, Mods, String)>::new();
                for s in chords {
                    if let Value::String(s) = s {
                        if let Some(ch) = parse_chord(s) {
                            let key_id = (ch.device, ch.mods, ch.key.clone());
                            if seen.insert(key_id) {
                                new_buckets.entry(ch.device).or_default().push(ch);
                            }
                        }
                    }
                }
                let act_entry = ctx_entry.entry(action.clone()).or_default();
                // last-wins je Gerät
                for (dev, list) in new_buckets {
                    act_entry.insert(dev, list);
                }
            }
        }
    }
    Ok(out)
}

/// Core deep-merge primitive used for settings:
/// - If both sides are tables, merge recursively (last-wins per key).
/// - Otherwise (arrays and scalars), return the right-hand side (replace).
fn deep_merge(a: Value, b: Value, arrays_policy: MergeArraysPolicy) -> Value {
    match (a, b) {
        (Value::Table(mut ta), Value::Table(tb)) => {
            for (k, v2) in tb {
                let v1 = ta.remove(&k);
                ta.insert(
                    k,
                    match v1 {
                        Some(v1) => deep_merge(v1, v2, arrays_policy),
                        None => v2,
                    },
                );
            }
            Value::Table(ta)
        }
        (Value::Array(mut aa), Value::Array(bb)) => match arrays_policy {
            MergeArraysPolicy::Replace => Value::Array(bb),
            MergeArraysPolicy::Concat => {
                aa.extend(bb);
                Value::Array(aa)
            }
            MergeArraysPolicy::Set => {
                for v in bb {
                    if !aa.contains(&v) {
                        aa.push(v);
                    }
                }
                Value::Array(aa)
            }
        },
        // Scalars or differing types: replace (last-wins)
        (_v1, v2) => v2,
    }
}

// ---- Env → TOML (Optional: hier neutral) ----
fn env_to_toml(_prefix: &str) -> Value {
    Value::Table(Default::default())
}

fn parse_device(v: &Value) -> Option<DeviceKind> {
    match v {
        Value::String(s) => match s.to_ascii_lowercase().as_str() {
            "keyboard" => Some(DeviceKind::Keyboard),
            "mouse" => Some(DeviceKind::Mouse),
            "gamepad" => Some(DeviceKind::Gamepad),
            _ => None,
        },
        _ => None,
    }
}

// Sehr einfache Parser-Variante (erweiterbar)
fn parse_chord(s: &str) -> Option<KeyChord> {
    let (device, origin_prefix, rest) = if let Some((pref, tail)) = s.split_once(':') {
        let pref_l = pref.to_ascii_lowercase();
        match pref_l.as_str() {
            "mouse" => (DeviceKind::Mouse, Some(pref_l), tail),
            "xbox" | "dualshock" | "gp" | "gamepad" => (DeviceKind::Gamepad, Some(pref_l), tail),
            _ => (DeviceKind::Keyboard, None, s),
        }
    } else {
        (DeviceKind::Keyboard, None, s)
    };

    let mut mods = Mods::empty();
    let mut key = String::new();
    for part in rest.split('+') {
        let part_lower = part.to_ascii_lowercase();
        match part_lower.as_str() {
            "ctrl" => {
                mods |= Mods::CTRL;
            }
            "shift" => {
                mods |= Mods::SHIFT;
            }
            "alt" => {
                mods |= Mods::ALT;
            }
            "meta" => {
                mods |= Mods::META;
            }
            // Key-Synonyme/Normalisierung
            "esc" | "escape" => {
                key = "esc".to_string();
            }
            "enter" | "return" => {
                key = "enter".to_string();
            }
            "space" | "spc" => {
                key = "space".to_string();
            }
            _ => {
                // Einzelne Zeichen wie '?' als Literal behandeln.
                // Keine Layout-spezifische Expansion (z. B. shift+/ vs. shift+ß).
                key = part.to_string();
            }
        }
    }
    if key.is_empty() {
        return None;
    }
    Some(KeyChord {
        device,
        mods,
        key,
        origin_prefix,
    })
}

// ---------- TOML diff (write only deviations from defaults) ----------

/// Convert a serializable model to a TOML `Value` by round-tripping through string form.
fn toml_model_to_value<T: serde::Serialize>(m: &T) -> Result<Value, SettingsError> {
    let s = toml::to_string(m)?;
    Ok(toml::from_str::<Value>(&s)?)
}

/// Compute the difference from `default` to `current`.
/// Returns `None` if identical, or a `Value` containing only keys that differ.
fn toml_diff(default: &Value, current: &Value) -> Option<Value> {
    match (default, current) {
        (Value::Table(a), Value::Table(b)) => {
            let mut out = toml::map::Map::new();
            for (k, v_curr) in b.iter() {
                match a.get(k) {
                    Some(v_def) => {
                        if let Some(sub) = toml_diff(v_def, v_curr) {
                            out.insert(k.clone(), sub);
                        }
                    }
                    None => {
                        out.insert(k.clone(), v_curr.clone());
                    }
                }
            }
            if out.is_empty() {
                None
            } else {
                Some(Value::Table(out))
            }
        }
        // Arrays and scalars: include only if different
        (a, b) => {
            if a == b {
                None
            } else {
                Some(b.clone())
            }
        }
    }
}

/// String-Repräsentation wie in deinen TOMLs
fn stringify_chord(ch: &KeyChord) -> String {
    let mods = [
        (Mods::CTRL, "ctrl"),
        (Mods::SHIFT, "shift"),
        (Mods::ALT, "alt"),
        (Mods::META, "meta"),
    ]
    .into_iter()
    .filter_map(|(m, name)| {
        if ch.mods.contains(m) {
            Some(name)
        } else {
            None
        }
    })
    .collect::<Vec<_>>()
    .join("+");

    // Gerätepräfix nur setzen, wenn es ursprünglich vorhanden war
    let prefix = match (&ch.device, &ch.origin_prefix) {
        (DeviceKind::Keyboard, _) => None,
        (_, Some(p)) if !p.is_empty() => Some(p.as_str()),
        (DeviceKind::Mouse, _) => Some("mouse"),
        (DeviceKind::Gamepad, _) => Some("gp"),
    };

    match (prefix, mods.is_empty()) {
        (Some(pref), true) => format!("{pref}:{}", ch.key),
        (Some(pref), false) => format!("{pref}:{}+{}", mods, ch.key),
        (None, true) => ch.key.clone(),
        (None, false) => format!("{mods}+{}", ch.key),
    }
}
