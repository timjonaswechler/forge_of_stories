// src/plugins/genetics_plugin.rs
use crate::resources::eye_color_inheritance::EyeColorInheritance;

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
            // Grundlegende genetische Systeme
            .add_systems(
                Update,
                genotype_to_phenotype_system.in_set(GeneticsSystemSet::GenotypePhenotype),
            )
            // Attribut-Anwendungs-Systeme mit generischem apply_attributes
            .add_systems(
                Update,
                (
                    |phenotype: Query<(&Phenotype, &mut PhysicalAttributes)>| {
                        attr_systems::apply_attributes::<PhysicalAttributes>(phenotype, "gene_")
                    },
                    |phenotype: Query<(&Phenotype, &mut MentalAttributes)>| {
                        attr_systems::apply_attributes::<MentalAttributes>(phenotype, "gene_")
                    },
                    |phenotype: Query<(&Phenotype, &mut SocialAttributes)>| {
                        attr_systems::apply_attributes::<SocialAttributes>(phenotype, "gene_")
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
