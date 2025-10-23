//! Keymap store for managing default and user-defined descriptors with persistence.
//!
//! The store mirrors the behaviour of the settings system: defaults are authored
//! in code, user overrides are deserialized from JSON. The merged specification
//! can then be converted to the legacy keyboard-based keymap as well as to
//! `bevy_enhanced_input` entities.

use crate::binding::{ActionId, KeyBindingMetaIndex};
use crate::keymap::{Keymap, KeymapVersion};
use crate::keystroke::Keystroke;
use crate::spec::{BindingDescriptor, KeymapSpec};
use anyhow::{Context as _, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Root structure for keymap JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeymapFile {
    /// Schema version for migrations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,
    /// The persisted specification.
    #[serde(default)]
    pub spec: KeymapSpec,
}

impl Default for KeymapFile {
    fn default() -> Self {
        Self {
            schema_version: None,
            spec: KeymapSpec::default(),
        }
    }
}

/// Builder for creating a [`KeymapStore`].
pub struct KeymapStoreBuilder {
    user_keymap_path: Option<PathBuf>,
    default_spec: KeymapSpec,
}

impl KeymapStoreBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            user_keymap_path: None,
            default_spec: KeymapSpec::default(),
        }
    }

    /// Set the path for user keymaps.
    pub fn with_user_keymap_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.user_keymap_path = Some(path.into());
        self
    }

    /// Replace the default specification entirely.
    pub fn with_default_spec(mut self, spec: KeymapSpec) -> Self {
        self.default_spec = spec;
        self
    }

    /// Add a default action descriptor.
    pub fn add_default_action(mut self, action: crate::spec::ActionDescriptor) -> Self {
        self.default_spec.actions.push(action);
        self
    }

    /// Add a default context descriptor.
    pub fn add_default_context(mut self, context: crate::spec::ContextDescriptor) -> Self {
        self.default_spec.contexts.push(context);
        self
    }

    /// Add a default binding descriptor.
    pub fn add_default_binding(mut self, mut binding: BindingDescriptor) -> Self {
        ensure_meta(&mut binding, KeyBindingMetaIndex::DEFAULT);
        self.default_spec.bindings.push(binding);
        self
    }

    /// Build the [`KeymapStore`].
    pub fn build(self) -> Result<KeymapStore> {
        let user_keymap_path = self
            .user_keymap_path
            .unwrap_or_else(|| PathBuf::from("keymap.json"));

        if let Some(parent) = user_keymap_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let store = KeymapStore {
            user_keymap_path,
            inner: Mutex::new(StoreInner::new(self.default_spec)),
        };

        // Initial rebuild to populate merged spec and keymap.
        {
            let mut inner = store.inner.lock().unwrap();
            rebuild_keymap_internal(&mut inner)?;
        }

        Ok(store)
    }
}

impl Default for KeymapStoreBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Inner state of the [`KeymapStore`], protected by a mutex.
struct StoreInner {
    default_spec: KeymapSpec,
    user_spec: KeymapSpec,
    merged_spec: KeymapSpec,
    keymap: Keymap,
    version: KeymapVersion,
}

impl StoreInner {
    fn new(default_spec: KeymapSpec) -> Self {
        Self {
            default_spec,
            user_spec: KeymapSpec::default(),
            merged_spec: KeymapSpec::default(),
            keymap: Keymap::new(),
            version: KeymapVersion::default(),
        }
    }
}

/// Store for managing keymaps with persistence.
pub struct KeymapStore {
    user_keymap_path: PathBuf,
    inner: Mutex<StoreInner>,
}

impl KeymapStore {
    /// Create a new builder.
    pub fn builder() -> KeymapStoreBuilder {
        KeymapStoreBuilder::new()
    }

    /// Get the current keymap version.
    pub fn version(&self) -> KeymapVersion {
        self.inner.lock().unwrap().version
    }

    /// Access the merged specification.
    pub fn merged_spec(&self) -> KeymapSpec {
        self.inner.lock().unwrap().merged_spec.clone()
    }

    /// Append default descriptors and rebuild.
    pub fn add_default_bindings(&self, mut bindings: Vec<BindingDescriptor>) -> Result<()> {
        for binding in &mut bindings {
            ensure_meta(binding, KeyBindingMetaIndex::DEFAULT);
        }

        let mut inner = self.inner.lock().unwrap();
        inner.default_spec.bindings.extend(bindings);
        rebuild_keymap_internal(&mut inner)
    }

