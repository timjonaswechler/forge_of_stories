// src/initialization/debug/systems.rs
use bevy::{dev_tools::ui_debug_overlay::UiDebugOptions, prelude::*};

// Startup-System zum Deaktivieren des Debug-Overlays beim Start
pub fn enable_ui_debug_on_startup(mut debug_options: ResMut<UiDebugOptions>) {
    // Setze es hier auf false, um es standardmäßig deaktiviert zu haben.
    debug_options.enabled = false;

    info!("UI Debug Overlay forced OFF at startup by DebugPlugin.");
}
