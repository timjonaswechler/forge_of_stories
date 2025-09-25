use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::{
    Settings, SettingsError,
    json_utils::{parse_json_with_comments, to_pretty_json},
};

use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map as JsonMap, Value as JsonValue};

/// Convert any serializable struct to `serde_json::Value`.
fn to_json_value<T: Serialize>(value: &T) -> Result<JsonValue, SettingsError> {
    serde_json::to_value(value).map_err(SettingsError::Json)
}

/// Merge default + delta recursively (objects only).
fn merge_maps(
    default: &JsonMap<String, JsonValue>,
    delta: &JsonMap<String, JsonValue>,
) -> JsonMap<String, JsonValue> {
    let mut merged = default.clone();
    for (k, v_delta) in delta.iter() {
        if let Some(v_def) = merged.get(k) {
            match (v_def, v_delta) {
                (JsonValue::Object(def_m), JsonValue::Object(delta_m)) => {
                    let rec = merge_maps(def_m, delta_m);
                    merged.insert(k.clone(), JsonValue::Object(rec));
                }
                _ => {
                    merged.insert(k.clone(), v_delta.clone());
                }
            }
        } else {
            merged.insert(k.clone(), v_delta.clone());
        }
    }
    merged
}

/// Compute recursive diff (new vs default). Returns None if identical.
fn diff_value(new_v: &JsonValue, default_v: &JsonValue) -> Option<JsonValue> {
    match (new_v, default_v) {
        (JsonValue::Object(new_m), JsonValue::Object(def_m)) => {
            let diff_m = diff_map(new_m, def_m);
            if diff_m.is_empty() {
                None
            } else {
                Some(JsonValue::Object(diff_m))
            }
        }
        _ => {
            if new_v == default_v {
                None
            } else {
                Some(new_v.clone())
            }
        }
    }
}

fn diff_map(
    new_m: &JsonMap<String, JsonValue>,
    def_m: &JsonMap<String, JsonValue>,
) -> JsonMap<String, JsonValue> {
    let mut out = JsonMap::new();
    for (k, new_v) in new_m.iter() {
        match def_m.get(k) {
            Some(def_v) => {
                if let Some(d) = diff_value(new_v, def_v) {
                    out.insert(k.clone(), d);
                }
            }
            None => {
                out.insert(k.clone(), new_v.clone());
            }
        }
    }
    out
}

/// Builder for `SettingsStore` (single delta file).
pub struct SettingsStoreBuilder {
    settings_file: Option<PathBuf>,
    logger: Option<Arc<dyn Fn(&str) + Send + Sync>>,
}

impl SettingsStoreBuilder {
    pub fn new() -> Self {
        Self {
            settings_file: None,
            logger: None,
        }
    }

    pub fn with_settings_file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.settings_file = Some(path.into());
        self
    }

    /// Provide a logger callback; if unset falls back to eprintln!.
    pub fn with_logger<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.logger = Some(Arc::new(f));
        self
    }

    pub fn build(self) -> Result<SettingsStore, SettingsError> {
        let file_path = self
            .settings_file
            .ok_or(SettingsError::Invalid("settings file not specified"))?;

        if let Some(dir) = file_path.parent() {
            if !dir.exists() {
                fs::create_dir_all(dir)?;
            }
        }

        let delta_map: JsonMap<String, JsonValue> = if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;

            if content.trim().is_empty() {
                JsonMap::new()
            } else {
                parse_json_with_comments(&content)
                    .map_err(|_| SettingsError::Invalid("parse settings file"))?
            }
        } else {
            JsonMap::new()
        };

        Ok(SettingsStore {
            file_path,
            deltas: RwLock::new(delta_map),
            defaults: RwLock::new(HashMap::new()),
            values: RwLock::new(HashMap::new()),
            logger: RwLock::new(self.logger),
        })
    }
}

/// Settings store (thread-safe).
///
/// Features added:
/// - Recursive diff/merge (nested maps)
/// - `try_get` (Option)
/// - `reload` (re-read delta file, re-merge sections)

pub struct SettingsStore {
    file_path: PathBuf,
    deltas: RwLock<JsonMap<String, JsonValue>>, // section -> delta value (usually object)
    defaults: RwLock<HashMap<&'static str, JsonMap<String, JsonValue>>>, // section -> full default map
    values: RwLock<HashMap<&'static str, JsonValue>>, // section -> full effective merged value
    logger: RwLock<Option<Arc<dyn Fn(&str) + Send + Sync>>>, // optional logging hook
}

