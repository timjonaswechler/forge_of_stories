// src/builders/entity_builder.rs
use bevy::prelude::*;

use crate::components::attributes::{MentalAttributes, PhysicalAttributes, SocialAttributes};
use crate::components::genetics::{Genotype, Phenotype, SpeciesGenes};
use crate::components::visual_traits::VisualTraits;

/// Builder für genetisch definierte Entitäten
pub struct EntityBuilder;

impl EntityBuilder {
    /// Erstellt eine vollständige Entität basierend auf einem Genotyp
    pub fn create_entity_from_genotype(
        commands: &mut Commands,
        genotype: Genotype,
        species_names: Vec<String>,
    ) -> Entity {
        // Spawn Entity mit allen Grundkomponenten - die Systeme werden die Berechnungen übernehmen
        commands
            .spawn((
                genotype,
                Phenotype::new(), // Leerer Phänotyp, wird vom System berechnet
                PhysicalAttributes::new(), // Standardattribute, werden vom System überschrieben
                MentalAttributes::new(), // Standardattribute, werden vom System überschrieben
                SocialAttributes::new(), // Standardattribute, werden vom System überschrieben
                VisualTraits::new(), // Standardwerte, werden vom System überschrieben
                // Füge die Spezies-Gene hinzu
                SpeciesGenes {
                    species: species_names,
                },
            ))
            .id()
    }
}
