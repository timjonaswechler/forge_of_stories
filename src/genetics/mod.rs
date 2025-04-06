pub mod components;
pub mod plugin;
pub mod systems;
pub mod types;

// Re-exportiere wichtige Typen für den Zugriff via crate::genetics::*
pub use components::{GenePair, GeneVariant, Genotype, Phenotype, PhenotypeGene, SpeciesGenes};

pub use types::{
    AttributeGene, ChromosomeType, GeneExpression, GeneType, ParseGeneError, VisualGene,
};

pub use plugin::GeneticsCorePlugin;
pub use systems::genotype_to_phenotype_system;
