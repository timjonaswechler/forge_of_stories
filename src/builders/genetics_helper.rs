// src/builders/genetics_helper.rs
use bevy::prelude::*;
use rand::prelude::*;

use crate::components::gene_types::{AttributeGene, GeneType, VisualGene};
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
        if let Some(gene_eye_color) = gene_library.create_eye_color_genes(species) {
            genotype
                .gene_pairs
                .insert("gene_eye_color".to_string(), gene_eye_color);

            genotype
                .chromosome_groups
                .entry(ChromosomeType::VisualTraits)
                .or_insert_with(Vec::new)
                .append(&mut vec!["gene_eye_color".to_string()]);
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
        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::Strength),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::Agility),
            generate_random_value(0.6),
            generate_random_value(0.6),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::Toughness),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::Endurance),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::Recuperation),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::DiseaseResistance),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::Focus),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );

        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::Creativity),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );
        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::Willpower),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );
        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::AnalyticalAbility),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );
        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::Intuition),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );
        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::Memory),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );
        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::Patience),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );
        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::SpatialSense),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );
        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::Empathy),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );
        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::Leadership),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );
        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::SocialAwareness),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );
        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::LinguisticAbility),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );
        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::Negotiation),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );
        genotype.add_gene_pair_enum(
            GeneType::Attribute(AttributeGene::Musicality),
            generate_random_value(0.5),
            generate_random_value(0.5),
            GeneExpression::Codominant,
            ChromosomeType::Attributes,
        );
    }
}
