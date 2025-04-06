// src/genetics/mod.rs
pub mod components; // Verweist auf den Ordner components/
pub mod plugin;
pub mod systems; // Falls vorhanden

// Re-exportiere die wichtigsten Typen für den Zugriff via genetics::*
pub use components::{
    AttributeGene,
    // Aus common_types
    GeneExpression,
    GenePair,
    GeneType,
    GeneVariant,
    // Aus genome
    Genotype,
    ParseGeneError,
    // Aus phenotype
    Phenotype,
    PhenotypeGene,
    SpeciesGenes,
    VisualGene,
};
pub use systems::genotype_to_phenotype_system;
// pub use plugin::GeneticsCorePlugin; // Falls vorhanden
