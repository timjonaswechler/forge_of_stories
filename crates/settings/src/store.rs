use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::{
    json_utils::{parse_json_with_comments, to_pretty_json},
    Settings, SettingsError,
};

use semver::Version;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};

const META_KEY: &str = "__meta";
const META_SCHEMA_VERSION_KEY: &str = "version";

type MergeFn = fn(
    &JsonMap<String, JsonValue>,
    Option<&JsonValue>,
    Option<&Version>,
    &Version,
) -> Result<SectionMergeOutcome, SettingsError>;

#[derive(Copy, Clone)]
struct SectionRuntime {
    merge: MergeFn,
}

struct SectionMergeOutcome {
    merged_value: JsonValue,
    delta_value: Option<JsonValue>,
    delta_changed: bool,
    migrated: bool,
    pruned_unknown_fields: Vec<String>,
}

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

fn prune_unknown_fields(
    default_ref: &JsonMap<String, JsonValue>,
    candidate: &mut JsonMap<String, JsonValue>,
) -> Vec<String> {
    fn prune_recursive(
        default_ref: &JsonMap<String, JsonValue>,
        candidate: &mut JsonMap<String, JsonValue>,
        prefix: &str,
        removed: &mut Vec<String>,
    ) {
        let keys: Vec<String> = candidate.keys().cloned().collect();

        for key in keys {
            let full_path = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{prefix}.{key}")
            };

            match (default_ref.get(&key), candidate.get_mut(&key)) {
                (None, _) => {
                    candidate.remove(&key);
                    removed.push(full_path);
                }
                (Some(JsonValue::Object(def_sub)), Some(JsonValue::Object(cand_sub))) => {
                    prune_recursive(def_sub, cand_sub, &full_path, removed);
                    if cand_sub.is_empty() {
                        candidate.remove(&key);
                    }
                }
                _ => {}
            }
        }
    }

    let mut removed = Vec::new();
    prune_recursive(default_ref, candidate, "", &mut removed);
    removed
}

fn split_meta(
    mut document: JsonMap<String, JsonValue>,
) -> Result<(JsonMap<String, JsonValue>, Option<Version>), SettingsError> {
    let mut version = None;

    if let Some(meta_value) = document.remove(META_KEY) {
        let meta_obj = meta_value
            .as_object()
            .ok_or(SettingsError::Invalid("meta section must be an object"))?;

        if let Some(JsonValue::String(ver)) = meta_obj.get(META_SCHEMA_VERSION_KEY) {
            version = Some(
                Version::parse(ver)
                    .map_err(|_| SettingsError::Invalid("invalid schema version string"))?,
            );
        }
    }

    Ok((document, version))
}

fn encode_meta(version: &Version) -> JsonValue {
    let mut meta_obj = JsonMap::new();
    meta_obj.insert(
        META_SCHEMA_VERSION_KEY.to_string(),
        JsonValue::String(version.to_string()),
    );

    JsonValue::Object(meta_obj)
}

fn merge_for_settings<T>(
    default_map: &JsonMap<String, JsonValue>,
    existing_delta: Option<&JsonValue>,
    file_version: Option<&Version>,
    target_version: &Version,
) -> Result<SectionMergeOutcome, SettingsError>
where
    T: Settings + Default + Serialize + DeserializeOwned,
{
    if let Some(file_version) = file_version {
        if file_version > target_version {
            return Err(SettingsError::Invalid(
                "settings schema from future version",
            ));
        }
    }

    let merged_value = match existing_delta {
        Some(JsonValue::Object(delta_m)) => JsonValue::Object(merge_maps(default_map, delta_m)),
        Some(other) => other.clone(),
        None => JsonValue::Object(default_map.clone()),
    };

    let (working_value, migrated) = T::migrate(file_version, target_version, merged_value)?;

    let mut object_map = match working_value {
        JsonValue::Object(map) => map,
        _ => return Err(SettingsError::Invalid("settings must serialize to map")),
    };

    let pruned_unknown_fields = prune_unknown_fields(default_map, &mut object_map);

    let diff_root = diff_map(&object_map, default_map);
    let delta_value = if diff_root.is_empty() {
        None
    } else {
        Some(JsonValue::Object(diff_root))
    };

    let normalize_delta = |value: &JsonValue| -> Option<JsonValue> {
        match value {
            JsonValue::Object(map) if map.is_empty() => None,
            other => Some(other.clone()),
        }
    };

    let existing_normalized = existing_delta.and_then(|v| normalize_delta(v));
    let delta_changed = match (&existing_normalized, &delta_value) {
        (None, None) => migrated,
        (Some(a), Some(b)) => a != b,
        _ => true,
    };

    Ok(SectionMergeOutcome {
        merged_value: JsonValue::Object(object_map.clone()),
        delta_value,
        delta_changed,
        migrated,
        pruned_unknown_fields,
    })
}

