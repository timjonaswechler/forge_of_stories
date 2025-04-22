// src/initialization/state/types.rs
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
