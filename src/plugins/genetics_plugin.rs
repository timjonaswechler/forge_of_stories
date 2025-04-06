// src/plugins/genetics_plugin.rs
use super::setup_plugin::AppState;
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
                GeneticsSystemSet::SpeciesReproduction, // Bleibt hier, falls du es so willst
            )
                .chain()
                // WICHTIG: Sorge dafür, dass alle Sets nur im Running State laufen
                .run_if(in_state(AppState::Running)),
        )
        // Ressourcen hier einfügen ist ok, da sie nicht vom State abhängen
        .insert_resource(EyeColorInheritance::new())
        // GeneticsGenerator wird nicht mehr benötigt, wenn die Library extern geladen wird?
        // Überprüfe, ob GeneticsGenerator noch gebraucht wird. Wenn ja, hier lassen.
        // .insert_resource(GeneticsGenerator::default()) // Vorerst auskommentieren/entfernen, falls nicht mehr benötigt
        // Füge .run_if(in_state(AppState::Running)) zu allen Systemen hinzu
        .add_systems(
            Update,
            genotype_to_phenotype_system
                .in_set(GeneticsSystemSet::GenotypePhenotype)
                .run_if(in_state(AppState::Running)), // <- Hinzufügen
        )
        .add_systems(
            Update,
            (
                // Closure ruft apply_attributes jetzt ohne String-Prefix auf
                |query: Query<(&Phenotype, &mut PhysicalAttributes), Changed<Phenotype>>| {
                    attr_systems::apply_attributes::<PhysicalAttributes>(query);
                },
                |query: Query<(&Phenotype, &mut MentalAttributes), Changed<Phenotype>>| {
                    attr_systems::apply_attributes::<MentalAttributes>(query);
                },
                |query: Query<(&Phenotype, &mut SocialAttributes), Changed<Phenotype>>| {
                    attr_systems::apply_attributes::<SocialAttributes>(query);
                },
            )
                .in_set(GeneticsSystemSet::AttributeApplication)
                .run_if(in_state(AppState::Running)), // <- Hinzufügen
        )
        .add_systems(
            Update,
            (
                attr_systems::calculate_effective_attribute_values,
                attr_systems::update_attribute_rust,
                attr_systems::update_physical_attributes,
                attr_systems::update_mental_attributes,
                attr_systems::update_social_attributes,
            )
                .in_set(GeneticsSystemSet::AttributeCalculation)
                .run_if(in_state(AppState::Running)), // <- Hinzufügen
        )
        .add_systems(
            Update,
            (attr_systems::apply_visual_traits_system,)
                .in_set(GeneticsSystemSet::PhysicalTraits)
                .run_if(in_state(AppState::Running)), // <- Hinzufügen
        );
        // ReproductionSystem wird in main.rs hinzugefügt, muss dort auch das run_if bekommen!
    }
}
