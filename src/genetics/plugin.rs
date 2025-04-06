use bevy::prelude::*;

use crate::{AppState, SimulationSystemSet}; // Importiere aus lib.rs

// Importiere aus dem eigenen Modul
use super::systems::genotype_to_phenotype_system;

pub struct GeneticsCorePlugin;

impl Plugin for GeneticsCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            genotype_to_phenotype_system // Das Kernsystem der Genetik-Pipeline
                .in_set(SimulationSystemSet::GenotypePhenotype)
                .run_if(in_state(AppState::Running)),
        );
        // Die Genetik-Komponenten werden implizit durch Verwendung registriert.
    }
}