    /// Append default actions and rebuild.
    pub fn add_default_actions(
        &self,
        actions: Vec<crate::spec::ActionDescriptor>,
    ) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.default_spec.actions.extend(actions);
        rebuild_keymap_internal(&mut inner)
    }

    /// Append default contexts and rebuild.
    pub fn add_default_contexts(
        &self,
        contexts: Vec<crate::spec::ContextDescriptor>,
    ) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.default_spec.contexts.extend(contexts);
        rebuild_keymap_internal(&mut inner)
    }

    /// Load user bindings from disk and rebuild.
    pub fn load_user_bindings(&self) -> Result<()> {
        if !self.user_keymap_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.user_keymap_path)
            .context("Failed to read user keymap file")?;

        let file: KeymapFile =
            serde_json::from_str(&content).context("Failed to parse user keymap JSON")?;

        let mut inner = self.inner.lock().unwrap();
        inner.user_spec = file.spec;

        // Ensure metadata for user bindings.
        for binding in &mut inner.user_spec.bindings {
            ensure_meta(binding, KeyBindingMetaIndex::USER);
        }

        rebuild_keymap_internal(&mut inner)
    }

    /// Persist the current user specification to disk.
    pub fn save_user_bindings(&self) -> Result<()> {
        let inner = self.inner.lock().unwrap();
        let file = KeymapFile {
            schema_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            spec: inner.user_spec.clone(),
        };

        let json = serde_json::to_string_pretty(&file)?;
        fs::write(&self.user_keymap_path, json)?;
        Ok(())
    }

    /// Reload the user bindings from disk.
    pub fn reload(&self) -> Result<()> {
        self.load_user_bindings()
    }

    /// Add a user binding descriptor.
    pub fn add_user_binding(&self, mut binding: BindingDescriptor) -> Result<()> {
        ensure_meta(&mut binding, KeyBindingMetaIndex::USER);
        let mut inner = self.inner.lock().unwrap();
        inner.user_spec.bindings.push(binding);
        rebuild_keymap_internal(&mut inner)
    }

    /// Remove user bindings that match the given keyboard sequence.
    pub fn remove_user_binding(&self, sequence: &[Keystroke]) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.user_spec.bindings.retain(|binding| {
            binding
                .keyboard_sequence()
                .map(|seq| seq != sequence)
                .unwrap_or(true)
        });
        rebuild_keymap_internal(&mut inner)
    }

    /// Execute a function with read access to the merged keymap.
    pub fn with_keymap<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Keymap) -> R,
    {
        let inner = self.inner.lock().unwrap();
        f(&inner.keymap)
    }

    /// Access the generated keymap spec directly.
    pub fn with_spec<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&KeymapSpec) -> R,
    {
        let inner = self.inner.lock().unwrap();
        f(&inner.merged_spec)
    }

    /// Get the path to the user keymap file.
    pub fn user_keymap_path(&self) -> &Path {
        &self.user_keymap_path
    }
}

fn ensure_meta(binding: &mut BindingDescriptor, default: KeyBindingMetaIndex) {
    if binding.meta.is_none() {
        binding.meta = Some(default);
    }
}

fn rebuild_keymap_internal(inner: &mut StoreInner) -> Result<()> {
    inner.merged_spec = merge_specs(&inner.default_spec, &inner.user_spec);

    let mut bindings = Vec::new();
    for descriptor in &inner.merged_spec.bindings {
        if let Some(binding) = descriptor.to_key_binding()? {
            bindings.push(binding);
        }
    }

    inner.keymap = Keymap::with_bindings(bindings);
    inner.version = inner.keymap.version();
    Ok(())
}

