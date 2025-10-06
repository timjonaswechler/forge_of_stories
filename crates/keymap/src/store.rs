//! Keymap store for managing default and user key bindings with persistence.
//!
//! This module provides a store that manages keymaps similar to the Settings system,
//! with support for:
//! - Default (built-in) bindings
//! - User overrides loaded from JSON files
//! - Merge logic with proper precedence
//! - Hot-reloading of user keymaps

use crate::action::{Action, NoAction};
use crate::binding::{KeyBinding, KeyBindingMetaIndex};
use crate::context::KeyBindingContextPredicate;
use crate::keymap::{Keymap, KeymapVersion};
use crate::keystroke::{Keystroke, parse_keystroke_sequence};
use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// A keymap file section with context and bindings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeymapSection {
    /// Optional context predicate for this section.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// Whether to use key equivalents (platform-specific).
    #[serde(default)]
    pub use_key_equivalents: bool,

    /// Map of keystroke sequences to action names.
    /// Value can be:
    /// - String for action name (e.g., "editor::Save")
    /// - null for disabling a binding
    pub bindings: HashMap<String, Option<String>>,
}

/// Root structure for keymap JSON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeymapFile {
    /// Schema version for migrations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,

    /// List of keymap sections.
    #[serde(default)]
    pub sections: Vec<KeymapSection>,
}

/// Registry for mapping action names to action constructors.
type ActionRegistry = HashMap<String, Box<dyn Fn() -> Box<dyn Action> + Send + Sync>>;

/// Builder for creating a KeymapStore.
pub struct KeymapStoreBuilder {
    user_keymap_path: Option<PathBuf>,
    action_registry: Arc<Mutex<ActionRegistry>>,
    default_bindings: Vec<KeyBinding>,
}

impl KeymapStoreBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            user_keymap_path: None,
            action_registry: Arc::new(Mutex::new(HashMap::new())),
            default_bindings: Vec::new(),
        }
    }

    /// Set the path for user keymaps.
    pub fn with_user_keymap_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.user_keymap_path = Some(path.into());
        self
    }

    /// Register an action with a constructor.
    pub fn register_action<F>(self, name: impl Into<String>, constructor: F) -> Self
    where
        F: Fn() -> Box<dyn Action> + Send + Sync + 'static,
    {
        self.action_registry
            .lock()
            .unwrap()
            .insert(name.into(), Box::new(constructor));
        self
    }

    /// Add a default binding.
    pub fn add_default_binding(mut self, binding: KeyBinding) -> Self {
        self.default_bindings.push(binding);
        self
    }

    /// Build the KeymapStore.
    pub fn build(self) -> Result<KeymapStore> {
        let user_keymap_path = self.user_keymap_path.unwrap_or_else(|| {
            // Default to current directory + keymaps.json if no path specified
            // In practice, the application should always provide a path via PathContext
            PathBuf::from("keymaps.json")
        });

        // Create parent directory if needed
        if let Some(parent) = user_keymap_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let store = KeymapStore {
            user_keymap_path,
            inner: Mutex::new(StoreInner {
                default_bindings: Vec::new(),
                user_bindings: Vec::new(),
                keymap: Keymap::new(),
                version: KeymapVersion::default(),
            }),
            action_registry: self.action_registry,
        };

        // Add default bindings if any
        if !self.default_bindings.is_empty() {
            store.add_default_bindings(self.default_bindings);
        }

        Ok(store)
    }
}

impl Default for KeymapStoreBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Inner state of the KeymapStore, protected by a single Mutex.
struct StoreInner {
    default_bindings: Vec<KeyBinding>,
    user_bindings: Vec<KeyBinding>,
    keymap: Keymap,
    version: KeymapVersion,
}

/// Store for managing keymaps with persistence.
pub struct KeymapStore {
    user_keymap_path: PathBuf,
    inner: Mutex<StoreInner>,
    action_registry: Arc<Mutex<ActionRegistry>>,
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

    /// Add default bindings (built-in, lowest priority).
    pub fn add_default_bindings(&self, bindings: Vec<KeyBinding>) {
        let mut inner = self.inner.lock().unwrap();
        inner.default_bindings.extend(bindings);
        Self::rebuild_keymap_internal(&mut inner);
    }

    /// Load user bindings from the configured file path.
    pub fn load_user_bindings(&self) -> Result<()> {
        if !self.user_keymap_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.user_keymap_path)
            .context("Failed to read user keymap file")?;

