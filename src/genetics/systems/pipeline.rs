// src/genetics/systems/pipeline.rs

use bevy::prelude::*;

// Nutze die Re-Exports aus src/genetics/mod.rs
use crate::genetics::{GeneExpression, Genotype, Phenotype, PhenotypeGene, SpeciesGenes};

// System zur Berechnung des Phänotyps aus dem Genotyp
pub fn genotype_to_phenotype_system(
    mut query: Query<(&Genotype, &mut Phenotype, &SpeciesGenes), Changed<Genotype>>,
) {
    for (genotype, mut phenotype, _species_genes) in query.iter_mut() {
        phenotype.attributes.clear();
        phenotype.attribute_groups.clear();

        for (gene_id, gene_pair) in genotype.gene_pairs.iter() {
            let (value, expression) =
                match (gene_pair.maternal.expression, gene_pair.paternal.expression) {
                    (GeneExpression::Dominant, GeneExpression::Dominant) => (
                        (gene_pair.maternal.value + gene_pair.paternal.value) / 2.0,
                        GeneExpression::Dominant,
                    ),
                    (GeneExpression::Recessive, GeneExpression::Recessive) => (
                        (gene_pair.maternal.value + gene_pair.paternal.value) / 2.0,
                        GeneExpression::Recessive,
                    ),
                    (GeneExpression::Dominant, GeneExpression::Recessive) => {
                        (gene_pair.maternal.value, GeneExpression::Dominant)
                    }
                    (GeneExpression::Recessive, GeneExpression::Dominant) => {
                        (gene_pair.paternal.value, GeneExpression::Dominant)
                    }
                    (GeneExpression::Codominant, _) | (_, GeneExpression::Codominant) => (
                        (gene_pair.maternal.value + gene_pair.paternal.value) / 2.0,
                        GeneExpression::Codominant,
                    ),
                };

            let phenotype_gene = PhenotypeGene { value, expression };

            phenotype.attributes.insert(gene_id.clone(), phenotype_gene); // Klonen für die erste Map

            phenotype
                .attribute_groups
                .entry(gene_pair.chromosome_type)
                .or_default() // Verwende or_default statt or_insert_with(HashMap::new)
                .insert(gene_id.clone(), phenotype_gene); // Klonen für die zweite Map
        }
    }
}
