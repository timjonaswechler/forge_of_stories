use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::{Settings, SettingsError};

use ron::value::{Map as RonMap, Value as RonValue};
use serde::{Serialize, de::DeserializeOwned};

/// Convert any serializable struct to `ron::Value`.
fn to_ron_value<T: Serialize>(value: &T) -> Result<RonValue, SettingsError> {
    let s = ron::to_string(value).map_err(SettingsError::Ron)?;
    let v: RonValue =
        ron::from_str(&s).map_err(|_| SettingsError::Invalid("parse ron value (internal)"))?;
    Ok(v)
}

/// Merge default + delta recursively (maps only).
fn merge_maps(default: &RonMap, delta: &RonMap) -> RonMap {
    let mut merged = default.clone();
    for (k, v_delta) in delta.iter() {
        if let Some(v_def) = merged.get(k) {
            match (v_def, v_delta) {
                (RonValue::Map(def_m), RonValue::Map(delta_m)) => {
                    let rec = merge_maps(def_m, delta_m);
                    merged.insert(k.clone(), RonValue::Map(rec));
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
fn diff_value(new_v: &RonValue, default_v: &RonValue) -> Option<RonValue> {
    match (new_v, default_v) {
        (RonValue::Map(new_m), RonValue::Map(def_m)) => {
            let diff_m = diff_map(new_m, def_m);
            if diff_m.is_empty() {
                None
            } else {
                Some(RonValue::Map(diff_m))
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

fn diff_map(new_m: &RonMap, def_m: &RonMap) -> RonMap {
    let mut out = RonMap::new();
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

        let delta_map: HashMap<String, RonValue> = if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;

            if content.trim().is_empty() {
                HashMap::new()
            } else {
                ron::from_str(&content)
                    .map_err(|_| SettingsError::Invalid("parse settings file"))?
            }
        } else {
            HashMap::new()
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
    deltas: RwLock<HashMap<String, RonValue>>, // section -> delta value (usually Map)
    defaults: RwLock<HashMap<&'static str, RonMap>>, // section -> full default map
    values: RwLock<HashMap<&'static str, RonValue>>, // section -> full effective merged value
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

        // Serialize defaults to ron::Value::Map
        let default_val = to_ron_value(&T::default())
            .map_err(|_| SettingsError::Invalid("serialize default failed"))?;
        let default_map = match default_val {
            RonValue::Map(m) => m,
            _ => return Err(SettingsError::Invalid("default must serialize to map")),
        };

        // Merge default + delta (if any)
        let merged_value = {
            let deltas = self.deltas.read().unwrap();
            if let Some(delta) = deltas.get(section) {
                match delta {
                    RonValue::Map(delta_m) => RonValue::Map(merge_maps(&default_map, delta_m)),
                    other => other.clone(), // unexpected but take it
                }
            } else {
                RonValue::Map(default_map.clone())
            }
        };

        self.defaults.write().unwrap().insert(section, default_map);
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
        let ron_str =
            ron::to_string(value).map_err(|_| SettingsError::Invalid("serialize section view"))?;
        let inst: T =
            ron::from_str(&ron_str).map_err(|_| SettingsError::Invalid("deserialize section"))?;
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
        let ron_str =
            ron::to_string(value).map_err(|_| SettingsError::Invalid("serialize section view"))?;
        let inst: T =
            ron::from_str(&ron_str).map_err(|_| SettingsError::Invalid("deserialize section"))?;
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
            let s = ron::to_string(raw)
                .map_err(|_| SettingsError::Invalid("serialize current section"))?;
            ron::from_str(&s).map_err(|_| SettingsError::Invalid("deserialize current section"))?
        };

        // Mutate
        let mut new_instance = current_value;
        mutator(&mut new_instance);

        // Serialize new
        let new_val =
            to_ron_value(&new_instance).map_err(|_| SettingsError::Invalid("serialize updated"))?;
        let new_map = match new_val {
            RonValue::Map(m) => m,
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
            .insert(section, RonValue::Map(new_map));

        // Update deltas
        {
            let mut deltas = self.deltas.write().unwrap();
            if diff_root.is_empty() {
                deltas.remove(section);
            } else {
                deltas.insert(section.to_string(), RonValue::Map(diff_root));
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

        let new_deltas: HashMap<String, RonValue> = if content.trim().is_empty() {
            HashMap::new()
        } else {
            ron::from_str(&content)
                .map_err(|_| SettingsError::Invalid("reload parse settings file"))?
        };

        {
            // Replace deltas
            let mut deltas_guard = self.deltas.write().unwrap();
            *deltas_guard = new_deltas;
        }

        // Re-merge for all registered sections
        let (sections, defaults_snapshot): (Vec<&'static str>, HashMap<&'static str, RonMap>) = {
            let defs = self.defaults.read().unwrap();
            (defs.keys().cloned().collect(), defs.clone())
        };

        let deltas_guard = self.deltas.read().unwrap();
        let mut values_guard = self.values.write().unwrap();

        for section in sections {
            if let Some(default_map) = defaults_snapshot.get(section) {
                let merged = if let Some(delta_val) = deltas_guard.get(section) {
                    match delta_val {
                        RonValue::Map(delta_m) => RonValue::Map(merge_maps(default_map, delta_m)),
                        other => other.clone(),
                    }
                } else {
                    RonValue::Map(default_map.clone())
                };
                values_guard.insert(section, merged);
            }
        }

        Ok(())
    }

    fn persist_deltas(&self) -> Result<(), SettingsError> {
        let deltas_guard = self.deltas.read().unwrap();

        // Keep only non-empty map deltas
        let mut clean: HashMap<String, RonValue> = HashMap::new();
        for (k, v) in deltas_guard.iter() {
            match v {
                RonValue::Map(m) if m.is_empty() => {}
                _ => {
                    clean.insert(k.clone(), v.clone());
                }
            }
        }

        let pretty = ron::ser::PrettyConfig::default();
        let ron_string = ron::ser::to_string_pretty(&clean, pretty).map_err(SettingsError::Ron)?;

        let tmp = self.file_path.with_extension("tmp");
        fs::write(&tmp, ron_string)?;
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
        let defaults_snapshot: HashMap<&'static str, RonMap> = {
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

                // Only prune maps recursively
                match delta_val {
                    RonValue::Map(delta_map) => {
                        let changed = Self::prune_map_recursive(default_map, delta_map);
                        // After recursion: remove empty
                        if changed && delta_map.is_empty() {
                            deltas.remove(&section);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Persist updated deltas
        self.persist_deltas()
    }

    /// Recursively prune keys in `candidate` that do not exist in `default_ref`.
    /// Returns true if any modification was made.
    fn prune_map_recursive(default_ref: &RonMap, candidate: &mut RonMap) -> bool {
        let mut to_remove: Vec<ron::Value> = Vec::new();
        let mut changed = false;

        for (k, v) in candidate.iter_mut() {
            if default_ref.get(k).is_none() {
                to_remove.push(k.clone());
                continue;
            }
            if let (Some(RonValue::Map(def_sub)), RonValue::Map(cand_sub)) = (default_ref.get(k), v)
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
