// src/initialization/events/plugin.rs
use super::{
    systems::{log_app_startup_progress, log_asset_loading_progress}, // Import systems
    {AppStartupCompletedEvent, AssetsLoadedEvent},                   // Import events to register
};
use bevy::prelude::*;

/// Plugin zum Registrieren von Events.
pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app // Register events
            .add_event::<AssetsLoadedEvent>()
            .add_event::<AppStartupCompletedEvent>()
            // Add systems
            .add_systems(
                Update,
                (log_app_startup_progress, log_asset_loading_progress),
            );
    }
}