        let keymap_file: KeymapFile =
            serde_json::from_str(&content).context("Failed to parse user keymap JSON")?;

        let bindings = self.parse_keymap_file(&keymap_file, KeyBindingMetaIndex::USER)?;

        let mut inner = self.inner.lock().unwrap();
        inner.user_bindings = bindings;
        Self::rebuild_keymap_internal(&mut inner);
        Ok(())
    }

    /// Save user bindings to the configured file path.
    pub fn save_user_bindings(&self) -> Result<()> {
        // Group bindings by context
        let sections_map: HashMap<Option<String>, KeymapSection> = {
            let inner = self.inner.lock().unwrap();
            let mut sections_map: HashMap<Option<String>, KeymapSection> = HashMap::new();

            for binding in inner.user_bindings.iter() {
                let context_str = binding.predicate().map(|p| p.to_string());

                let section =
                    sections_map
                        .entry(context_str.clone())
                        .or_insert_with(|| KeymapSection {
                            context: context_str,
                            use_key_equivalents: false,
                            bindings: HashMap::new(),
                        });

                let keystroke_str = binding
                    .keystrokes()
                    .iter()
                    .map(|k| k.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");

                let action_name = if crate::action::is_no_action(binding.action()) {
                    None
                } else {
                    Some(binding.action().name().to_string())
                };

                section.bindings.insert(keystroke_str, action_name);
            }

            sections_map
        };

        let keymap_file = KeymapFile {
            schema_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            sections: sections_map.into_values().collect(),
        };

        let json = serde_json::to_string_pretty(&keymap_file)?;
        fs::write(&self.user_keymap_path, json)?;

        Ok(())
    }

    /// Parse a keymap file into bindings.
    fn parse_keymap_file(
        &self,
        file: &KeymapFile,
        meta: KeyBindingMetaIndex,
    ) -> Result<Vec<KeyBinding>> {
        let mut bindings = Vec::new();

        for section in &file.sections {
            let predicate = section
                .context
                .as_ref()
                .map(|ctx| {
                    KeyBindingContextPredicate::parse(ctx)
                        .map(Arc::new)
                        .context("Failed to parse context predicate")
                })
                .transpose()?;

            for (keystroke_str, action_name) in &section.bindings {
                let keystrokes = parse_keystroke_sequence(keystroke_str)
                    .context("Failed to parse keystrokes")?;

                let action: Box<dyn Action> = match action_name {
                    None => Box::new(NoAction),
                    Some(name) => {
                        let registry = self.action_registry.lock().unwrap();
                        let constructor = registry
                            .get(name)
                            .with_context(|| format!("Unknown action: {}", name))?;
                        constructor()
                    }
                };

                let binding =
                    KeyBinding::new(keystrokes, action, predicate.clone()).with_meta(meta);
                bindings.push(binding);
            }
        }

        Ok(bindings)
    }

    /// Rebuild the merged keymap from default and user bindings.
    /// This is an internal method that expects the caller to hold the lock.
    fn rebuild_keymap_internal(inner: &mut StoreInner) {
        let mut all_bindings = Vec::new();

        // Add defaults first (lower priority)
        for binding in inner.default_bindings.iter() {
            all_bindings.push(binding.clone());
        }

        // Add user bindings (higher priority)
        for binding in inner.user_bindings.iter() {
            all_bindings.push(binding.clone());
        }

        inner.keymap = Keymap::with_bindings(all_bindings);
        inner.version = inner.keymap.version();
    }

    /// Execute a function with read access to the keymap.
    pub fn with_keymap<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Keymap) -> R,
    {
        let inner = self.inner.lock().unwrap();
        f(&inner.keymap)
    }

    /// Reload user bindings from disk.
    pub fn reload(&self) -> Result<()> {
        self.load_user_bindings()
    }

    /// Add a user binding programmatically.
    pub fn add_user_binding(&self, binding: KeyBinding) {
        let mut inner = self.inner.lock().unwrap();
        inner
            .user_bindings
            .push(binding.with_meta(KeyBindingMetaIndex::USER));
        Self::rebuild_keymap_internal(&mut inner);
    }

    /// Remove all user bindings for a specific keystroke sequence.
    pub fn remove_user_binding(&self, keystrokes: &[Keystroke]) {
        let mut inner = self.inner.lock().unwrap();
        inner.user_bindings.retain(|b| b.keystrokes() != keystrokes);
        Self::rebuild_keymap_internal(&mut inner);
    }

    /// Get the path to the user keymap file.
    pub fn user_keymap_path(&self) -> &Path {
        &self.user_keymap_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions;
    use crate::context::KeyContext;
    use tempfile::TempDir;

    actions![SaveFile, OpenFile];

    #[test]
    fn test_builder() {
        let temp_dir = TempDir::new().unwrap();
        let keymap_path = temp_dir.path().join("keymaps.json");

        let store = KeymapStore::builder()
            .with_user_keymap_path(keymap_path.clone())
            .register_action("test::Save", || Box::new(SaveFile))
            .build()
            .unwrap();

        assert_eq!(store.user_keymap_path(), keymap_path.as_path());
    }

    #[test]
    fn test_add_default_bindings() {
        let store = KeymapStore::builder().build().unwrap();

        let binding = KeyBinding::new(
            vec![Keystroke::parse("cmd-s").unwrap()],
            Box::new(SaveFile),
            None,
        );

        store.add_default_bindings(vec![binding]);

        store.with_keymap(|keymap| {
            assert_eq!(keymap.bindings().count(), 1);
        });
    }

    #[test]
    fn test_save_and_load_user_bindings() {
        let temp_dir = TempDir::new().unwrap();
        let keymap_path = temp_dir.path().join("keymaps.json");

        let store = KeymapStore::builder()
            .with_user_keymap_path(keymap_path.clone())
            .register_action("SaveFile", || Box::new(SaveFile))
            .register_action("OpenFile", || Box::new(OpenFile))
            .build()
            .unwrap();

        // Add user bindings
        let binding1 = KeyBinding::new(
            vec![Keystroke::parse("cmd-s").unwrap()],
            Box::new(SaveFile),
            None,
        );

        store.add_user_binding(binding1);

        // Save to file
        store.save_user_bindings().unwrap();
        assert!(keymap_path.exists());

        // Create new store and load
        let store2 = KeymapStore::builder()
            .with_user_keymap_path(keymap_path)
            .register_action("SaveFile", || Box::new(SaveFile))
            .build()
            .unwrap();

        store2.load_user_bindings().unwrap();

        store2.with_keymap(|keymap| {
            assert_eq!(keymap.bindings().count(), 1);
        });
    }

    #[test]
    fn test_user_overrides_default() {
        let temp_dir = TempDir::new().unwrap();
        let keymap_path = temp_dir.path().join("keymaps.json");

        let store = KeymapStore::builder()
            .with_user_keymap_path(keymap_path)
            .register_action("SaveFile", || Box::new(SaveFile))
            .register_action("OpenFile", || Box::new(OpenFile))
            .build()
            .unwrap();

        // Add default binding
        let default_binding = KeyBinding::new(
            vec![Keystroke::parse("cmd-s").unwrap()],
            Box::new(SaveFile),
            None,
        )
        .with_meta(KeyBindingMetaIndex::DEFAULT);
        store.add_default_bindings(vec![default_binding]);

        // Add user override
        let user_binding = KeyBinding::new(
            vec![Keystroke::parse("cmd-s").unwrap()],
            Box::new(OpenFile),
            None,
        )
        .with_meta(KeyBindingMetaIndex::USER);
        store.add_user_binding(user_binding);

        let empty_ctx: Vec<KeyContext> = vec![];
        store.with_keymap(|keymap| {
            let (result, _) =
                keymap.bindings_for_input(&[Keystroke::parse("cmd-s").unwrap()], &empty_ctx);

            assert_eq!(result.len(), 2);
            // User binding should come first
            assert!(result[0].action().partial_eq(&OpenFile));
            assert!(result[1].action().partial_eq(&SaveFile));
        });
    }

    #[test]
    fn test_remove_user_binding() {
        let store = KeymapStore::builder().build().unwrap();

        let binding = KeyBinding::new(
            vec![Keystroke::parse("cmd-s").unwrap()],
            Box::new(SaveFile),
            None,
        );

        store.add_user_binding(binding);
        store.with_keymap(|keymap| {
            assert_eq!(keymap.bindings().count(), 1);
        });

        store.remove_user_binding(&[Keystroke::parse("cmd-s").unwrap()]);
        store.with_keymap(|keymap| {
            assert_eq!(keymap.bindings().count(), 0);
        });
    }
}