impl SettingsStore {
    pub fn builder() -> SettingsStoreBuilder {
        SettingsStoreBuilder::new()
    }

    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }

    /// Check if a section is already registered.
    pub fn is_registered<T>(&self) -> bool
    where
        T: Settings,
    {
        let section = T::name();
        self.values.read().unwrap().contains_key(section)
    }

    /// Install / replace logger at runtime.
    pub fn set_logger<F>(&self, f: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        *self.logger.write().unwrap() = Some(Arc::new(f));
    }

    /// Register a section type (loads defaults and applies existing delta if present).
    pub fn register<T>(&self) -> Result<(), SettingsError>
    where
        T: Settings + Default + Serialize + DeserializeOwned,
    {
        let section = T::name();

        {
            if self.values.read().unwrap().contains_key(section) {
                return Err(SettingsError::Invalid("section already registered"));
            }
        }

        // Serialize defaults to serde_json::Value::Object
        let default_val = to_json_value(&T::default())?;
        let default_map = match default_val {
            JsonValue::Object(m) => m,
            _ => return Err(SettingsError::Invalid("default must serialize to map")),
        };

        // Merge default + delta (if any)
        let merged_value = {
            let deltas = self.deltas.read().unwrap();
            if let Some(delta) = deltas.get(section) {
                match delta {
                    JsonValue::Object(delta_m) => {
                        JsonValue::Object(merge_maps(&default_map, delta_m))
                    }
                    other => other.clone(), // unexpected but take it
                }
            } else {
                JsonValue::Object(default_map.clone())
            }
        };

        self.defaults
            .write()
            .unwrap()
            .insert(section, default_map.clone());
        self.values.write().unwrap().insert(section, merged_value);

        Ok(())
    }

    /// Snapshot get (Arc).
    pub fn get<T>(&self) -> Result<Arc<T>, SettingsError>
    where
        T: Settings + DeserializeOwned,
    {
        let section = T::name();
        let values = self.values.read().unwrap();
        let value = values.get(section).ok_or(SettingsError::NotRegistered)?;
        let inst: T = serde_json::from_value(value.clone()).map_err(SettingsError::Json)?;
        Ok(Arc::new(inst))
    }

    /// Optional variant: None if not registered.
    pub fn try_get<T>(&self) -> Result<Option<Arc<T>>, SettingsError>
    where
        T: Settings + DeserializeOwned,
    {
        let section = T::name();
        let values = self.values.read().unwrap();
        let Some(value) = values.get(section) else {
            return Ok(None);
        };
        let inst: T = serde_json::from_value(value.clone()).map_err(SettingsError::Json)?;
        Ok(Some(Arc::new(inst)))
    }

    /// Update via mutable closure. Only delta (recursive) is persisted.
    pub fn update<T, F>(&self, mutator: F) -> Result<(), SettingsError>
    where
        T: Settings + Default + Serialize + DeserializeOwned,
        F: FnOnce(&mut T),
    {
        let section = T::name();

        // Current
        let current_value: T = {
            let values = self.values.read().unwrap();
            let raw = values.get(section).ok_or(SettingsError::NotRegistered)?;
            serde_json::from_value(raw.clone()).map_err(SettingsError::Json)?
        };

        // Mutate
        let mut new_instance = current_value;
        mutator(&mut new_instance);

        // Serialize new
        let new_val = to_json_value(&new_instance)?;
        let new_map = match new_val {
            JsonValue::Object(m) => m,
            _ => return Err(SettingsError::Invalid("updated must serialize to map")),
        };

        // Defaults
        let defaults_guard = self.defaults.read().unwrap();
        let default_map = defaults_guard
            .get(section)
            .ok_or(SettingsError::NotRegistered)?;

        // Compute recursive diff
        let diff_root = diff_map(&new_map, default_map);

        // Update merged full value
        self.values
            .write()
            .unwrap()
            .insert(section, JsonValue::Object(new_map.clone()));

        // Update deltas
        {
            let mut deltas = self.deltas.write().unwrap();
            if diff_root.is_empty() {
                deltas.remove(section);
            } else {
                deltas.insert(section.to_string(), JsonValue::Object(diff_root));
            }
        }

        self.persist_deltas()?;
        Ok(())
    }

    /// Reload deltas from disk and re-merge all registered sections.
    pub fn reload(&self) -> Result<(), SettingsError> {
        let content = if self.file_path.exists() {
            fs::read_to_string(&self.file_path)?
        } else {
            String::new()
        };

        let new_deltas: JsonMap<String, JsonValue> = if content.trim().is_empty() {
            JsonMap::new()
        } else {
            parse_json_with_comments(&content)
                .map_err(|_| SettingsError::Invalid("reload parse settings file"))?
        };

        {
            // Replace deltas
            let mut deltas_guard = self.deltas.write().unwrap();
            *deltas_guard = new_deltas;
        }

        // Re-merge for all registered sections
        let (sections, defaults_snapshot): (
            Vec<&'static str>,
            HashMap<&'static str, JsonMap<String, JsonValue>>,
        ) = {
            let defs = self.defaults.read().unwrap();
            (defs.keys().cloned().collect(), defs.clone())
        };

        let deltas_guard = self.deltas.read().unwrap();
        let mut values_guard = self.values.write().unwrap();

        for section in sections {
            if let Some(default_map) = defaults_snapshot.get(section) {
                let merged = if let Some(delta_val) = deltas_guard.get(section) {
                    match delta_val {
                        JsonValue::Object(delta_m) => {
                            JsonValue::Object(merge_maps(default_map, delta_m))
                        }
                        other => other.clone(),
                    }
                } else {
                    JsonValue::Object(default_map.clone())
                };
                values_guard.insert(section, merged);
            }
        }

        Ok(())
    }

    fn persist_deltas(&self) -> Result<(), SettingsError> {
        let deltas_guard = self.deltas.read().unwrap();

        // Keep only non-empty object deltas
        let mut clean = JsonMap::new();
        for (k, v) in deltas_guard.iter() {
            match v {
                JsonValue::Object(m) if m.is_empty() => {}
                _ => {
                    clean.insert(k.clone(), v.clone());
                }
            }
        }

        let json_string = to_pretty_json(&clean, 4, 0);

        let tmp = self.file_path.with_extension("tmp");
        fs::write(&tmp, json_string)?;
        fs::rename(&tmp, &self.file_path)?;
        Ok(())
    }

    /// Remove stale / orphaned delta entries:
    /// * Sections not registered (no defaults) are dropped.
    /// * Keys inside a section that no longer exist in defaults are pruned recursively.
    /// * Empty sections after pruning are removed.
    ///
    /// Returns Ok after persisting (even if nothing changed).
    pub fn prune_stale(&self) -> Result<(), SettingsError> {
        // Snapshot defaults
        let defaults_snapshot: HashMap<&'static str, JsonMap<String, JsonValue>> = {
            let defs = self.defaults.read().unwrap();
            defs.clone()
        };

        // Work on a mutable copy of deltas
        {
            let mut deltas = self.deltas.write().unwrap();

            // Collect section names to iterate (avoid borrow issues)
            let section_names: Vec<String> = deltas.keys().cloned().collect();

            for section in section_names {
                // If section not in defaults -> remove entirely
                let default_map = match defaults_snapshot.get(section.as_str()) {
                    Some(m) => m,
                    None => {
                        deltas.remove(&section);
                        continue;
                    }
                };

                // If delta value is not a map just keep (cannot reconcile structure) unless defaults is map
                let delta_val = match deltas.get_mut(&section) {
                    Some(v) => v,
                    None => continue,
                };

                if let JsonValue::Object(delta_map) = delta_val {
                    let changed = Self::prune_map_recursive(default_map, delta_map);
                    if changed && delta_map.is_empty() {
                        deltas.remove(&section);
                    }
                }
            }
        }

        // Persist updated deltas
        self.persist_deltas()
    }

    /// Recursively prune keys in `candidate` that do not exist in `default_ref`.
    /// Returns true if any modification was made.
    fn prune_map_recursive(
        default_ref: &JsonMap<String, JsonValue>,
        candidate: &mut JsonMap<String, JsonValue>,
    ) -> bool {
        let mut to_remove: Vec<String> = Vec::new();
        let mut changed = false;

        for (k, v) in candidate.iter_mut() {
            if !default_ref.contains_key(k) {
                to_remove.push(k.clone());
                continue;
            }
            if let (Some(JsonValue::Object(def_sub)), JsonValue::Object(cand_sub)) =
                (default_ref.get(k), v)
            {
                if Self::prune_map_recursive(def_sub, cand_sub) {
                    changed = true;
                }
                if cand_sub.is_empty() {
                    to_remove.push(k.clone());
                }
            }
        }

        if !to_remove.is_empty() {
            for k in to_remove {
                candidate.remove(&k);
            }
            changed = true;
        }

        changed
    }
}

// (file watcher functionality removed; doc comment left intentionally cleared)
