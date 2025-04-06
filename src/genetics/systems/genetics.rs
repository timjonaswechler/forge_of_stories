// src/systems/genetics.rs
use bevy::prelude::*;
use std::collections::HashMap;

use crate::genetics::components::genetics::{
    GeneExpression, Genotype, Phenotype, PhenotypeGene, SpeciesGenes,
};

// System zur Berechnung des Phänotyps aus dem Genotyp
// Reagiert nur auf Änderungen am Genotyp
pub fn genotype_to_phenotype_system(
    mut query: Query<(&Genotype, &mut Phenotype, &SpeciesGenes), Changed<Genotype>>,
) {
    for (genotype, mut phenotype, _species_genes) in query.iter_mut() {
        // Leere die Phänotyp-Gruppen
        phenotype.attribute_groups.clear();
        phenotype.attributes.clear();

        for (gene_id, gene_pair) in genotype.gene_pairs.iter() {
            // Standardverarbeitung für alle Gene
            let (value, expression) =
                match (gene_pair.maternal.expression, gene_pair.paternal.expression) {
                    // Wenn beide dominant sind oder beide rezessiv, nimm den Durchschnitt
                    (GeneExpression::Dominant, GeneExpression::Dominant) => (
                        (gene_pair.maternal.value + gene_pair.paternal.value) / 2.0,
                        GeneExpression::Dominant,
                    ),
                    (GeneExpression::Recessive, GeneExpression::Recessive) => (
                        (gene_pair.maternal.value + gene_pair.paternal.value) / 2.0,
                        GeneExpression::Recessive,
                    ),

                    // Wenn eins dominant und eins rezessiv ist, nimm das dominante
                    (GeneExpression::Dominant, GeneExpression::Recessive) => {
                        (gene_pair.maternal.value, GeneExpression::Dominant)
                    }
                    (GeneExpression::Recessive, GeneExpression::Dominant) => {
                        (gene_pair.paternal.value, GeneExpression::Dominant)
                    }

                    // Bei Kodominanz: gewichteter Durchschnitt und Codominante Expression
                    (GeneExpression::Codominant, _) | (_, GeneExpression::Codominant) => (
                        (gene_pair.maternal.value + gene_pair.paternal.value) / 2.0,
                        GeneExpression::Codominant,
                    ),
                };

            // Phänotyp-Gen erstellen
            let phenotype_gene = PhenotypeGene { value, expression };

            // Speichere den Wert im allgemeinen Phänotyp
            phenotype
                .attributes
                .insert(gene_id.clone(), phenotype_gene.clone());

            // Speichere den Wert auch in der entsprechenden Chromosomen-Gruppe
            phenotype
                .attribute_groups
                .entry(gene_pair.chromosome_type)
                .or_insert_with(HashMap::new)
                .insert(gene_id.clone(), phenotype_gene);
        }
    }
}
