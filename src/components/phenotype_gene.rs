// src/components/phenotype_gene.rs
//
// In dieser Datei definieren wir die PhenotypeGene-Struktur,
// die als Wrapper für genetische Werte im Phänotyp dient.
use crate::components::genetics::GeneExpression;

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
