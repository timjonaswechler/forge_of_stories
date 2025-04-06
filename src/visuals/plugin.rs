// src/visuals/plugin.rs
use bevy::prelude::*;

use crate::{AppState, SimulationSystemSet}; // Importiere aus lib.rs

// Importiere aus dem eigenen Modul
use super::resources::*; // Importiert EyeColorInheritance
use super::systems::*; // Importiert apply_visual_traits_system

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Füge die Ressource für die Augenfarben-Vererbung hinzu
            .insert_resource(EyeColorInheritance::new())
            .add_systems(
                Update,
                apply_visual_traits_system // Das System, das die VisualTraits Komponente aktualisiert
                    // Dieses System sollte nach der Phänotyp-Berechnung laufen
                    .in_set(SimulationSystemSet::VisualTraitApplication)
                    .run_if(in_state(AppState::Running)),
            );
        // Die VisualTraits Komponente wird implizit durch Verwendung registriert.
    }
}
