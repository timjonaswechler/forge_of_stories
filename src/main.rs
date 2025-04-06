// src/main.rs
use bevy::prelude::*;

// Importiere direkt aus dem Crate-Root (lib.rs)
use forge_of_stories::{
    app_setup::{AppState, CorePlugin, EventPlugin, SetupPlugin},
    attributes::plugin::AttributesPlugin,
    debug::plugin::DebugPlugin,
    genetics::plugin::GeneticsCorePlugin, // Füge dies hinzu, wenn du es erstellt hast
    simulation::plugin::SimulationPlugin,
    visuals::plugin::VisualsPlugin,
    SimulationSystemSet, // Importiere das Set
                         // Konstanten werden jetzt auch hier importiert, falls sie in lib.rs sind
                         // FIXED_SEED, USE_FIXED_SEED,
};

fn main() {
    App::new()
        // Konfiguriere die System Sets für die Update-Phase
        .configure_sets(
            Update,
            (
                // Reihenfolge der Ausführung definieren
                SimulationSystemSet::GenotypePhenotype,
                SimulationSystemSet::AttributeApplication,
                SimulationSystemSet::VisualTraitApplication,
                SimulationSystemSet::AttributeCalculation,
            )
                .chain()
                .run_if(in_state(AppState::Running)), // Alle laufen nur im Running State und nacheinander
        )
        // Füge die Plugins hinzu
        .add_plugins((
            CorePlugin,         // Basis-Setup (DefaultPlugins, Log, Window, RNG)
            EventPlugin,        // Registriert alle Events
            SetupPlugin,        // Lädt Assets, verwaltet AppState, init GeneLibrary
            GeneticsCorePlugin, // Fügt genotype_to_phenotype hinzu (wenn erstellt)
            AttributesPlugin,   // Fügt Attribut-Systeme hinzu
            VisualsPlugin,      // Fügt Visual-Systeme und Ressourcen hinzu
            SimulationPlugin,   // Charakter-Erstellung, laufende Systeme (Reproduktion etc.)
            DebugPlugin,        // Debugging-Systeme (nur im Debug-Build)
        ))
        .run();
}
