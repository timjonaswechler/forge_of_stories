// src/resources/gene_library.rs
use crate::genetics::components::gene_types::{AttributeGene, GeneType, VisualGene};
use crate::genetics::components::genetics::{
    ChromosomeType, GeneExpression, GenePair, GeneVariant,
};
use crate::visuals::components::EyeColor;
use bevy::prelude::Color;
use bevy::prelude::*;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use serde::Deserialize;
use std::collections::HashMap;

// Struktur zur Beschreibung der Verteilung eines Gens (Mittelwert und Standardabweichung für 0.0-1.0 Skala)
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct GeneDistribution {
    pub mean: f32,    // Mittelwert (sollte zwischen 0 und 1 liegen)
    pub std_dev: f32, // Standardabweichung
}

impl Default for GeneDistribution {
    fn default() -> Self {
        // Ein generischer Default (z.B. für unbekannte Gene/Spezies)
        GeneDistribution {
            mean: 0.5,     // Entspricht 2500 auf der 0-5000 Skala
            std_dev: 0.15, // Standardabweichung auf der 0-1 Skala
        }
    }
}

#[derive(Resource)]
pub struct GeneLibrary {
    // --- Visuelle Merkmale (Presets/Paletten) ---
    pub skin_colors: HashMap<String, Vec<Color>>,
    pub hair_colors: HashMap<String, Vec<Color>>,
    pub eye_colors: HashMap<String, Vec<EyeColor>>,

    // --- Attribut-Merkmale (Verteilungen auf 0.0-1.0 Skala) ---
    // Spezies -> Attribut-Typ -> Verteilung
    pub attribute_distributions: HashMap<String, HashMap<AttributeGene, GeneDistribution>>,
}

impl Default for GeneLibrary {
    fn default() -> Self {
        // Initialisiert jetzt LEERE HashMaps
        Self {
            skin_colors: HashMap::new(),
            hair_colors: HashMap::new(),
            eye_colors: HashMap::new(),
            attribute_distributions: HashMap::new(),
        }
    }
}

impl GeneLibrary {
    // --- Methoden zur Abfrage ---

    /// Gibt die Verteilung (Mittelwert/StdAbw auf 0-1 Skala) für ein spezifisches Attribut-Gen einer Spezies zurück.
    /// Fällt auf einen generischen Default zurück, wenn nichts gefunden wird.
    pub fn get_attribute_distribution(
        &self,
        species: &str,
        attribute: AttributeGene,
    ) -> GeneDistribution {
        self.attribute_distributions
            .get(species)
            .and_then(|species_dist| species_dist.get(&attribute))
            .copied() // Kopiert die gefundene Distribution
            .unwrap_or_else(|| {
                 // Warnung, wenn Spezies oder Attribut nicht gefunden wurde, nutze Default
                warn!("Keine spezifische Genverteilung für {:?} in Spezies '{}' gefunden. Verwende Default (0.5 ± 0.15).", attribute, species);
                GeneDistribution::default()
            })
    }

    /// Generiert einen zufälligen Wert (0.0 bis 1.0) für ein Gen basierend auf seiner Verteilung.
    /// Der Wert wird auf den Bereich [0.0, 1.0] geklemmt.
    pub fn generate_value_from_distribution<R: Rng + ?Sized>(
        &self,
        species: &str,
        attribute: AttributeGene,
        rng: &mut R,
    ) -> f32 {
        // Gibt jetzt immer einen f32 zurück (mit Fallback)
        let dist_params = self.get_attribute_distribution(species, attribute);
        match Normal::new(dist_params.mean, dist_params.std_dev) {
            Ok(normal_dist) => {
                let value = normal_dist.sample(rng);
                value.clamp(0.0, 1.0) // Klemmt den Wert auf [0.0, 1.0]
            }
            Err(err) => {
                warn!(
                    "Konnte Normalverteilung nicht erstellen für Spezies '{}', Gen '{:?}': {:?}. Parameter: {:?}. Verwende Mittelwert als Fallback.",
                    species, attribute, err, dist_params
                );
                // Fallback auf Mittelwert, wenn Verteilung ungültig
                dist_params.mean.clamp(0.0, 1.0)
            }
        }
    }

    // --- Methoden zur Farb-Gene-Erzeugung (basierend auf Paletten) ---
    fn create_color_gene_pair(visual_gene: VisualGene, value: f32) -> GenePair {
        let gene_id = GeneType::Visual(visual_gene).to_string(); // Korrekte String-ID ableiten
        GenePair {
            maternal: GeneVariant {
                gene_id: gene_id.clone(),
                value,
                expression: GeneExpression::Codominant,
            },
            paternal: GeneVariant {
                gene_id, // Move statt Clone ist ok
                value,
                expression: GeneExpression::Codominant,
            },
            chromosome_type: ChromosomeType::VisualTraits,
        }
    }

    pub fn create_skin_color_genes<R: Rng + ?Sized>(
        &self,
        species: &str,
        rng: &mut R,
    ) -> Option<(GenePair, GenePair, GenePair)> {
        if let Some(colors) = self.skin_colors.get(species) {
            if !colors.is_empty() {
                let color = colors[rng.gen_range(0..colors.len())];
                let rgba_color = color.to_srgba();
                let (r_val, g_val, b_val) = (rgba_color.red, rgba_color.green, rgba_color.blue);

                let gene_r = Self::create_color_gene_pair(VisualGene::SkinColorR, r_val);
                let gene_g = Self::create_color_gene_pair(VisualGene::SkinColorG, g_val);
                let gene_b = Self::create_color_gene_pair(VisualGene::SkinColorB, b_val);
                return Some((gene_r, gene_g, gene_b));
            }
        }
        None
    }

    pub fn create_hair_color_genes<R: Rng + ?Sized>(
        &self,
        species: &str,
        rng: &mut R,
    ) -> Option<(GenePair, GenePair, GenePair)> {
        if let Some(colors) = self.hair_colors.get(species) {
            if !colors.is_empty() {
                let color = colors[rng.gen_range(0..colors.len())];
                let rgba_color = color.to_srgba();
                let (r_val, g_val, b_val) = (rgba_color.red, rgba_color.green, rgba_color.blue);

                let gene_r = Self::create_color_gene_pair(VisualGene::HairColorR, r_val);
                let gene_g = Self::create_color_gene_pair(VisualGene::HairColorG, g_val);
                let gene_b = Self::create_color_gene_pair(VisualGene::HairColorB, b_val);
                return Some((gene_r, gene_g, gene_b));
            }
        }
        None
    }

    pub fn create_eye_color_genes<R: Rng + ?Sized>(
        // <- Signatur geändert
        &self,
        species: &str,
        rng: &mut R, // <- RNG Parameter
    ) -> Option<GenePair> {
        if let Some(colors) = self.eye_colors.get(species) {
            if !colors.is_empty() {
                let color1 = colors[rng.gen_range(0..colors.len())];
                let color2 = colors[rng.gen_range(0..colors.len())];
                // Der GeneType::Visual(VisualGene::EyeColor) String wird hier weiterhin verwendet, da der Key String sein muss.
                let gene_id = GeneType::Visual(VisualGene::EyeColor).to_string();

                return Some(GenePair {
                    maternal: GeneVariant {
                        gene_id: gene_id.clone(),
                        value: color1.to_f32(),
                        expression: GeneExpression::Codominant,
                    },
                    paternal: GeneVariant {
                        gene_id,
                        value: color2.to_f32(),
                        expression: GeneExpression::Codominant,
                    },
                    chromosome_type: ChromosomeType::VisualTraits,
                });
            }
        }
        None
    }
}
