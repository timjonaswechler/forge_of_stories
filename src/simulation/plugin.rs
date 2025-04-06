// Neue Datei: src/plugins/simulation_plugin.rs
use bevy::prelude::*;

use crate::app_setup::AppState;

use crate::simulation::resources::genetics_generator::GeneticsGenerator;
use crate::simulation::systems::character_spawner::spawn_initial_characters;
use crate::simulation::systems::event_handlers::{
    handle_temporary_attribute_modifiers, send_entity_initialized_events,
};
use crate::simulation::systems::reproduction::reproduction_system;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Ressourcen, die für die Simulation gebraucht werden
            .insert_resource(GeneticsGenerator::default())
            // Startup-Systeme (laufen nur, wenn State erreicht wird)
            .add_systems(OnEnter(AppState::Running), spawn_initial_characters)
            // Update-Systeme (laufen nur im Running State)
            .add_systems(
                Update,
                (
                    send_entity_initialized_events,
                    handle_temporary_attribute_modifiers,
                    reproduction_system, // Dieses System war schon in systems::reproduction
                )
                    .run_if(in_state(AppState::Running)),
            );
    }
}

// Entferne die Implementierungen der Systeme, die verschoben wurden.
// Importiere die Systeme jetzt aus super::systems::*.
// Stelle sicher, dass die Systeme spawn_initial_characters, send_entity_initialized_events, handle_temporary_attribute_modifiers, reproduction_system korrekt registriert werden (mit OnEnter oder Update und run_if).
