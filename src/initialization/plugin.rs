// src/initialization/plugin.rs
use bevy::prelude::*;

use super::{
    assets::AssetManagementPlugins, // Internes Bündel-Plugin für Assets
    core::CorePlugin,
    debug::DebugPlugin, // Name konsistent gemacht
    events::EventPlugin,
    state::StatePlugin, // Plugin zum Initialisieren des States
};

/// Das Haupt-Plugin, das alle notwendigen Setup- und Initialisierungs-Plugins hinzufügt.
pub struct InitializationPlugin;

impl Plugin for InitializationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // --- Reihenfolge geändert! Core zuerst ---
            CorePlugin,  // Fenster, Logging, RNG, Kamera (fügt DefaultPlugins hinzu!)
            StatePlugin, // AppState initialisieren (jetzt NACH DefaultPlugins)
            // ----------------------------------------
            EventPlugin,            // App-weite Events (Startup, AssetsLoaded)
            AssetManagementPlugins, // Bündelt Essential- & GameAsset-Ladevorgänge
            DebugPlugin,            // Debug-UI
        ));
    }
}
