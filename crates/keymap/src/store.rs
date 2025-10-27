//! Central store for managing default and user-defined key bindings.

use anyhow::{Context, Result};
use bevy::log::{error, info};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::keystroke::Keystroke;
use crate::spec::ActionBinding;

/// The configuration saved to and loaded from disk.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct KeymapConfig {
    /// User-defined overrides for default key bindings.
    pub user_overrides: HashMap<String, String>, // Storing as String for simpler (de)serialization
}

/// Central store for managing default and user-defined key bindings.
///
/// This is a Bevy resource, typically initialized by a `KeymapPlugin`.
#[derive(Resource)]
pub struct KeymapStore {
    defaults: HashMap<String, Keystroke>,
    user_overrides: HashMap<String, Keystroke>,
    config_path: PathBuf,
    dirty: bool,
}

impl Default for KeymapStore {
    fn default() -> Self {
        Self {
            defaults: HashMap::new(),
            user_overrides: HashMap::new(),
            config_path: PathBuf::from("keymap.json"), // Default path, can be configured
            dirty: false,
        }
    }
}

impl KeymapStore {
    /// Registers a set of default action-to-keystroke bindings.
    ///
    /// This should typically be called by feature plugins (e.g., `PlayerPlugin`)
    /// during their `build` method.
    pub fn register_defaults(&mut self, bindings: &[ActionBinding]) {
        for binding in bindings {
            match Keystroke::parse(binding.default_keystroke) {
                Ok(keystroke) => {
                    self.defaults
                        .insert(binding.action_id.to_string(), keystroke);
                }
                Err(e) => {
                    error!(
                        "Failed to parse default keystroke '{}' for action '{}': {}",
                        binding.default_keystroke, binding.action_id, e
                    );
                }
            }
        }
    }

    /// Attempts to load user-defined key binding overrides from the configured path.
    pub fn load_user_overrides(&mut self) -> Result<()> {
        if !self.config_path.exists() {
            info!(
                "No user keymap config found at {:?}, using defaults.",
                self.config_path
            );
            return Ok(());
        }

        let content = fs::read_to_string(&self.config_path).context(format!(
            "Failed to read user keymap config from {:?}",
            self.config_path
        ))?;

        let config: KeymapConfig =
            serde_json::from_str(&content).context("Failed to parse user keymap config JSON")?;

        self.user_overrides.clear();
        for (action_id, keystroke_str) in config.user_overrides {
            match Keystroke::parse(&keystroke_str) {
                Ok(keystroke) => {
                    self.user_overrides.insert(action_id, keystroke);
                }
                Err(e) => {
                    error!(
                        "Failed to parse user override keystroke '{}' for action '{}': {}",
                        keystroke_str, action_id, e
                    );
                }
            }
        }
        info!(
            "Successfully loaded user keymap config from {:?}",
            self.config_path
        );
        Ok(())
    }

    /// Saves the current user-defined key binding overrides to the configured path.
    pub fn save_user_overrides(&mut self) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let mut serializable_overrides = HashMap::new();
        for (action_id, keystroke) in &self.user_overrides {
            serializable_overrides.insert(action_id.clone(), keystroke.to_string());
        }

        let config = KeymapConfig {
            user_overrides: serializable_overrides,
        };

        let json = serde_json::to_string_pretty(&config)
            .context("Failed to serialize user keymap config to JSON")?;

        fs::write(&self.config_path, json).context(format!(
            "Failed to write user keymap config to {:?}",
            self.config_path
        ))?;

        info!(
            "Successfully saved user keymap config to {:?}",
            self.config_path
        );
        self.dirty = false;
        Ok(())
    }

    /// Sets a user override for a specific action ID.
    ///
    /// This will immediately affect `get_binding` calls. Call `save_user_overrides`
    /// to persist this change.
    pub fn set_user_override(&mut self, action_id: String, new_key: Keystroke) {
        self.user_overrides.insert(action_id, new_key);
        self.dirty = true;
    }

    /// Retrieves the effective keystroke for a given action ID,
    /// applying user overrides if present, otherwise using the default.
    ///
    /// Returns `None` if no default binding is registered for the action ID.
    pub fn get_binding(&self, action_id: &str) -> Option<Keystroke> {
        self.user_overrides
            .get(action_id)
            .cloned()
            .or_else(|| self.defaults.get(action_id).cloned())
    }

    /// Allows configuring the path where user overrides are loaded from/saved to.
    ///
    /// This should be called before `load_user_overrides`.
    pub fn set_config_path(&mut self, path: PathBuf) {
        self.config_path = path;
    }

    /// Check if the store has unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark the store as clean (no unsaved changes).
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
}
