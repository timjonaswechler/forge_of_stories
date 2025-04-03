// src/resources/genetics_generator.rs
use bevy::prelude::*;
use rand::prelude::*;
// use bevy_prng::ChaCha8Rng; // Wenn bevy_prng verwendet wird
// use bevy_rand::prelude::GlobalEntropy; // Wenn bevy_rand verwendet wird

use crate::components::gene_types::{AttributeGene, GeneType, VisualGene};
use crate::components::genetics::{ChromosomeType, GeneExpression, Genotype};
use crate::resources::gene_library::GeneLibrary;

/// Resource für die Genotyp-Generierung
#[derive(Resource, Default)]
pub struct GeneticsGenerator;

// Hilfs-Array mit allen Attribut-Genen für robuste Iteration
const ALL_ATTRIBUTE_GENES: [AttributeGene; 19] = [
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
    AttributeGene::Negotiation, //AttributeGene::Musicality, // Fehlt im Array in Vorlage, ergänzt? JA!
];
// Wichtig: Anzahl im Array muss mit der Anzahl der enum-Varianten übereinstimmen! Musicality fehlte.
const ALL_ATTRIBUTE_GENES_FULL: [AttributeGene; 20] = [
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
    /// Erzeugt einen vollständigen Genotyp für eine neue Entität
    pub fn create_initial_genotype(
        &self,
        gene_library: &Res<GeneLibrary>,
        species: &str,
        // mut rng: ResMut<GlobalEntropy<ChaCha8Rng>> // Besser: RNG als Resource
    ) -> Genotype {
        let mut genotype = Genotype::new();
        // TODO: Einheitliche RNG Resource verwenden statt thread_rng()
        let mut rng = rand::thread_rng();

        // Visuelle Gene hinzufügen (Palette-basiert)
        self.add_visual_genes(&mut genotype, gene_library, species, &mut rng);

        // Attribute-Gene hinzufügen (Verteilungs-basiert)
        self.add_attribute_genes(&mut genotype, gene_library, species, &mut rng);

        genotype
    }

    /// Fügt Gene für visuelle Eigenschaften hinzu (Palette-basiert)
    fn add_visual_genes<R: Rng + ?Sized>(
        &self,
        genotype: &mut Genotype,
        gene_library: &Res<GeneLibrary>,
        species: &str,
        _rng: &mut R, // Aktuell nicht direkt hier verwendet
    ) {
        // Hautfarben-Gene
        if let Some((gene_r, gene_g, gene_b)) = gene_library.create_skin_color_genes(species) {
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

        // Haarfarben-Gene
        if let Some((gene_r, gene_g, gene_b)) = gene_library.create_hair_color_genes(species) {
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

        // Augenfarben-Gene
        if let Some(gene_eye_color) = gene_library.create_eye_color_genes(species) {
            genotype.gene_pairs.insert(
                GeneType::Visual(VisualGene::EyeColor).to_string(),
                gene_eye_color,
            );
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
    fn add_attribute_genes<R: Rng + ?Sized>(
        &self,
        genotype: &mut Genotype,
        gene_library: &Res<GeneLibrary>,
        species: &str,
        rng: &mut R,
    ) {
        // Iteriere über *alle* definierten AttributeGene
        for attribute in ALL_ATTRIBUTE_GENES_FULL.iter() {
            // Generiere maternale und paternale Werte basierend auf der Verteilung der Spezies
            // generate_value_from_distribution gibt den Default, falls Spezies/Attribut unbekannt
            let maternal_value =
                gene_library.generate_value_from_distribution(species, *attribute, rng);
            let paternal_value =
                gene_library.generate_value_from_distribution(species, *attribute, rng);

            // Füge das Genpaar hinzu (nehme Kodominant als Standard-Expression an)
            // Die `add_gene_pair_enum` fügt es auch zur Chromosomengruppe hinzu.
            genotype.add_gene_pair_enum(
                GeneType::Attribute(*attribute),
                maternal_value,
                paternal_value,
                GeneExpression::Codominant, // TODO: Expression könnte auch aus Library kommen?
                ChromosomeType::Attributes,
            );
        }
    }
}
