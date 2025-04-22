// src/initialization/events/systems.rs
use super::{AppStartupCompletedEvent, AssetsLoadedEvent};
use bevy::prelude::*; // Import events from types.rs

// System zum Ausgeben von Debugging-Informationen während des Startens
pub fn log_app_startup_progress(mut startup_events: EventReader<AppStartupCompletedEvent>) {
    for _ in startup_events.read() {
        info!("Application startup completed, game is now running");
    }
}

// System zum Ausgeben von Debugging-Informationen während des Asset-Ladens
pub fn log_asset_loading_progress(mut asset_events: EventReader<AssetsLoadedEvent>) {
    for _ in asset_events.read() {
        info!("All assets loaded successfully");
    }
}
