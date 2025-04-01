// src/components/genetics.rs
use bevy::prelude::*;
use std::collections::HashMap;

// Chromosomen-Typ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChromosomeType {
    BodyStructure, // Körperbau
    Attributes,    // Attributwerte
    Personality,   // Persönlichkeit
    VisualTraits,  // Aussehen
    Specialized,   // Spezielle Fähigkeiten/Merkmale
}

// Gen-Ausprägung (dominant, rezessiv, kodominant)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeneExpression {
    Dominant,
    Recessive,
    Codominant,
}

// Gene: Blaupause für ein Gen in der Gendatenbank/dem Genpool der Welt
// Definiert die Eigenschaften und möglichen Werte, die ein Gen haben kann
#[derive(Component, Debug, Clone)]
pub struct Gene {
    pub id: String,                                // Eindeutiger Identifikator
    pub name: String,                              // Lesbarer Name (z.B. "Augenfarbe")
    pub description: String,                       // Kurze Beschreibung des Gens
    pub possible_expressions: Vec<GeneExpression>, // Mögliche Expressionen
    pub default_value: f32,                        // Standard/Ausgangswert
    pub value_range: (f32, f32),                   // Min/Max-Wertebereich
    pub mutation_rate: f32,                        // Wahrscheinlichkeit von Mutationen
    pub chromosome_type: ChromosomeType,           // Zuordnung zu einem Chromosomentyp
}

// GeneVariant: Eine spezifische Ausprägung (Allel) eines Gens in einem Individuum
// Jedes Individuum hat zwei GeneVariants für jedes Gen (von Mutter und Vater)
#[derive(Debug, Clone)]
pub struct GeneVariant {
    pub gene_id: String,
    pub value: f32,
    pub expression: GeneExpression,
}

// GenePair: Ein Paar von Genvarianten, das ein komplettes Gen in einem diploiden Organismus darstellt
// Beinhaltet sowohl das mütterliche als auch das väterliche Allel
#[derive(Debug, Clone)]
pub struct GenePair {
    pub maternal: GeneVariant,           // Von der Mutter
    pub paternal: GeneVariant,           // Vom Vater
    pub chromosome_type: ChromosomeType, // Art des Chromosoms
}

// Genpool eines Organismus
#[derive(Component, Debug, Clone)]
pub struct Genotype {
    pub gene_pairs: HashMap<String, GenePair>, // Gen-ID -> Genpaar
    pub chromosome_groups: HashMap<ChromosomeType, Vec<String>>, // Gruppierung nach Chromosomen-Typ
}

impl Genotype {
    pub fn new() -> Self {
        Self {
            gene_pairs: HashMap::new(),
            chromosome_groups: HashMap::new(),
        }
    }

    // Hilfsmethode zum Hinzufügen eines Genpaars
    pub fn add_gene_pair(
        &mut self,
        gene_id: &str,
        maternal_value: f32,
        paternal_value: f32,
        expression: GeneExpression,
        chromosome_type: ChromosomeType,
    ) {
        let gene_pair = GenePair {
            maternal: GeneVariant {
                gene_id: gene_id.to_string(),
                value: maternal_value,
                expression,
            },
            paternal: GeneVariant {
                gene_id: gene_id.to_string(),
                value: paternal_value,
                expression,
            },
            chromosome_type,
        };

        self.gene_pairs.insert(gene_id.to_string(), gene_pair);

        // Zum entsprechenden Chromosomen-Typ hinzufügen
        self.chromosome_groups
            .entry(chromosome_type)
            .or_insert_with(Vec::new)
            .push(gene_id.to_string());
    }
}

// Phänotyp (die sichtbaren/wirksamen Eigenschaften)
#[derive(Component, Debug, Clone)]
pub struct Phenotype {
    pub attributes: HashMap<String, PhenotypeGene>, // Gen-ID -> Phänotyp-Gen
    pub attribute_groups: HashMap<ChromosomeType, HashMap<String, PhenotypeGene>>, // Gruppierung nach Chromosomen-Typ
}

impl Phenotype {
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
            attribute_groups: HashMap::new(),
        }
    }
}
/// Repräsentiert einen einzelnen Wert im Phänotyp
#[derive(Debug, Clone, Copy)]
pub struct PhenotypeGene {
    /// Der numerische Wert des Gens (0.0 - 1.0)
    pub value: f32,
    /// Die Expressionsart des Gens
    pub expression: GeneExpression,
}

impl PhenotypeGene {
    /// Erstellt ein neues PhenotypeGene mit dem angegebenen Wert und der Expressionsart
    pub fn new(value: f32, expression: GeneExpression) -> Self {
        Self { value, expression }
    }

    /// Gibt den numerischen Wert des Gens zurück
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Gibt die Expressionsart des Gens zurück
    pub fn expression(&self) -> GeneExpression {
        self.expression
    }
}

// Komponente für Spezieszugehörigkeit
#[derive(Component, Debug, Clone)]
pub struct SpeciesGenes {
    pub species: Vec<String>, // Liste aller Spezies, die in dem Genpool vorkommen
}

impl SpeciesGenes {
    pub fn new() -> Self {
        Self {
            species: Vec::new(),
        }
    }
}

// Komponente, die anzeigt, dass dieses Wesen ein Elternteil ist
#[derive(Component, Debug)]
pub struct Parent {
    pub children: Vec<Entity>,
}

#[derive(Component, Debug, Clone)]
pub struct VisualTraits {
    pub skin_color: (f32, f32, f32),
    pub hair_color: (f32, f32, f32),
    pub eye_color: (f32, f32, f32),
}

// Komponente, die auf die Eltern verweist
#[derive(Component, Debug)]
pub struct Ancestry {
    pub mother: Option<Entity>,
    pub father: Option<Entity>,
    pub generation: u32, // Generationszähler für evolutionäre Analyse
}

// Komponente für die Fruchtbarkeit und Fortpflanzungsfähigkeit
#[derive(Component, Debug, Clone)]
pub struct Fertility {
    pub fertility_rate: f32, // Grundlegende Fruchtbarkeitsrate (0.0-1.0)
    pub reproduction_cooldown: Option<f32>, // Abklingzeit nach Fortpflanzung
    pub compatibility_modifiers: HashMap<String, f32>, // Kompatibilität mit verschiedenen Spezies
    pub maturity: bool,      // Ist das Wesen fortpflanzungsfähig?
}

impl Fertility {
    pub fn new(fertility_rate: f32) -> Self {
        Self {
            fertility_rate,
            reproduction_cooldown: None,
            compatibility_modifiers: HashMap::new(),
            maturity: false,
        }
    }
}
