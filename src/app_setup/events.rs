// src/app_setup/events.rs
use bevy::prelude::*;

// Events für Zustandsänderungen in der Anwendung
#[derive(Event)]
pub struct AssetsLoadedEvent;

#[derive(Event)]
pub struct AppStartupCompletedEvent;

// Plugin zum Registrieren von Events
pub struct EventPlugin;

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AssetsLoadedEvent>()
            .add_event::<AppStartupCompletedEvent>()
            .add_systems(
                Update,
                (log_app_startup_progress, log_asset_loading_progress),
            );
    }
}

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
