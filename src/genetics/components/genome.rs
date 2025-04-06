// src/genetics/components/genome.rs
use crate::genetics::types::{ChromosomeType, GeneExpression, GeneType};
use bevy::prelude::*;
use std::collections::HashMap;
use std::str::FromStr;

// --- GeneVariant ---
#[derive(Debug, Clone)]
pub struct GeneVariant {
    pub gene_id: String,
    pub value: f32,
    pub expression: GeneExpression,
}

// --- GenePair ---
#[derive(Debug, Clone)]
pub struct GenePair {
    pub maternal: GeneVariant,
    pub paternal: GeneVariant,
    pub chromosome_type: ChromosomeType,
}

// --- Genotype (Component) ---
#[derive(Component, Debug, Clone)]
pub struct Genotype {
    pub gene_pairs: HashMap<String, GenePair>,
    pub chromosome_groups: HashMap<ChromosomeType, Vec<String>>,
}

impl Genotype {
    pub fn new() -> Self {
        Self {
            gene_pairs: HashMap::new(),
            chromosome_groups: HashMap::new(),
        }
    }

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
        self.chromosome_groups
            .entry(chromosome_type)
            .or_default()
            .push(gene_id.to_string());
    }

    pub fn add_gene_pair_enum(
        &mut self,
        gene_type: GeneType,
        maternal_value: f32,
        paternal_value: f32,
        expression: GeneExpression,
        chromosome_type: ChromosomeType,
    ) {
        self.add_gene_pair(
            &gene_type.to_string(),
            maternal_value,
            paternal_value,
            expression,
            chromosome_type,
        );
    }

    pub fn get_gene_pair(&self, gene_type: GeneType) -> Option<&GenePair> {
        self.gene_pairs.get(&gene_type.to_string())
    }

    pub fn get_gene_pair_mut(&mut self, gene_type: GeneType) -> Option<&mut GenePair> {
        self.gene_pairs.get_mut(&gene_type.to_string())
    }

    pub fn get_all_gene_types(&self) -> Vec<GeneType> {
        self.gene_pairs
            .keys()
            .filter_map(|key| GeneType::from_str(key).ok())
            .collect()
    }
}

// --- SpeciesGenes (Component) ---
#[derive(Component, Debug, Clone)]
pub struct SpeciesGenes {
    pub species: Vec<String>,
}

impl SpeciesGenes {
    pub fn new() -> Self {
        Self {
            species: Vec::new(),
        }
    }
}
