// src/debug_setup.rs
use bevy::{
    dev_tools::ui_debug_overlay::{DebugUiPlugin, UiDebugOptions},
    prelude::*,
};

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

// Startup-System zum Deaktivieren des Debug-Overlays beim Start
// (Bleibt funktional gleich, nur hierher verschoben)
fn enable_ui_debug_on_startup(mut debug_options: ResMut<UiDebugOptions>) {
    // Setze es hier auf false, um es standardmäßig deaktiviert zu haben.
    // Man kann es dann bei Bedarf im Spiel aktivieren (z.B. mit einer Taste).
    debug_options.enabled = false;

    info!("UI Debug Overlay forced OFF at startup by DebugSetupPlugin.");
}