fn merge_specs(default_spec: &KeymapSpec, user_spec: &KeymapSpec) -> KeymapSpec {
    let mut merged = KeymapSpec::default();

    // Merge actions by id (user overrides replace defaults).
    let mut actions: IndexMap<ActionId, _> = IndexMap::new();
    for action in &default_spec.actions {
        actions.insert(action.id.clone(), action.clone());
    }
    for action in &user_spec.actions {
        actions.insert(action.id.clone(), action.clone());
    }
    merged.actions = actions.into_values().collect();

    // Merge contexts by id (user overrides replace defaults).
    let mut contexts: IndexMap<crate::binding::ContextId, _> = IndexMap::new();
    for context in &default_spec.contexts {
        contexts.insert(context.id.clone(), context.clone());
    }
    for context in &user_spec.contexts {
        contexts.insert(context.id.clone(), context.clone());
    }
    merged.contexts = contexts.into_values().collect();

    // Bindings: defaults first, then user overrides (higher precedence).
    merged.bindings.extend(default_spec.bindings.clone());
    merged.bindings.extend(user_spec.bindings.clone());

    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::ActionId;
    use crate::keystroke::Keystroke;
    use crate::spec::{ActionDescriptor, BindingInputDescriptor};
    use tempfile::TempDir;

    fn sample_binding(sequence: &str, action: &str) -> BindingDescriptor {
        BindingDescriptor {
            action_id: Some(ActionId::from(action)),
            context_id: None,
            predicate: None,
            meta: None,
            modifiers: Vec::new(),
            conditions: Vec::new(),
            settings: None,
            input: Some(BindingInputDescriptor::keyboard(
                crate::parse_keystroke_sequence(sequence).unwrap(),
            )),
        }
    }

    #[test]
    fn builder_sets_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("keymap.json");

        let store = KeymapStore::builder()
            .with_user_keymap_path(path.clone())
            .add_default_binding(sample_binding("cmd-s", "Save"))
            .build()
            .unwrap();

        store.with_keymap(|keymap| {
            assert_eq!(keymap.bindings().count(), 1);
        });

        assert_eq!(store.user_keymap_path(), path.as_path());
    }

    #[test]
    fn load_and_save_user_bindings() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("keymap.json");

        let store = KeymapStore::builder()
            .with_user_keymap_path(path.clone())
            .build()
            .unwrap();

        // Add a user binding and save.
        store
            .add_user_binding(sample_binding("cmd-s", "Save"))
            .unwrap();
        store.save_user_bindings().unwrap();
        assert!(path.exists());

        // Create a new store and load.
        let store2 = KeymapStore::builder()
            .with_user_keymap_path(path)
            .build()
            .unwrap();
        store2.load_user_bindings().unwrap();

        store2.with_keymap(|keymap| {
            assert_eq!(keymap.bindings().count(), 1);
        });
    }

    #[test]
    fn user_overrides_precedence() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("keymap.json");

        let store = KeymapStore::builder()
            .with_user_keymap_path(path)
            .add_default_binding(sample_binding("cmd-s", "SaveDefault"))
            .build()
            .unwrap();

        store
            .add_user_binding(sample_binding("cmd-s", "SaveOverride"))
            .unwrap();

        store.with_keymap(|keymap| {
            let (bindings, _) =
                keymap.bindings_for_input(&[Keystroke::parse("cmd-s").unwrap()], &[]);
            assert_eq!(bindings.len(), 2);
            assert_eq!(bindings[0].action_id().unwrap().as_str(), "SaveOverride");
            assert_eq!(bindings[1].action_id().unwrap().as_str(), "SaveDefault");
        });
    }

    #[test]
    fn remove_user_binding_by_sequence() {
        let store = KeymapStore::builder()
            .add_default_binding(sample_binding("cmd-s", "Save"))
            .build()
            .unwrap();

        store
            .add_user_binding(sample_binding("cmd-s", "SaveOverride"))
            .unwrap();
        store
            .remove_user_binding(&[Keystroke::parse("cmd-s").unwrap()])
            .unwrap();

        store.with_keymap(|keymap| {
            let (bindings, _) =
                keymap.bindings_for_input(&[Keystroke::parse("cmd-s").unwrap()], &[]);
            assert_eq!(bindings.len(), 1);
            assert_eq!(bindings[0].action_id().unwrap().as_str(), "Save");
        });
    }

    #[test]
    fn merge_actions_and_contexts() {
        let builder = KeymapStore::builder()
            .add_default_action(ActionDescriptor {
                id: ActionId::from("default"),
                output: Some("bool".into()),
                modifiers: Vec::new(),
                conditions: Vec::new(),
                settings: None,
            })
            .add_default_binding(sample_binding("cmd-s", "default"));

        let store = builder.build().unwrap();
        store
            .add_user_binding(sample_binding("cmd-o", "user"))
            .unwrap();

        let spec = store.merged_spec();
        assert_eq!(spec.actions.len(), 1);
        assert_eq!(spec.bindings.len(), 2);
    }
}
