// src/components/genetics.rs
use crate::components::gene_types::GeneType;
use bevy::prelude::*;
use std::collections::HashMap;
use std::str::FromStr;
// Chromosomen-Typ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChromosomeType {
    BodyStructure, // Körperbau
    Attributes,    // Attributwerte
    VisualTraits,  // Aussehen
}

// Gen-Ausprägung (dominant, rezessiv, kodominant)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeneExpression {
    Dominant,
    Recessive,
    Codominant,
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
    /// Fügt ein Genpaar mit Enum-basierter Gen-ID hinzu
    pub fn add_gene_pair_enum(
        &mut self,
        gene_type: GeneType,
        maternal_value: f32,
        paternal_value: f32,
        expression: GeneExpression,
        chromosome_type: ChromosomeType,
    ) {
        let gene_id = gene_type.to_string();
        self.add_gene_pair(
            &gene_id,
            maternal_value,
            paternal_value,
            expression,
            chromosome_type,
        );
    }

    /// Gibt ein Genpaar basierend auf dem Enum-Typ zurück
    pub fn get_gene_pair(&self, gene_type: GeneType) -> Option<&GenePair> {
        self.gene_pairs.get(&gene_type.to_string())
    }

    /// Gibt ein Genpaar basierend auf dem Enum-Typ zurück (mutable)
    pub fn get_gene_pair_mut(&mut self, gene_type: GeneType) -> Option<&mut GenePair> {
        self.gene_pairs.get_mut(&gene_type.to_string())
    }

    /// Konvertiert alle String-Gen-IDs in der Genotype-Struktur zu Enum-Typen
    pub fn get_all_gene_types(&self) -> Vec<GeneType> {
        self.gene_pairs
            .keys()
            .filter_map(|key| GeneType::from_str(key).ok())
            .collect()
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
