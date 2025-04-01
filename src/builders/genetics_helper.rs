// src/builders/genetics_helper.rs
use bevy::prelude::*;
use rand::prelude::*;

use crate::components::genetics::{ChromosomeType, GeneExpression, Genotype};
use crate::resources::gene_library::GeneLibrary;

/// Hilfsfunktionen für die Genotyp-Generierung
pub struct GeneticsHelper;

impl GeneticsHelper {
    /// Erzeugt einen vollständigen Genotyp für eine neue Entität
    pub fn create_initial_genotype(gene_library: &Res<GeneLibrary>, species: &str) -> Genotype {
        let mut genotype = Genotype::new();

        // Visuelle Gene hinzufügen
        Self::add_visual_genes(&mut genotype, gene_library, species);

        // Attribute-Gene hinzufügen
        Self::add_attribute_genes(&mut genotype, gene_library, species);

        genotype
    }

    /// Fügt Gene für visuelle Eigenschaften hinzu
    pub fn add_visual_genes(
        genotype: &mut Genotype,
        gene_library: &Res<GeneLibrary>,
        species: &str,
    ) {
        // Hautfarben-Gene
        if let Some((gene_r, gene_g, gene_b)) = gene_library.create_skin_color_genes(species) {
            genotype
                .gene_pairs
                .insert("gene_skin_r".to_string(), gene_r);
            genotype
                .gene_pairs
                .insert("gene_skin_g".to_string(), gene_g);
            genotype
                .gene_pairs
                .insert("gene_skin_b".to_string(), gene_b);

            genotype
                .chromosome_groups
                .entry(ChromosomeType::VisualTraits)
                .or_insert_with(Vec::new)
                .append(&mut vec![
                    "gene_skin_r".to_string(),
                    "gene_skin_g".to_string(),
                    "gene_skin_b".to_string(),
                ]);
        }

        // Haarfarben-Gene
        if let Some((gene_r, gene_g, gene_b)) = gene_library.create_hair_color_genes(species) {
            genotype
                .gene_pairs
                .insert("gene_hair_r".to_string(), gene_r);
            genotype
                .gene_pairs
                .insert("gene_hair_g".to_string(), gene_g);
            genotype
                .gene_pairs
                .insert("gene_hair_b".to_string(), gene_b);

            genotype
                .chromosome_groups
                .entry(ChromosomeType::VisualTraits)
                .or_insert_with(Vec::new)
                .append(&mut vec![
                    "gene_hair_r".to_string(),
                    "gene_hair_g".to_string(),
                    "gene_hair_b".to_string(),
                ]);
        }

        // Augenfarben-Gene
        if let Some((gene_r, gene_g, gene_b)) = gene_library.create_eye_color_genes(species) {
            genotype.gene_pairs.insert("gene_eye_r".to_string(), gene_r);
            genotype.gene_pairs.insert("gene_eye_g".to_string(), gene_g);
            genotype.gene_pairs.insert("gene_eye_b".to_string(), gene_b);

            genotype
                .chromosome_groups
                .entry(ChromosomeType::VisualTraits)
                .or_insert_with(Vec::new)
                .append(&mut vec![
                    "gene_eye_r".to_string(),
                    "gene_eye_g".to_string(),
                    "gene_eye_b".to_string(),
                ]);
        }
    }

    /// Fügt Gene für Attribute hinzu
    pub fn add_attribute_genes(
        genotype: &mut Genotype,
        gene_library: &Res<GeneLibrary>,
        species: &str,
    ) {
        let randomize = true;
        let mut rng = rand::thread_rng();

        // Generiert einen zufälligen Wert innerhalb des angegebenen Bereichs
        let mut generate_random_value = |base: f32| -> f32 {
            if randomize {
                rng.gen_range(base - 0.2..=base + 0.2).max(0.1).min(0.9)
            } else {
                base
            }
        };

        // Physische Attribute
        genotype.add_gene_pair(
            "gene_strength",
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair(
            "gene_agility",
            generate_random_value(0.6),
            generate_random_value(0.6),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair(
            "gene_toughness",
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair(
            "gene_endurance",
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair(
            "gene_recuperation",
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair(
            "gene_disease_resistance",
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        // Mentale Attribute
        genotype.add_gene_pair(
            "gene_focus",
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair(
            "gene_creativity",
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair(
            "gene_willpower",
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair(
            "gene_analytical_ability",
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair(
            "gene_intuition",
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair(
            "gene_memory",
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        // Soziale Attribute
        genotype.add_gene_pair(
            "gene_empathy",
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair(
            "gene_leadership",
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair(
            "gene_social_awareness",
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair(
            "gene_linguistic_ability",
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair(
            "gene_negotiation",
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );
    }
}
