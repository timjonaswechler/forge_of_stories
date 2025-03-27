use crate::systems::attributes::*;
use crate::systems::genetics::*;
use bevy::prelude::*;

#[derive(Default)]
pub struct GeneticsPlugin;

impl Plugin for GeneticsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Füge die genetischen Systeme hinzu
            .add_systems(
                Update,
                (
                    genotype_to_phenotype_system,
                    apply_physical_attributes_system,
                    apply_mental_attributes_system,
                    apply_social_attributes_system,
                    update_visual_traits_system,
                    calculate_fertility_system,
                    // Neue Attribut-Systeme
                    calculate_effective_attribute_values,
                    update_attribute_rust,
                    update_physical_attributes,
                    update_mental_attributes,
                    update_social_attributes,
                ),
            )
            // Das Reproduktionssystem könnte in einem eigenen Schedule laufen,
            // z.B. wenn ein Tag in der Simulation vergeht
            .add_systems(Update, reproduction_system);
    }
}
