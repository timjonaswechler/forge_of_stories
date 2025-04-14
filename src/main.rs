// src/main.rs
use bevy::prelude::*;

use forge_of_stories::{
    app_setup::{AppState, CorePlugin, EventPlugin, SetupPlugin},
    attributes::plugin::AttributesPlugin,
    debug::plugin::DebugPlugin,
    genetics::plugin::GeneticsCorePlugin,
    simulation::plugin::SimulationPlugin,
    visuals::plugin::VisualsPlugin,
    SimulationSystemSet,
};

fn main() {
    App::new()
        .configure_sets(
            Update,
            (
                SimulationSystemSet::GenotypePhenotype,
                SimulationSystemSet::AttributeApplication,
                SimulationSystemSet::VisualTraitApplication,
                SimulationSystemSet::AttributeCalculation,
            )
                .chain()
                .run_if(in_state(AppState::Running)),
        )
        .add_plugins((
            CorePlugin,
            EventPlugin,
            SetupPlugin,
            GeneticsCorePlugin,
            AttributesPlugin,
            VisualsPlugin,
            SimulationPlugin,
            DebugPlugin,
        ))
        .run();
}
