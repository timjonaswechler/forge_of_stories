// src/plugins/genetics_plugin.rs
use crate::resources::eye_color_inheritance::EyeColorInheritance;
use crate::resources::genetics_generator::GeneticsGenerator;

use crate::components::attributes::{MentalAttributes, PhysicalAttributes, SocialAttributes};
use crate::components::genetics::Phenotype;

use crate::systems::attributes as attr_systems;
use crate::systems::genetics::*;
use bevy::prelude::*;

#[derive(Default)]
pub struct GeneticsPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GeneticsSystemSet {
    GenotypePhenotype,
    AttributeApplication,
    AttributeCalculation,
    PhysicalTraits,
    SpeciesReproduction,
}

impl Plugin for GeneticsPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                GeneticsSystemSet::GenotypePhenotype,
                GeneticsSystemSet::AttributeApplication,
                GeneticsSystemSet::AttributeCalculation,
                GeneticsSystemSet::PhysicalTraits,
                GeneticsSystemSet::SpeciesReproduction,
            )
                .chain(),
        )
        .insert_resource(EyeColorInheritance::new())
        .insert_resource(GeneticsGenerator::default())
        // Genetische Systeme
        .add_systems(
            Update,
            genotype_to_phenotype_system.in_set(GeneticsSystemSet::GenotypePhenotype),
        )
        // Attribut-Anwendungs-Systeme (ohne Prefix-Argument)
        .add_systems(
            Update,
            (
                // Closure ruft apply_attributes jetzt ohne String-Prefix auf
                |query: Query<(&Phenotype, &mut PhysicalAttributes), Changed<Phenotype>>| {
                    attr_systems::apply_attributes::<PhysicalAttributes>(query);
                    // <- Kein Prefix mehr
                },
                |query: Query<(&Phenotype, &mut MentalAttributes), Changed<Phenotype>>| {
                    attr_systems::apply_attributes::<MentalAttributes>(query); // <- Kein Prefix mehr
                },
                |query: Query<(&Phenotype, &mut SocialAttributes), Changed<Phenotype>>| {
                    attr_systems::apply_attributes::<SocialAttributes>(query); // <- Kein Prefix mehr
                },
            )
                .in_set(GeneticsSystemSet::AttributeApplication),
        )
        // Attribut-Berechnungs-Systeme
        .add_systems(
            Update,
            (
                attr_systems::calculate_effective_attribute_values,
                attr_systems::update_attribute_rust,
                // Die update_*_attributes Systeme sind noch Platzhalter
                attr_systems::update_physical_attributes,
                attr_systems::update_mental_attributes,
                attr_systems::update_social_attributes,
            )
                .in_set(GeneticsSystemSet::AttributeCalculation),
        )
        // Körperliche/Visuelle Systeme
        .add_systems(
            Update,
            (
                attr_systems::apply_visual_traits_system,
                // attr_systems::apply_body_structure_system, // Falls implementiert
            )
                .in_set(GeneticsSystemSet::PhysicalTraits),
        );
        // Das ReproductionSystem wird NICHT hier hinzugefügt, da es eher auf Events reagiert
        // und nicht Teil der Genotyp->Phänotyp Pipeline ist. Es wird in main.rs hinzugefügt.
    }
}