impl SectionRuntime {
    fn new<T>() -> Self
    where
        T: Settings + Default + Serialize + DeserializeOwned,
    {
        Self {
            merge: merge_for_settings::<T>,
        }
    }
}

/// Builder for `SettingsStore` (single delta file).
pub struct SettingsStoreBuilder {
    settings_file: Option<PathBuf>,
    logger: Option<Arc<dyn Fn(&str) + Send + Sync>>,
    schema_version: Version,
}

impl SettingsStoreBuilder {
    pub fn new(version: &'static str) -> Self {
        Self {
            settings_file: None,
            logger: None,
            schema_version: Version::parse(version)
                .expect("invalid crate version for settings schema"),
        }
    }

    pub fn with_settings_file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.settings_file = Some(path.into());
        self
    }

    pub fn with_schema_version(mut self, version: Version) -> Self {
        self.schema_version = version;
        self
    }

    /// Provide a logger callback; if unset falls back to the global tracing
    /// subscriber when available (otherwise `eprintln!`).
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

        let raw_map: JsonMap<String, JsonValue> = if file_path.exists() {
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

        let (delta_map, stored_version) = split_meta(raw_map)?;
        let stored_version_for_loaded = stored_version.clone();

        if let Some(ref version) = stored_version {
            if version > &self.schema_version {
                return Err(SettingsError::Invalid("settings file targets newer schema"));
            }
        }

        Ok(SettingsStore {
            file_path,
            deltas: RwLock::new(delta_map),
            defaults: RwLock::new(HashMap::new()),
            values: RwLock::new(HashMap::new()),
            target_schema_version: self.schema_version,
            loaded_file_schema_version: RwLock::new(stored_version_for_loaded),
            file_schema_version: RwLock::new(stored_version),
            section_runtimes: RwLock::new(HashMap::new()),
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
    target_schema_version: Version,
    loaded_file_schema_version: RwLock<Option<Version>>,
    file_schema_version: RwLock<Option<Version>>,
    section_runtimes: RwLock<HashMap<&'static str, SectionRuntime>>, // migration helpers per section
    logger: RwLock<Option<Arc<dyn Fn(&str) + Send + Sync>>>,         // optional logging hook
}

impl SettingsStore {
    pub fn builder(version: &'static str) -> SettingsStoreBuilder {
        SettingsStoreBuilder::new(version)
    }

    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }

    pub fn schema_version(&self) -> &Version {
        &self.target_schema_version
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

        let existing_delta = {
            let deltas = self.deltas.read().unwrap();
            deltas.get(section).cloned()
        };
        let file_version_snapshot = self.loaded_file_schema_version.read().unwrap().clone();

        let SectionMergeOutcome {
            merged_value,
            delta_value,
            delta_changed,
            migrated,
            pruned_unknown_fields,
        } = merge_for_settings::<T>(
            &default_map,
            existing_delta.as_ref(),
            file_version_snapshot.as_ref(),
            &self.target_schema_version,
        )?;

        self.defaults
            .write()
            .unwrap()
            .insert(section, default_map.clone());
        self.section_runtimes
            .write()
            .unwrap()
            .insert(section, SectionRuntime::new::<T>());
        self.values
            .write()
            .unwrap()
            .insert(section, merged_value.clone());

        if delta_changed {
            let mut deltas = self.deltas.write().unwrap();
            match delta_value.clone() {
                Some(value) => {
                    deltas.insert(section.to_string(), value);
                }
                None => {
                    deltas.remove(section);
                }
            }
        }

        let mut persist_needed = delta_changed;

        if !pruned_unknown_fields.is_empty() {
            let joined = pruned_unknown_fields.join(", ");
            self.log(&format!(
                "removed unknown fields from settings section '{section}': {joined}"
            ));
            persist_needed = true;
        }

        if self.file_path.exists() {
            let deltas_guard = self.deltas.read().unwrap();
            if deltas_guard.is_empty() {
                persist_needed = true;
            }
        }

        if migrated {
            let from_display = file_version_snapshot
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unversioned".to_string());
            self.log(&format!(
                "migrated settings section '{section}' from schema {from_display} to {}",
                self.target_schema_version
            ));
            persist_needed = true;
        }

        let version_changed = self.ensure_file_version();
        if version_changed {
            persist_needed = true;
        }

        if persist_needed {
            self.persist_deltas()?;
        }

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

        let version_changed = self.ensure_file_version();

        self.persist_deltas()?;
        if version_changed {
            self.log(&format!(
                "updated settings schema version metadata to {}",
                self.target_schema_version
            ));
        }
        Ok(())
    }

    /// Reload deltas from disk and re-merge all registered sections.
    pub fn reload(&self) -> Result<(), SettingsError> {
        let content = if self.file_path.exists() {
            fs::read_to_string(&self.file_path)?
        } else {
            String::new()
        };

        let raw_map: JsonMap<String, JsonValue> = if content.trim().is_empty() {
            JsonMap::new()
        } else {
            parse_json_with_comments(&content)
                .map_err(|_| SettingsError::Invalid("reload parse settings file"))?
        };

        let (mut delta_map, file_version_snapshot) = split_meta(raw_map)?;

        if let Some(ref version) = file_version_snapshot {
            if version > &self.target_schema_version {
                return Err(SettingsError::Invalid("settings file targets newer schema"));
            }
        }

        {
            let mut version_guard = self.file_schema_version.write().unwrap();
            *version_guard = file_version_snapshot.clone();
        }

        {
            let mut loaded_guard = self.loaded_file_schema_version.write().unwrap();
            *loaded_guard = file_version_snapshot.clone();
        }

        let defaults_snapshot = {
            let defs = self.defaults.read().unwrap();
            defs.clone()
        };
        let runtimes_snapshot = {
            let runtimes = self.section_runtimes.read().unwrap();
            runtimes.clone()
        };

        let mut values_updates: Vec<(&'static str, JsonValue)> = Vec::new();
        let mut persist_needed = false;

        for (section, default_map) in defaults_snapshot.iter() {
            let runtime = match runtimes_snapshot.get(section) {
                Some(rt) => rt,
                None => continue,
            };

            let existing_delta = delta_map.get(*section).cloned();

            let SectionMergeOutcome {
                merged_value,
                delta_value,
                delta_changed,
                migrated,
                pruned_unknown_fields,
            } = (runtime.merge)(
                default_map,
                existing_delta.as_ref(),
                file_version_snapshot.as_ref(),
                &self.target_schema_version,
            )?;

            values_updates.push((*section, merged_value));

            if delta_changed {
                match delta_value {
                    Some(value) => {
                        delta_map.insert((*section).to_string(), value);
                    }
                    None => {
                        delta_map.remove(*section);
                    }
                }
                persist_needed = true;
            }

            if migrated {
                let from_display = file_version_snapshot
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unversioned".to_string());
                self.log(&format!(
                    "migrated settings section '{section}' from schema {from_display} to {}",
                    self.target_schema_version
                ));
                persist_needed = true;
            }

            if !pruned_unknown_fields.is_empty() {
                let joined = pruned_unknown_fields.join(", ");
                self.log(&format!(
                    "removed unknown fields from settings section '{section}': {joined}"
                ));
                persist_needed = true;
            }
        }

        {
            let mut values_guard = self.values.write().unwrap();
            for (section, value) in values_updates {
                values_guard.insert(section, value);
            }
        }

        let delta_map_is_empty = delta_map.is_empty();

        {
            let mut deltas_guard = self.deltas.write().unwrap();
            *deltas_guard = delta_map;
        }

        if delta_map_is_empty && self.file_path.exists() {
            persist_needed = true;
        }

        let version_changed = self.ensure_file_version();
        if version_changed {
            persist_needed = true;
        }

        if persist_needed {
            self.persist_deltas()?;
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

        drop(deltas_guard);

        if clean.is_empty() {
            self.remove_settings_file_if_exists()?;
            {
                let mut guard = self.file_schema_version.write().unwrap();
                *guard = None;
            }
            {
                let mut guard = self.loaded_file_schema_version.write().unwrap();
                *guard = None;
            }
            return Ok(());
        }

        let schema_version = self
            .file_schema_version
            .read()
            .unwrap()
            .clone()
            .unwrap_or_else(|| self.target_schema_version.clone());

        clean.insert(META_KEY.to_string(), encode_meta(&schema_version));

        self.write_deltas_document(&clean)
    }

    fn write_deltas_document(
        &self,
        clean: &JsonMap<String, JsonValue>,
    ) -> Result<(), SettingsError> {
        const TAB_SIZE: usize = 4;

        let buffer = if clean.is_empty() {
            "{}".to_string()
        } else {
            let mut snapshot = clean.clone();
            let value = JsonValue::Object(std::mem::take(&mut snapshot));
            to_pretty_json(&value, TAB_SIZE, 0)
        };

        let tmp = self.file_path.with_extension("tmp");
        fs::write(&tmp, buffer)?;
        fs::rename(&tmp, &self.file_path)?;
        Ok(())
    }

    fn remove_settings_file_if_exists(&self) -> Result<(), SettingsError> {
        match fs::remove_file(&self.file_path) {
            Ok(()) => {
                self.log("removed empty settings delta file");
                Ok(())
            }
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err.into()),
        }
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
                    let removed = prune_unknown_fields(default_map, delta_map);
                    if !removed.is_empty() && delta_map.is_empty() {
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
    fn ensure_file_version(&self) -> bool {
        let target = self.target_schema_version.clone();
        let mut guard = self.file_schema_version.write().unwrap();
        match guard.as_ref() {
            Some(existing) if existing == &target => false,
            _ => {
                *guard = Some(target);
                true
            }
        }
    }

    fn log(&self, message: &str) {
        if let Some(logger) = self.logger.read().unwrap().as_ref() {
            logger(message);
            return;
        }

        if tracing::dispatcher::has_been_set() {
            tracing::info!(target: "settings::store", "{message}");
        } else {
            eprintln!("{message}");
        }
    }
}

// (file watcher functionality removed; doc comment left intentionally cleared)

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;
    use serde::{Deserialize, Serialize};
    use serde_json::{json, Number as JsonNumber, Value as JsonValue};
    use std::{cell::RefCell, fs, thread_local};
    use tempfile::tempdir;

    thread_local! {
        static MIGRATION_VERSIONS: RefCell<Vec<Option<String>>> = const { RefCell::new(Vec::new()) };
    }

    fn reset_migration_versions() {
        MIGRATION_VERSIONS.with(|versions| versions.borrow_mut().clear());
    }

    fn push_migration_version(file_version: Option<&Version>) {
        let entry = file_version.map(|ver| ver.to_string());
        MIGRATION_VERSIONS.with(|versions| versions.borrow_mut().push(entry));
    }

    fn collected_migration_versions() -> Vec<Option<String>> {
        MIGRATION_VERSIONS.with(|versions| versions.borrow().clone())
    }

    #[derive(Clone, Serialize, Deserialize)]
    struct ExampleSettings {
        new_field: u32,
    }

    impl Default for ExampleSettings {
        fn default() -> Self {
            Self { new_field: 0 }
        }
    }

    impl Settings for ExampleSettings {
        const SECTION: &'static str = "example";

        fn migrate(
            file_version: Option<&Version>,
            target_version: &Version,
            data: JsonValue,
        ) -> Result<(JsonValue, bool), SettingsError> {
            push_migration_version(file_version);
            let mut map = match data {
                JsonValue::Object(map) => map,
                _ => return Err(SettingsError::Invalid("migration expects object")),
            };

            let needs_upgrade = file_version.map(|ver| ver < target_version).unwrap_or(true);

            if needs_upgrade {
                if let Some(raw_value) = map.remove("old_field") {
                    let number = raw_value
                        .as_u64()
                        .ok_or(SettingsError::Invalid("old_field not number"))?;
                    map.insert(
                        "new_field".to_string(),
                        JsonValue::Number(JsonNumber::from(number)),
                    );
                    return Ok((JsonValue::Object(map), true));
                }
            }

            Ok((JsonValue::Object(map), false))
        }
    }

    fn schema_v1_document() -> String {
        json!({
            "__meta": {
                "version": "0.1.0"
            },
            "example": {
                "old_field": 7
            }
        })
        .to_string()
    }

    #[test]
    fn migrates_schema_on_register() -> Result<(), Box<dyn std::error::Error>> {
        reset_migration_versions();
        let dir = tempdir()?;
        let path = dir.path().join("settings.json");
        fs::write(&path, schema_v1_document())?;

        let store = SettingsStore::builder("0.2.0")
            .with_settings_file(&path)
            .build()?;
        store.register::<ExampleSettings>()?;
        let expected_version = store.schema_version().to_string();

        let cfg = store.get::<ExampleSettings>()?;
        assert_eq!(cfg.new_field, 7);

        let recorded = collected_migration_versions();
        assert!(!recorded.is_empty());
        assert!(recorded
            .iter()
            .all(|entry| entry.as_deref() == Some("0.1.0")));

        let doc: JsonValue = serde_json::from_str(&fs::read_to_string(&path)?)?;
        assert_eq!(
            doc["__meta"]["version"].as_str(),
            Some(expected_version.as_str())
        );
        let example_obj = doc["example"].as_object().unwrap();
        assert!(example_obj.get("old_field").is_none());
        assert_eq!(
            example_obj.get("new_field").and_then(JsonValue::as_u64),
            Some(7)
        );

        Ok(())
    }

    #[test]
    fn reload_migrates_after_external_downgrade() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let path = dir.path().join("settings.json");
        fs::write(&path, schema_v1_document())?;

        let store = SettingsStore::builder("0.2.0")
            .with_settings_file(&path)
            .build()?;
        store.register::<ExampleSettings>()?;
        let expected_version = store.schema_version().to_string();

        // Simulate external downgrade back to schema v1.
        fs::write(&path, schema_v1_document())?;
        store.reload()?;

        let cfg = store.get::<ExampleSettings>()?;
        assert_eq!(cfg.new_field, 7);

        let doc: JsonValue = serde_json::from_str(&fs::read_to_string(&path)?)?;
        assert_eq!(
            doc["__meta"]["version"].as_str(),
            Some(expected_version.as_str())
        );
        let example_obj = doc["example"].as_object().unwrap();
        assert!(example_obj.get("old_field").is_none());
        assert_eq!(
            example_obj.get("new_field").and_then(JsonValue::as_u64),
            Some(7)
        );

        Ok(())
    }

    #[derive(Clone, Serialize, Deserialize)]
    struct SectionOne {
        value: u32,
    }

    impl Default for SectionOne {
        fn default() -> Self {
            Self { value: 0 }
        }
    }

    impl Settings for SectionOne {
        const SECTION: &'static str = "one";

        fn migrate(
            file_version: Option<&Version>,
            _target_version: &Version,
            data: JsonValue,
        ) -> Result<(JsonValue, bool), SettingsError> {
            push_migration_version(file_version);
            match data {
                JsonValue::Object(map) => Ok((JsonValue::Object(map), false)),
                _ => Err(SettingsError::Invalid("section one expects object")),
            }
        }
    }

    #[derive(Clone, Serialize, Deserialize)]
    struct SectionTwo {
        value: u32,
    }

    impl Default for SectionTwo {
        fn default() -> Self {
            Self { value: 0 }
        }
    }

    impl Settings for SectionTwo {
        const SECTION: &'static str = "two";

        fn migrate(
            file_version: Option<&Version>,
            _target_version: &Version,
            data: JsonValue,
        ) -> Result<(JsonValue, bool), SettingsError> {
            push_migration_version(file_version);
            match data {
                JsonValue::Object(map) => Ok((JsonValue::Object(map), false)),
                _ => Err(SettingsError::Invalid("section two expects object")),
            }
        }
    }

    #[test]
    fn register_uses_loaded_version_for_each_section() -> Result<(), Box<dyn std::error::Error>> {
        reset_migration_versions();

        let dir = tempdir()?;
        let path = dir.path().join("settings.json");
        fs::write(
            &path,
            json!({
                "__meta": { "version": "0.1.0" },
                "one": { "value": 1 },
                "two": { "value": 2 }
            })
            .to_string(),
        )?;

        let store = SettingsStore::builder("0.2.0")
            .with_settings_file(&path)
            .build()?;

        store.register::<SectionOne>()?;
        store.register::<SectionTwo>()?;

        let recorded = collected_migration_versions();
        assert!(recorded.len() >= 2);
        assert!(recorded
            .iter()
            .all(|entry| entry.as_deref() == Some("0.1.0")));

        let doc: JsonValue = serde_json::from_str(&fs::read_to_string(&path)?)?;
        assert_eq!(doc["__meta"]["version"].as_str(), Some("0.2.0"));

        Ok(())
    }

    #[test]
    fn register_prunes_unknown_fields() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let path = dir.path().join("settings.json");
        fs::write(
            &path,
            json!({
                "__meta": { "version": "0.1.0" },
                "one": { "value": 7, "typo_key": true }
            })
            .to_string(),
        )?;

        let store = SettingsStore::builder("0.2.0")
            .with_settings_file(&path)
            .build()?;
        store.register::<SectionOne>()?;

        let doc: JsonValue = serde_json::from_str(&fs::read_to_string(&path)?)?;
        let section = doc["one"].as_object().unwrap();
        assert!(!section.contains_key("typo_key"));
        assert_eq!(section.get("value").and_then(JsonValue::as_u64), Some(7));

        Ok(())
    }

    #[test]
    fn reload_prunes_unknown_fields() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let path = dir.path().join("settings.json");

        let store = SettingsStore::builder("0.2.0")
            .with_settings_file(&path)
            .build()?;
        store.register::<SectionOne>()?;

        // Introduce typo directly in the file to simulate external edit.
        fs::write(
            &path,
            json!({
                "__meta": { "version": "0.2.0" },
                "one": { "value": 5, "typo_key": true }
            })
            .to_string(),
        )?;

        store.reload()?;

        let doc: JsonValue = serde_json::from_str(&fs::read_to_string(&path)?)?;
        let section = doc["one"].as_object().unwrap();
        assert!(!section.contains_key("typo_key"));
        assert_eq!(section.get("value").and_then(JsonValue::as_u64), Some(5));

        Ok(())
    }

    #[test]
    fn register_removes_file_when_only_meta_remains() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let path = dir.path().join("settings.json");
        fs::write(
            &path,
            json!({
                "__meta": { "version": "0.1.0" },
                "one": { "typo_only": true }
            })
            .to_string(),
        )?;

        let store = SettingsStore::builder("0.2.0")
            .with_settings_file(&path)
            .build()?;
        store.register::<SectionOne>()?;

        assert!(!path.exists());

        Ok(())
    }

    #[test]
    fn reload_removes_file_when_only_meta_remains() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let path = dir.path().join("settings.json");

        let store = SettingsStore::builder("0.2.0")
            .with_settings_file(&path)
            .build()?;
        store.register::<SectionOne>()?;

        fs::write(
            &path,
            json!({
                "__meta": { "version": "0.2.0" }
            })
            .to_string(),
        )?;

        assert!(path.exists());
        store.reload()?;

        assert!(!path.exists());

        Ok(())
    }
}
