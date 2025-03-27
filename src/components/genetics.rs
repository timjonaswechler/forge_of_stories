use bevy::prelude::*;
use std::collections::HashMap;

// Gen-Ausprägung (dominant, rezessiv, kodominant)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeneExpression {
    Dominant,
    Recessive,
    Codominant,
}

// Einzelnes Gen
#[derive(Component, Debug, Clone)]
pub struct Gene {
    pub id: String,
    pub name: String,
    pub expression: GeneExpression,
    pub value: f32, // 0.0-1.0 für die Stärke der Ausprägung
}

// Allel (eine konkrete Ausprägung eines Gens)
#[derive(Debug, Clone)]
pub struct Allele {
    pub gene_id: String,
    pub value: f32,
    pub expression: GeneExpression,
}

// Chromosomen-Paar (für diploide Organismen)
#[derive(Debug, Clone)]
pub struct ChromosomePair {
    pub maternal: Allele, // Von der Mutter
    pub paternal: Allele, // Vom Vater
}

// Genpool eines Organismus
#[derive(Component, Debug, Clone)]
pub struct Genotype {
    pub chromosome_pairs: HashMap<String, ChromosomePair>, // Gen-ID -> Chromosomen-Paar
}

impl Genotype {
    pub fn new() -> Self {
        Self {
            chromosome_pairs: HashMap::new(),
        }
    }
}

// Phänotyp (die sichtbaren/wirksamen Eigenschaften)
#[derive(Component, Debug, Clone)]
pub struct Phenotype {
    pub attributes: HashMap<String, f32>, // Gen-ID -> Phänotyp-Wert
}

impl Phenotype {
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }
}

// Körperstruktur
#[derive(Debug, Clone)]
pub struct BodyPart {
    pub id: String,
    pub name: String,
    pub children: Vec<BodyPart>,
    pub genetic_traits: HashMap<String, f32>, // Gen-ID -> Ausprägung
}

// Komponente für die körperliche Struktur
#[derive(Component, Debug, Clone)]
pub struct BodyStructure {
    pub root: BodyPart,
}

// Zusätzliche Gene für spezifische visuelle Merkmale
#[derive(Component, Debug, Clone)]
pub struct VisualTraits {
    pub skin_color: (f32, f32, f32), // RGB-Werte für die Hautfarbe
    pub hair_color: (f32, f32, f32), // RGB-Werte für die Haarfarbe
    pub eye_color: (f32, f32, f32),  // RGB-Werte für die Augenfarbe
}

// Komponente für Spezieszugehörigkeit (für traditionelle Rassenidentifikation)
#[derive(Component, Debug, Clone)]
pub struct SpeciesIdentity {
    pub primary_species: String, // z.B. "Mensch", "Elf", "Zwerg"
    pub species_percentage: HashMap<String, f32>, // Prozentuale Anteile verschiedener Spezies
}

// Komponente, die anzeigt, dass dieses Wesen ein Elternteil ist
#[derive(Component, Debug)]
pub struct Parent {
    pub children: Vec<Entity>,
}

// Komponente, die auf die Eltern verweist
#[derive(Component, Debug)]
pub struct Ancestry {
    pub mother: Option<Entity>,
    pub father: Option<Entity>,
}
