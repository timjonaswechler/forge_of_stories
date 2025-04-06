use bevy::prelude::*;

// Konstanten bleiben hier oder werden in ein config-Modul verschoben
pub const FIXED_SEED: u64 = 1234567890;
pub const USE_FIXED_SEED: bool = true;

// Modul-Deklarationen bleiben
mod builders;
mod components;
mod config;
mod events;
mod plugins;
mod resources;
mod systems;

// Importiere die Plugins
use plugins::{
    core_plugin::CorePlugin, debug_plugin::DebugPlugin, event_plugin::EventPlugin,
    genetics_plugin::GeneticsPlugin, setup_plugin::SetupPlugin,
    simulation_plugin::SimulationPlugin,
};

fn main() {
    App::new()
        .add_plugins((
            CorePlugin,       // Basis-Setup (DefaultPlugins, Log, Window, RNG)
            EventPlugin,      // Registriert alle Events
            SetupPlugin,      // Lädt Assets, verwaltet AppState, init GeneLibrary
            GeneticsPlugin,   // Genotyp->Phänotyp Pipeline, Attribut-Berechnung
            SimulationPlugin, // Charakter-Erstellung, laufende Systeme (Reproduktion etc.)
            DebugPlugin,      // Debugging-Systeme (nur im Debug-Build)
                              // Füge hier zukünftige Plugins hinzu (z.B. UiPlugin)
        ))
        .run();
}
