// src/initialization/plugin.rs
use bevy::prelude::*;

// Import the main plugin from each sub-module
use super::{
    assets::plugin::AssetManagementPlugins, // Import from assets/plugin.rs
    core::plugin::CorePlugin,               // Import from core/plugin.rs
    debug::plugin::DebugPlugin,             // Import from debug/plugin.rs
    events::plugin::EventPlugin,            // Import from events/plugin.rs
    state::plugin::StatePlugin,             // Import from state/plugin.rs
};

pub struct InitializationPlugin;

impl Plugin for InitializationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CorePlugin,
            StatePlugin,
            EventPlugin,
            AssetManagementPlugins,
            DebugPlugin,
        ));
    }
}
