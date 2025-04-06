// Neue Datei: src/plugins/debug_plugin.rs
use crate::app_setup::AppState; // Importiere AppState für run_if
use crate::debug::systems::debug_entities; // Importiere das Debug-System
use crate::SimulationSystemSet;
use bevy::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        // Füge Debug-Systeme nur hinzu, wenn im Debug-Build (optional aber empfohlen)
        #[cfg(debug_assertions)]
        {
            app.add_systems(
                Update,
                debug_entities
                    .after(SimulationSystemSet::AttributeCalculation) // Läuft nach den Genetik-Systemen
                    .run_if(in_state(AppState::Running)),
            );
        }
    }
}
