use crate::errors::SettingError;
use crate::key_path::KeyPath;
use crate::source::{SettingSource, SourceKind};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use toml_edit::{Item, Value};

/// Metadata for a setting key or prefix.
#[derive(Clone, Debug, Default)]
pub struct SettingMeta {
    pub requires_restart: bool,
    pub immutable: bool,
}

/// SettingStore: holds ordered sources (low -> high precedence).
pub struct SettingStore {
    sources: RwLock<Vec<Arc<RwLock<Box<dyn SettingSource>>>>>,
    /// metadata registry: key prefix -> meta
    pub metadata: RwLock<HashMap<String, SettingMeta>>,
}

impl SettingStore {
    pub fn new(sources: Vec<Box<dyn SettingSource>>) -> Self {
        let wrapped = sources
            .into_iter()
            .map(|s| Arc::new(RwLock::new(s)))
            .collect();
        Self {
            sources: RwLock::new(wrapped),
            metadata: RwLock::new(HashMap::new()),
        }
    }

    /// Create a `SettingStore` with the standard layout of sources using the
    /// workspace `paths` crate to resolve locations.
    ///
    /// Layering (later wins):
    /// Defaults (in-memory, precedence 0)
    /// Global (settings/global.toml, precedence 10, read-only)
    /// User (settings/user.toml, precedence 20, writable) -- optional
    /// World (saves/<id>/world.toml, precedence 30, writable) -- optional
    /// Server (server/server.toml, precedence 40, writable)
    pub fn standard(include_user: bool, world_id: Option<&str>) -> Result<Self, SettingError> {
        use crate::file_toml_source::FileTomlSource;
        use crate::in_memory::InMemoryDefaults;

        // Defaults (empty by default)
        let defaults = InMemoryDefaults::empty(0);
        let mut sources: Vec<Box<dyn SettingSource>> = vec![Box::new(defaults)];

        // Global (read-only)
        if let Some(p) =
            crate::simple_path_resolver::SimplePathResolver::path_for(SourceKind::Global, None)
        {
            let global = FileTomlSource::open(p, SourceKind::Global, 10, false)?;
            sources.push(Box::new(global));
        }

        // Optional user (writable)
        if include_user {
            if let Some(p) =
                crate::simple_path_resolver::SimplePathResolver::path_for(SourceKind::User, None)
            {
                let user = FileTomlSource::open(p, SourceKind::User, 20, true)?;
                sources.push(Box::new(user));
            }
        }

        // Optional world/save-specific settings
        if let Some(id) = world_id {
            if let Some(p) = crate::simple_path_resolver::SimplePathResolver::path_for(
                SourceKind::World,
                Some(id),
            ) {
                let world = FileTomlSource::open(p, SourceKind::World, 30, true)?;
                sources.push(Box::new(world));
            }
        }

        // Server (writable)
        if let Some(p) =
            crate::simple_path_resolver::SimplePathResolver::path_for(SourceKind::Server, None)
        {
            let server = FileTomlSource::open(p, SourceKind::Server, 40, true)?;
            sources.push(Box::new(server));
        }

        Ok(SettingStore::new(sources))
    }

    pub fn register_meta(&self, key_prefix: &str, meta: SettingMeta) {
        self.metadata
            .write()
            .unwrap()
            .insert(key_prefix.to_string(), meta);
    }

    fn find_target(
        &self,
        target: Option<SourceKind>,
    ) -> Result<Arc<RwLock<Box<dyn SettingSource>>>, SettingError> {
        let sources = self.sources.read().unwrap();
        if let Some(tk) = target {
            for s in sources.iter().rev() {
                let sref = s.read().unwrap();
                if sref.kind() == tk {
                    return Ok(Arc::clone(s));
                }
            }
            return Err(SettingError::InvalidTarget(Some(tk)));
        }
        // implicit: highest writable
        for s in sources.iter().rev() {
            let sref = s.read().unwrap();
            if sref.is_writable() {
                return Ok(Arc::clone(s));
            }
        }
        Err(SettingError::InvalidTarget(None))
    }

