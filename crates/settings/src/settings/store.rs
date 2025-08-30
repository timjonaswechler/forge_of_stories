use crate::{
    embedded::SettingsAssets,
    keymap::{DeviceFilter, DeviceKind, InputScheme, KeyChord, KeymapState, MergedKeymaps, Mods},
    settings::{Settings, SettingsError},
};
use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;
use toml::Value;
use util::asset_str;

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

pub struct SettingsStoreBuilder {
    app_id: String,
    watch_files: bool,
    layers: Vec<Layer>, // Reihenfolge = Priorität (letztes gewinnt)
}

impl SettingsStoreBuilder {
    pub fn new(app_id: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            watch_files: false,
            layers: vec![],
        }
    }
    pub fn watch_files(mut self, yes: bool) -> Self {
        self.watch_files = yes;
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

pub struct SettingsStore {
    app_id: String,
    layers: RwLock<Vec<Layer>>,

    // Effektive, bereits gemergte Sicht:
    effective_settings: RwLock<HashMap<String, Value>>, // section -> TOML
    effective_keymaps: RwLock<MergedKeymaps>,

    // Registrierte Abschnitte => Snapshots der Modelle
    snapshots: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
    // TODO: optional Watcher/Events
    keymap_state: RwLock<KeymapState>,
}

impl SettingsStore {
    pub fn builder(app_id: impl Into<String>) -> SettingsStoreBuilder {
        SettingsStoreBuilder::new(app_id)
    }

    fn from_builder(b: SettingsStoreBuilder) -> Result<Self, SettingsError> {
        let store = Self {
            app_id: b.app_id,
            layers: RwLock::new(b.layers),
            effective_settings: RwLock::new(HashMap::new()),
            effective_keymaps: RwLock::new(MergedKeymaps::default()),
            snapshots: RwLock::new(HashMap::new()),
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

    pub fn reload_all(&self) -> Result<(), SettingsError> {
        let layers = self.layers.read().unwrap().clone();
        let (merged_settings, merged_keymaps) = load_and_merge_layers(&layers)?;
        *self.effective_settings.write().unwrap() = merged_settings;
        *self.effective_keymaps.write().unwrap() = merged_keymaps;
        // registrierte Models neu bauen/validieren/broadcasten (optional)
        Ok(())
    }

    // Registrierung+Zugriff
    pub fn register<S: Settings>(&self) -> Result<(), SettingsError>
    where
        S::Model: Send + Sync,
    {
        let merged = self
            .effective_settings
            .read()
            .unwrap()
            .get(S::SECTION)
            .cloned()
            .unwrap_or(Value::Table(Default::default()));
        let migrated = S::migrate(merged)?;
        let model: S::Model = toml::from_str(&toml::to_string(&migrated)?)?;
        S::validate(&model)?;
        self.snapshots
            .write()
            .unwrap()
            .insert(TypeId::of::<S::Model>(), Box::new(Arc::new(model)));
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

    // ---- Keymap: (optionale) High-Level-API Platzhalter ----
    pub fn keymap_set_input_scheme(&self, scheme: InputScheme) {
        self.keymap_state.write().unwrap().scheme = scheme;
    }

    /// Exportiere (global + kontext) für eine Eingabeart als Action -> ["chord", ...]
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

        // Helfer: Action-Map eines Kontextes in out mergen (per Gerät last-wins)
        let mut merge_ctx = |ctx_name: &str| {
            if let Some(actions) = km.contexts.get(ctx_name) {
                for (action, device_map) in actions {
                    if let Some(list) = device_map.get(&want_device) {
                        // ggf. für Gamepad nach Kind filtern
                        let chords: Vec<String> = list
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
                            .collect();

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

// ---------- Laden & Mergen ----------

// ---- atomarer Write (einfach & portabel) ----
fn write_atomic(path: &PathBuf, contents: &str) -> Result<(), SettingsError> {
    use std::fs::{File, create_dir_all, rename};
    use std::io::Write;
    let parent = path
        .parent()
        .ok_or(SettingsError::Invalid("invalid path"))?;
    create_dir_all(parent)?;
    let tmp = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name().unwrap().to_string_lossy(),
        std::process::id()
    ));
    {
        let mut f = File::create(&tmp)?;
        f.write_all(contents.as_bytes())?;
        f.sync_all()?;
    }
    rename(&tmp, path)?; // atomic on same fs
    Ok(())
}

fn load_and_merge_layers(
    layers: &[Layer],
) -> Result<(HashMap<String, Value>, MergedKeymaps), SettingsError> {
    let mut settings_stack: Vec<Value> = vec![];
    let mut keymap_stack: Vec<Value> = vec![];

    for layer in layers {
        use LayerKind::*;
        let v = match &layer.kind {
            EmbeddedSettingText(s) | EmbeddedKeyMapText(s) => {
                if s.trim().is_empty() {
                    Value::Table(Default::default())
                } else {
                    toml::from_str::<Value>(s)?
                }
            }
            SettingsFile(p) | KeyMapFile(p) => {
                match std::fs::read_to_string(p) {
                    Ok(txt) if !txt.trim().is_empty() => toml::from_str::<Value>(&txt)?,
                    _ => Value::Table(Default::default()), // fehlend/leer -> neutral
                }
            }
            EnvPrefix(prefix) => env_to_toml(prefix),
        };

        match &layer.kind {
            EmbeddedSettingText(_) | SettingsFile(_) | EnvPrefix(_) => settings_stack.push(v),
            EmbeddedKeyMapText(_) | KeyMapFile(_) => keymap_stack.push(v),
        }
    }

    Ok((
        deep_merge_settings_by_section(&settings_stack)?,
        deep_merge_keymaps(&keymap_stack)?,
    ))
}

// ---- Settings: deep-merge nach Section (last-wins) ----

fn deep_merge_settings_by_section(
    stack: &[Value],
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
            *entry = deep_merge(entry.clone(), val.clone());
        }
    }
    Ok(out)
}

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
                // Sammle neue Buckets pro Gerät
                let mut new_buckets: HashMap<DeviceKind, Vec<KeyChord>> = HashMap::new();
                for s in chords {
                    if let Value::String(s) = s {
                        if let Some(ch) = parse_chord(s) {
                            new_buckets.entry(ch.device).or_default().push(ch);
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

fn deep_merge(a: Value, b: Value) -> Value {
    match (a, b) {
        (Value::Table(mut ta), Value::Table(tb)) => {
            for (k, v2) in tb {
                let v1 = ta.remove(&k);
                ta.insert(
                    k,
                    match v1 {
                        Some(v1) => deep_merge(v1, v2),
                        None => v2,
                    },
                );
            }
            Value::Table(ta)
        }
        // Arrays/Scalars: ersetzen (last-wins)
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
            "ctrl" => mods |= Mods::CTRL,
            "shift" => mods |= Mods::SHIFT,
            "alt" => mods |= Mods::ALT,
            "meta" => mods |= Mods::META,
            _ => key = part.to_string(),
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
