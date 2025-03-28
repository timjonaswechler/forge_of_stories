// src/plugins/genetics_plugin.rs
use crate::systems::attributes as attr_systems;
use crate::systems::genetics::*;
use crate::systems::species::*;
use bevy::prelude::*;

#[derive(Default)]
pub struct GeneticsPlugin;

// Definition der verschiedenen SystemSets für bessere Kontrolle
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum GeneticsSystemSet {
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
                    // Genetische Grundmechanismen müssen zuerst berechnet werden
                    GeneticsSystemSet::GenotypePhenotype,
                    // Dann werden Attribute angewendet
                    GeneticsSystemSet::AttributeApplication,
                    // Danach werden abgeleitete Attribute berechnet
                    GeneticsSystemSet::AttributeCalculation,
                    // Anschließend visuelle und körperliche Eigenschaften
                    GeneticsSystemSet::PhysicalTraits,
                    // Schließlich Spezies und Fortpflanzung
                    GeneticsSystemSet::SpeciesReproduction,
                )
                    .chain(),
            )
            // Grundlegende genetische Systeme
            .add_systems(
                Update,
                genotype_to_phenotype_system.in_set(GeneticsSystemSet::GenotypePhenotype),
            )
            // Attribut-Anwendungs-Systeme
            .add_systems(
                Update,
                (
                    attr_systems::apply_physical_attributes_system,
                    attr_systems::apply_mental_attributes_system,
                    attr_systems::apply_social_attributes_system,
                    attr_systems::apply_personality_system,
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
                    attr_systems::apply_body_structure_system,
                )
                    .in_set(GeneticsSystemSet::PhysicalTraits),
            )
            // Spezies und Fortpflanzungssysteme
            .add_systems(
                Update,
                (update_species_system, reproduction_system)
                    .in_set(GeneticsSystemSet::SpeciesReproduction),
            );
    }
}
