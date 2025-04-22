// src/initialization/debug/plugin.rs
use super::systems::enable_ui_debug_on_startup;
use bevy::{
    dev_tools::ui_debug_overlay::{DebugUiPlugin, UiDebugOptions},
    prelude::*,
}; // Import the system

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        // Füge das Bevy Debug UI Plugin hinzu
        app.add_plugins(DebugUiPlugin)
            // Initialisiere die Ressource mit Standardwerten (wird im Startup-System überschrieben)
            .init_resource::<UiDebugOptions>()
            // Füge das Startup-System hinzu, um das Overlay standardmäßig zu deaktivieren
            .add_systems(Startup, enable_ui_debug_on_startup);
    }
}
