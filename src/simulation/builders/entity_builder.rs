// src/builders/entity_builder.rs
use crate::attributes::components::{MentalAttributes, PhysicalAttributes, SocialAttributes};
use crate::genetics::components::genetics::{Genotype, Phenotype, SpeciesGenes};
use crate::visuals::components::VisualTraits;
use bevy::prelude::*;
// Entferne: use std::default::Default; -> Nicht mehr explizit nötig

pub struct EntityBuilder;

impl EntityBuilder {
    pub fn create_entity_from_genotype(
        commands: &mut Commands,
        genotype: Genotype,
        species_names: Vec<String>,
    ) -> Entity {
        commands
            .spawn((
                genotype,
                SpeciesGenes {
                    species: species_names,
                },
                Phenotype::new(),
                // Verwende jetzt wieder die expliziten Konstruktoren!
                PhysicalAttributes::new(),
                MentalAttributes::new(),
                SocialAttributes::new(),
                VisualTraits::default(), // Hier kann default bleiben, da es ::new() aufruft
            ))
            .id()
    }
}
