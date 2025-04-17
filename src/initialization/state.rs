// src/initialization/state.rs
use bevy::prelude::*;

/// Definiert die Hauptzustände der Anwendung.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Startup, // Lädt essentielle Assets für den Ladebildschirm
    Loading,  // Lädt Spiel-Assets und zeigt Ladebildschirm
    Running,  // Hauptspielschleife / Simulation
    MainMenu, // Zeigt das Hauptmenü
}

/// Plugin zum Initialisieren des `AppState`.
pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
        info!(
            "AppState initialized. Starting in state: {:?}",
            AppState::default()
        );
        // Das init_state wird jetzt hier gemacht, nicht mehr in main.rs
    }
}
