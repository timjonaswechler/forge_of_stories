use crate::components::genetics::{Genotype, Phenotype, SpeciesGenes};
use crate::resources::skin_color_palette::SkinColorPalette;
use bevy::prelude::*;

// System zur Aktualisierung der Speziesliste basierend auf den Genen
pub fn update_species_system(mut query: Query<(&Genotype, &mut SpeciesGenes)>) {
    for (genotype, mut species_genes) in query.iter_mut() {
        // Leere die aktuelle Liste
        species_genes.species.clear();

        // Sammle alle einzigartigen Spezies-Gene
        let mut found_species = std::collections::HashSet::new();

        for (gene_id, _) in genotype.gene_pairs.iter() {
            // Suche nach Genen, die Speziesmerkmale definieren
            // Dies könnte durch ein Präfix oder ein spezielles Merkmal im Gen-ID erkannt werden
            if gene_id.starts_with("gene_species_") {
                let species_name = gene_id
                    .strip_prefix("gene_species_")
                    .unwrap_or("unknown")
                    .to_string();

                found_species.insert(species_name);
            }
        }

        // Füge die gefundenen Spezies der Liste hinzu
        species_genes.species = found_species.into_iter().collect();
    }
}
