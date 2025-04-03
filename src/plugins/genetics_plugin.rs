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

// Definition der verschiedenen SystemSets für bessere Kontrolle
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GeneticsSystemSet {
    // Genetische Grundlagenberechnungen
    GenotypePhenotype,
    // Anwendung von Genen auf Attribute und Eigenschaften
    AttributeApplication,
    // Berechnung von Attributen und deren Effekten
    AttributeCalculation,
    // Visuelle und körperliche Eigenschaften
    PhysicalTraits,
    // Spezies und Fortpflanzung
    SpeciesReproduction,
}

impl Plugin for GeneticsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Definieren der SystemSets und ihrer Abhängigkeiten
            .configure_sets(
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
            // Grundlegende genetische Systeme
            .add_systems(
                Update,
                genotype_to_phenotype_system.in_set(GeneticsSystemSet::GenotypePhenotype),
            )
            // Attribut-Anwendungs-Systeme mit generischem apply_attributes und Changed<Phenotype>
            .add_systems(
                Update,
                (
                    |query: Query<(&Phenotype, &mut PhysicalAttributes), Changed<Phenotype>>| {
                        attr_systems::apply_attributes::<PhysicalAttributes>(query, "gene_")
                    },
                    |query: Query<(&Phenotype, &mut MentalAttributes), Changed<Phenotype>>| {
                        attr_systems::apply_attributes::<MentalAttributes>(query, "gene_")
                    },
                    |query: Query<(&Phenotype, &mut SocialAttributes), Changed<Phenotype>>| {
                        attr_systems::apply_attributes::<SocialAttributes>(query, "gene_")
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
                    attr_systems::update_physical_attributes,
                    attr_systems::update_mental_attributes,
                    attr_systems::update_social_attributes,
                )
                    .in_set(GeneticsSystemSet::AttributeCalculation),
            )
            // Körperliche und visuelle Systeme
            .add_systems(
                Update,
                (
                    attr_systems::apply_visual_traits_system,
                    // attr_systems::apply_body_structure_system,
                )
                    .in_set(GeneticsSystemSet::PhysicalTraits),
            );
    }
}
