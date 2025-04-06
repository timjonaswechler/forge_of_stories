use bevy::prelude::*;
use rand::Rng; // Für Rng Trait

use crate::genetics::{
    AttributeGene, ChromosomeType, GeneExpression, GeneType, Genotype, VisualGene,
};
use crate::simulation::resources::gene_library::GeneLibrary;

#[derive(Resource, Default)]
pub struct GeneticsGenerator;

const ALL_ATTRIBUTE_GENES: [AttributeGene; 20] = [
    AttributeGene::Strength,
    AttributeGene::Agility,
    AttributeGene::Toughness,
    AttributeGene::Endurance,
    AttributeGene::Recuperation,
    AttributeGene::DiseaseResistance,
    AttributeGene::Focus,
    AttributeGene::Creativity,
    AttributeGene::Willpower,
    AttributeGene::AnalyticalAbility,
    AttributeGene::Intuition,
    AttributeGene::Memory,
    AttributeGene::Patience,
    AttributeGene::SpatialSense,
    AttributeGene::Empathy,
    AttributeGene::Leadership,
    AttributeGene::SocialAwareness,
    AttributeGene::LinguisticAbility,
    AttributeGene::Negotiation,
    AttributeGene::Musicality,
];

impl GeneticsGenerator {
    pub fn create_initial_genotype<Gen: Rng + ?Sized>(
        &self,
        gene_library: &Res<GeneLibrary>,
        species: &str,
        rng: &mut Gen,
    ) -> Genotype {
        let mut genotype = Genotype::new();

        self.add_visual_genes(&mut genotype, gene_library, species, rng);
        self.add_attribute_genes(&mut genotype, gene_library, species, rng);

        genotype
    }

    // Helfer bleiben generisch mit <R: Rng + ?Sized>
    fn add_visual_genes<R: Rng + ?Sized>(
        &self,
        genotype: &mut Genotype,
        gene_library: &Res<GeneLibrary>,
        species: &str,
        rng: &mut R,
    ) {
        // Ruft gene_library Funktionen auf, die rng verwenden
        if let Some((gene_r, gene_g, gene_b)) = gene_library.create_skin_color_genes(species, rng) {
            genotype
                .gene_pairs
                .insert(GeneType::Visual(VisualGene::SkinColorR).to_string(), gene_r);
            genotype
                .gene_pairs
                .insert(GeneType::Visual(VisualGene::SkinColorG).to_string(), gene_g);
            genotype
                .gene_pairs
                .insert(GeneType::Visual(VisualGene::SkinColorB).to_string(), gene_b);
            genotype
                .chromosome_groups
                .entry(ChromosomeType::VisualTraits)
                .or_default()
                .extend(vec![
                    GeneType::Visual(VisualGene::SkinColorR).to_string(),
                    GeneType::Visual(VisualGene::SkinColorG).to_string(),
                    GeneType::Visual(VisualGene::SkinColorB).to_string(),
                ]);
        } else {
            warn!("Keine Hautfarbengene für Spezies '{}' generiert.", species);
        }

        // Haarfarben-Gene - GIB DEN RNG WEITER

        if let Some((gene_r, gene_g, gene_b)) = gene_library.create_hair_color_genes(species, rng) {
            genotype
                .gene_pairs
                .insert(GeneType::Visual(VisualGene::HairColorR).to_string(), gene_r);
            genotype
                .gene_pairs
                .insert(GeneType::Visual(VisualGene::HairColorG).to_string(), gene_g);
            genotype
                .gene_pairs
                .insert(GeneType::Visual(VisualGene::HairColorB).to_string(), gene_b);
            genotype
                .chromosome_groups
                .entry(ChromosomeType::VisualTraits)
                .or_default()
                .extend(vec![
                    GeneType::Visual(VisualGene::HairColorR).to_string(),
                    GeneType::Visual(VisualGene::HairColorG).to_string(),
                    GeneType::Visual(VisualGene::HairColorB).to_string(),
                ]);
        } else {
            warn!("Keine Haarfarbengene für Spezies '{}' generiert.", species);
        }

        // Augenfarben-Gene - GIB DEN RNG WEITER
        if let Some(gene_eye_color) = gene_library.create_eye_color_genes(species, rng) {
            genotype.gene_pairs.insert(
                GeneType::Visual(VisualGene::EyeColor).to_string(),
                gene_eye_color.clone(),
            ); // Clone, falls noch gebraucht
            genotype
                .chromosome_groups
                .entry(ChromosomeType::VisualTraits)
                .or_default()
                .push(GeneType::Visual(VisualGene::EyeColor).to_string());
        } else {
            warn!("Keine Augenfarbengene für Spezies '{}' generiert.", species);
        }
    }

    /// Fügt Gene für Attribute hinzu (Verteilungs-basiert) - Robuste Variante
    /// Akzeptiert jetzt einen mutable RNG Trait-Objekt
    fn add_attribute_genes<R: Rng + ?Sized>(
        &self,
        genotype: &mut Genotype,
        gene_library: &Res<GeneLibrary>,
        species: &str,
        rng: &mut R,
    ) {
        for attribute in ALL_ATTRIBUTE_GENES.iter() {
            let maternal_value =
                gene_library.generate_value_from_distribution(species, *attribute, rng);
            let paternal_value =
                gene_library.generate_value_from_distribution(species, *attribute, rng);

            genotype.add_gene_pair_enum(
                GeneType::Attribute(*attribute),
                maternal_value,
                paternal_value,
                GeneExpression::Codominant,
                ChromosomeType::Attributes,
            );
        }
    }
}
