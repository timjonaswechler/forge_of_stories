// src/initialization/state/plugin.rs
use super::AppState;
use bevy::prelude::*; // Import AppState from types.rs

/// Plugin zum Initialisieren des `AppState`.
pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
        info!(
            "AppState initialized. Starting in state: {:?}",
            AppState::default()
        );
    }
}