    /// Merge-read: get effective value by keypath
    pub fn get_effective(&self, key_path: &KeyPath) -> Result<Option<Item>, SettingError> {
        let sources = self.sources.read().unwrap();
        let mut result: Option<Item> = None;
        for s in sources.iter() {
            let sref = s.read().unwrap();
            if let Some(item) = sref.get(key_path)? {
                match &mut result {
                    None => result = Some(item),
                    Some(existing) => {
                        // merge tables if both are tables
                        if existing.is_table() && item.is_table() {
                            // simple merge: extend existing table with new keys
                            if let Some(et) = existing.as_table_mut() {
                                if let Some(nt) = item.as_table() {
                                    for (k, v) in nt.iter() {
                                        et.insert(k, v.clone());
                                    }
                                }
                            }
                        } else {
                            // overwrite with higher precedence scalar or table
                            *existing = item;
                        }
                    }
                }
            }
        }
        Ok(result)
    }

    /// get typed struct from merged view
    pub fn get_effective_struct<T: serde::de::DeserializeOwned>(&self) -> Result<T, SettingError> {
        // Build a combined toml::Value from merged documents.
        // We'll create an owned empty Document<String> and merge into it, then
        // convert to a TOML string and deserialize via `toml`.
        let sources = self.sources.read().unwrap();

        // Merge all source root tables into a toml::map::Map<String, toml::Value>.
        // We operate on toml::Value maps directly to avoid mutating toml_edit::Document internals.
        let mut merged_map: toml::map::Map<String, toml::Value> = toml::map::Map::new();

        // Recursive merge: later-src keys overwrite unless both are tables -> merge recursively.
        fn merge_tables(
            dst: &mut toml::map::Map<String, toml::Value>,
            src: &toml::map::Map<String, toml::Value>,
        ) {
            for (k, v) in src.iter() {
                match (dst.get_mut(k), v) {
                    (Some(toml::Value::Table(dst_table)), toml::Value::Table(src_table)) => {
                        merge_tables(dst_table, src_table);
                    }
                    _ => {
                        dst.insert(k.clone(), v.clone());
                    }
                }
            }
        }

        for s in sources.iter() {
            let sref = s.read().unwrap();
            // ask each source for its root table (empty key path)
            if let Ok(Some(item)) = sref.get(&KeyPath::new(Vec::<String>::new())) {
                if item.is_table() {
                    // convert item (toml_edit::Item) to toml::Value via string round-trip
                    let item_str = item.to_string();
                    if let Ok(tval) = toml::from_str::<toml::Value>(&item_str) {
                        if let toml::Value::Table(tbl) = tval {
                            merge_tables(&mut merged_map, &tbl);
                        }
                    }
                }
            }
        }

        // Serialize merged_map into a TOML string for serde deserialization
        let toml_str = toml::to_string(&toml::Value::Table(merged_map))
            .map_err(|e| SettingError::Other(e.to_string()))?;
        let v: toml::Value =
            toml::from_str(&toml_str).map_err(|e| SettingError::Other(e.to_string()))?;
        let t: T = v
            .try_into()
            .map_err(|e| SettingError::Other(e.to_string()))?;
        Ok(t)
    }

    /// Set a keypath to a value in either explicit target or highest-writable.
    pub fn set(
        &self,
        key_path: &KeyPath,
        value: Value,
        target: Option<SourceKind>,
    ) -> Result<(), SettingError> {
        // check metadata immutability
        for (k, m) in self.metadata.read().unwrap().iter() {
            if key_path.to_string().starts_with(k) && m.immutable {
                return Err(SettingError::ImmutableKey(k.clone()));
            }
        }

        let target_arc = self.find_target(target)?;
        let mut target_lock = target_arc.write().unwrap();
        target_lock.set(key_path, Item::Value(value))?;
        target_lock.persist()?;
        Ok(())
    }
}
