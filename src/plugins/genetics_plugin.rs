use crate::systems::attributes::*;
use crate::systems::genetics::*;
use crate::systems::species::*; // Neues Modul für Spezies-Systeme
use bevy::prelude::*;

#[derive(Default)]
pub struct GeneticsPlugin;

impl Plugin for GeneticsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Genetische Systeme
            .add_systems(
                Update,
                (
                    // Grundlegende genetische Systeme
                    genotype_to_phenotype_system,
                    // Attribut-Systeme
                    apply_physical_attributes_system,
                    apply_mental_attributes_system,
                    apply_social_attributes_system,
                    calculate_effective_attribute_values,
                    update_attribute_rust,
                    update_physical_attributes,
                    update_mental_attributes,
                    update_social_attributes,
                    // Körperliche und visuelle Systeme
                    update_visual_traits_system,
                    update_body_structure_system,
                    // Spezies-System
                    update_species_system,
                    reproduction_system,
                ),
            );
    }
}
