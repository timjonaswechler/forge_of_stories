// src/genetics/components/phenotype/components.rs
// Importiere aus dem übergeordneten components-Modul
use crate::genetics::components::chromosome::types::ChromosomeType;
use crate::genetics::components::genome::types::GeneExpression;
use bevy::prelude::*;
use std::collections::HashMap;

// --- PhenotypeGene ---
#[derive(Debug, Clone, Copy)]
pub struct PhenotypeGene {
    pub value: f32,
    pub expression: GeneExpression,
}

impl PhenotypeGene {
    pub fn new(value: f32, expression: GeneExpression) -> Self {
        Self { value, expression }
    }
    pub fn value(&self) -> f32 {
        self.value
    }
    pub fn expression(&self) -> GeneExpression {
        self.expression
    }
}

// --- Phenotype (Component) ---
#[derive(Component, Debug, Clone)]
pub struct Phenotype {
    pub attributes: HashMap<String, PhenotypeGene>,
    pub attribute_groups: HashMap<ChromosomeType, HashMap<String, PhenotypeGene>>,
}

impl Phenotype {
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
            attribute_groups: HashMap::new(),
        }
    }
}
