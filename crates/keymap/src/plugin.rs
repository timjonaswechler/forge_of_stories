use crate::store::KeymapStore;
use bevy::prelude::*;
use std::path::PathBuf;

pub struct KeymapPlugin {
    config_path: Option<PathBuf>,
}

impl Default for KeymapPlugin {
    fn default() -> Self {
        Self { config_path: None }
    }
}

impl KeymapPlugin {
    pub fn with_config_path(path: impl Into<PathBuf>) -> Self {
        Self {
            config_path: Some(path.into()),
        }
    }
}

impl Plugin for KeymapPlugin {
    fn build(&self, app: &mut App) {
        let mut store = KeymapStore::default();
        if let Some(path) = &self.config_path {
            store.set_config_path(path.clone());
        }

        app.insert_resource(store)
            .add_systems(Startup, load_user_keymap)
            .add_systems(
                PostUpdate,
                save_keymap_on_change.run_if(resource_changed::<KeymapStore>),
            );
    }
}

fn load_user_keymap(mut store: ResMut<KeymapStore>) {
    if let Err(e) = store.load_user_overrides() {
        warn!("Failed to load user keymap: {e}");
    }
}

/// Auto-save keymap when changes are detected.
///
/// This system checks if the KeymapStore has unsaved changes (dirty flag)
/// and saves it to disk. This is more efficient than using Bevy's change detection
/// as it only saves when user overrides are actually modified.
fn save_keymap_on_change(mut store: ResMut<KeymapStore>) {
    // Only save if the store has unsaved changes
    if store.is_dirty() {
        if let Err(e) = store.save_user_overrides() {
            error!("Failed to save user keymap: {e}");
        } else {
            info!("User keymap auto-saved");
        }
    }
}
