// src/main.rs
use bevy::prelude::*;

use forge_of_stories::{
    app_setup::{AppState, CorePlugin, EventPlugin, SetupPlugin},
    attributes::plugin::AttributesPlugin,
    // debug::plugin::DebugPlugin,
    // genetics::plugin::GeneticsCorePlugin,
    // simulation::plugin::SimulationPlugin,
    // Importiere BEIDE UI Plugins
    ui_components::{LoadingScreenPlugin, MainMenuPlugin},
    // visuals::plugin::VisualsPlugin,
    SimulationSystemSet,
};

fn main() {
    App::new()
        .add_plugins((
            CorePlugin,
            EventPlugin,
            SetupPlugin,
            LoadingScreenPlugin,
            MainMenuPlugin, // MainMenuPlugin hinzufügen
                            // GeneticsCorePlugin,
                            // AttributesPlugin, // Wird weiter unten hinzugefügt
                            // VisualsPlugin,
                            // SimulationPlugin,
                            // DebugPlugin,
        ))
        .init_state::<AppState>()
        .add_plugins(AttributesPlugin) // Lässt sich gut hier hinzufügen
        .configure_sets(
            Update,
            (
                SimulationSystemSet::GenotypePhenotype,
                SimulationSystemSet::AttributeApplication,
                SimulationSystemSet::VisualTraitApplication,
                SimulationSystemSet::AttributeCalculation,
            )
                .chain()
                // Läuft jetzt nur noch im Running State (war schon korrekt)
                .run_if(in_state(AppState::Running)),
        )
        .run();
}
