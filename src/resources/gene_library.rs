// src/resources/gen_library.rs
use crate::components::genetics::{ChromosomeType, GeneExpression, GenePair, GeneVariant};
use bevy::prelude::*;
use rand::{thread_rng, Rng};
use std::collections::HashMap;

#[derive(Resource)]
pub struct GeneLibrary {
    // Gene für visuelle Merkmale (RGB-Werte)
    pub skin_colors: HashMap<String, Vec<(f32, f32, f32)>>,
    pub hair_colors: HashMap<String, Vec<(f32, f32, f32)>>,
    pub eye_colors: HashMap<String, Vec<(f32, f32, f32)>>,
}

impl Default for GeneLibrary {
    fn default() -> Self {
        // Erstelle eine neue GeneLibrary mit Standardwerten
        let mut skin_colors = HashMap::new();
        let mut hair_colors = HashMap::new();
        let mut eye_colors = HashMap::new();

        // Hautfarben direkt aus SkinColorPalette übernommen
        skin_colors.insert(
            "Mensch".to_string(),
            vec![
                // [255, 233, 219] -> (255/255, 233/255, 219/255)
                (1.000, 0.914, 0.859),
                (1.000, 0.878, 0.800),
                (0.984, 0.843, 0.749),
                (0.969, 0.792, 0.671),
                (0.953, 0.757, 0.620),
                (0.929, 0.698, 0.541),
                (0.890, 0.620, 0.447),
                (1.000, 0.886, 0.855),
                (1.000, 0.843, 0.796),
                (0.984, 0.824, 0.753),
                (0.973, 0.776, 0.686),
                (1.000, 0.745, 0.620),
                (0.949, 0.671, 0.541),
                (0.894, 0.608, 0.478),
                (0.957, 0.835, 0.757),
                (0.937, 0.800, 0.714),
                (0.871, 0.725, 0.624),
                (0.843, 0.663, 0.533),
                (0.761, 0.588, 0.475),
                (0.694, 0.502, 0.376),
                (0.627, 0.435, 0.306),
                (0.949, 0.843, 0.769),
                (0.910, 0.784, 0.702),
                (0.859, 0.714, 0.612),
                (0.851, 0.667, 0.549),
                (0.835, 0.608, 0.459),
                (0.725, 0.498, 0.349),
                (0.620, 0.412, 0.278),
                (0.718, 0.459, 0.325),
                (0.624, 0.400, 0.286),
                (0.580, 0.361, 0.255),
                (0.553, 0.341, 0.231),
                (0.478, 0.282, 0.184),
                (0.408, 0.227, 0.145),
                (0.357, 0.188, 0.114),
                (0.588, 0.431, 0.329),
                (0.561, 0.384, 0.263),
                (0.533, 0.357, 0.235),
                (0.467, 0.294, 0.180),
                (0.416, 0.271, 0.169),
                (0.365, 0.208, 0.106),
                (0.337, 0.180, 0.078),
                (0.800, 0.588, 0.478),
                (0.753, 0.506, 0.376),
                (0.675, 0.443, 0.325),
                (0.643, 0.420, 0.306),
                (0.584, 0.365, 0.259),
                (0.506, 0.314, 0.224),
                (0.439, 0.275, 0.188),
                (0.651, 0.475, 0.384),
                (0.616, 0.439, 0.349),
                (0.576, 0.392, 0.306),
                (0.529, 0.341, 0.251),
                (0.459, 0.282, 0.192),
                (0.416, 0.239, 0.157),
                (0.290, 0.149, 0.086),
            ],
        );

        skin_colors.insert(
            "Elf".to_string(),
            vec![
                (1.000, 0.922, 0.804),
                (1.000, 0.855, 0.725),
                (0.961, 0.784, 0.569),
                (0.706, 0.784, 0.902),
            ],
        );
        // Orkische Hautfarben
        skin_colors.insert(
            "Ork".to_string(),
            vec![
                (0.843, 0.878, 0.592),
                (0.761, 0.800, 0.494),
                (0.631, 0.659, 0.396),
                (0.569, 0.600, 0.333),
                (0.443, 0.471, 0.235),
                (0.376, 0.400, 0.200),
                (0.322, 0.341, 0.180),
                (0.255, 0.271, 0.137),
                (0.898, 0.863, 0.561),
                (0.831, 0.788, 0.451),
                (0.741, 0.694, 0.357),
                (0.651, 0.604, 0.271),
                (0.549, 0.506, 0.208),
                (0.451, 0.412, 0.153),
                (0.369, 0.337, 0.118),
                (0.310, 0.282, 0.094),
                (0.878, 0.780, 0.565),
                (0.831, 0.714, 0.451),
                (0.718, 0.596, 0.329),
                (0.659, 0.537, 0.271),
                (0.569, 0.463, 0.224),
                (0.490, 0.392, 0.169),
                (0.431, 0.341, 0.133),
                (0.349, 0.275, 0.102),
            ],
        );
        // Menschliche Haarfarben
        hair_colors.insert(
            "Mensch".to_string(),
            vec![
                (0.1, 0.1, 0.1), // Schwarz
                (0.3, 0.2, 0.1), // Dunkelbraun
                (0.6, 0.4, 0.2), // Hellbraun
                (0.8, 0.7, 0.3), // Blond
                (0.6, 0.3, 0.1), // Rotbraun
                (0.8, 0.1, 0.1), // Rot
            ],
        );

        // Elfische Haarfarben
        hair_colors.insert(
            "Elf".to_string(),
            vec![
                (0.9, 0.9, 0.8), // Platinblond
                (0.8, 0.8, 0.6), // Hellblond
                (0.7, 0.6, 0.3), // Goldblond
                (0.3, 0.2, 0.1), // Dunkelbraun
                (0.1, 0.1, 0.1), // Schwarz
            ],
        );

        // Orkische Haarfarben
        hair_colors.insert(
            "Ork".to_string(),
            vec![
                (0.1, 0.1, 0.1),    // Schwarz
                (0.25, 0.15, 0.05), // Sehr Dunkelbraun
                (0.4, 0.2, 0.1),    // Dunkelbraun
                (0.5, 0.3, 0.2),    // Braun
            ],
        );

        // Menschliche Augenfarben
        eye_colors.insert(
            "Mensch".to_string(),
            vec![
                (0.3, 0.2, 0.1), // Braun
                (0.4, 0.3, 0.2), // Hellbraun
                (0.2, 0.4, 0.6), // Blau
                (0.3, 0.5, 0.2), // Grün
                (0.4, 0.4, 0.1), // Haselnuss
                (0.3, 0.3, 0.3), // Grau
            ],
        );

        // Elfische Augenfarben
        eye_colors.insert(
            "Elf".to_string(),
            vec![
                (0.2, 0.6, 0.8), // Hellblau
                (0.2, 0.7, 0.3), // Smaragdgrün
                (0.6, 0.4, 0.1), // Bernstein
                (0.4, 0.2, 0.7), // Violett
                (0.8, 0.8, 0.2), // Gold
            ],
        );

        // Orkische Augenfarben
        eye_colors.insert(
            "Ork".to_string(),
            vec![
                (0.6, 0.2, 0.1), // Rot
                (0.8, 0.6, 0.0), // Gelb/Amber
                (0.2, 0.2, 0.2), // Dunkelgrau
                (0.1, 0.1, 0.1), // Schwarz
            ],
        );

        Self {
            skin_colors,
            hair_colors,
            eye_colors,
        }
    }
}
impl GeneLibrary {
    // Erzeugt RGB-Gene für Hautfarbe basierend auf einer Spezies
    pub fn create_skin_color_genes(&self, species: &str) -> Option<(GenePair, GenePair, GenePair)> {
        let mut rng = rand::thread_rng();

        if let Some(colors) = self.skin_colors.get(species) {
            if !colors.is_empty() {
                let index = rng.gen_range(0..colors.len());
                let color = colors[index];

                let gene_r = GenePair {
                    maternal: GeneVariant {
                        gene_id: "gene_skin_r".to_string(),
                        value: color.0, // R-Wert
                        expression: GeneExpression::Codominant,
                    },
                    paternal: GeneVariant {
                        gene_id: "gene_skin_r".to_string(),
                        value: color.0, // Gleicher R-Wert
                        expression: GeneExpression::Codominant,
                    },
                    chromosome_type: ChromosomeType::VisualTraits,
                };

                let gene_g = GenePair {
                    maternal: GeneVariant {
                        gene_id: "gene_skin_g".to_string(),
                        value: color.1, // G-Wert
                        expression: GeneExpression::Codominant,
                    },
                    paternal: GeneVariant {
                        gene_id: "gene_skin_g".to_string(),
                        value: color.1, // Gleicher G-Wert
                        expression: GeneExpression::Codominant,
                    },
                    chromosome_type: ChromosomeType::VisualTraits,
                };

                let gene_b = GenePair {
                    maternal: GeneVariant {
                        gene_id: "gene_skin_b".to_string(),
                        value: color.2, // B-Wert
                        expression: GeneExpression::Codominant,
                    },
                    paternal: GeneVariant {
                        gene_id: "gene_skin_b".to_string(),
                        value: color.2, // Gleicher B-Wert
                        expression: GeneExpression::Codominant,
                    },
                    chromosome_type: ChromosomeType::VisualTraits,
                };

                return Some((gene_r, gene_g, gene_b));
            }
        }

        None
    }
    // Erzeugt RGB-Gene für Hautfarbe basierend auf einer Spezies
    pub fn create_hair_color_genes(&self, species: &str) -> Option<(GenePair, GenePair, GenePair)> {
        let mut rng = rand::thread_rng();

        if let Some(colors) = self.hair_colors.get(species) {
            if !colors.is_empty() {
                let index = rng.gen_range(0..colors.len());
                let color = colors[index];

                let gene_r = GenePair {
                    maternal: GeneVariant {
                        gene_id: "gene_hair_r".to_string(),
                        value: color.0, // R-Wert
                        expression: GeneExpression::Codominant,
                    },
                    paternal: GeneVariant {
                        gene_id: "gene_hair_r".to_string(),
                        value: color.0, // Gleicher R-Wert
                        expression: GeneExpression::Codominant,
                    },
                    chromosome_type: ChromosomeType::VisualTraits,
                };

                let gene_g = GenePair {
                    maternal: GeneVariant {
                        gene_id: "gene_hair_g".to_string(),
                        value: color.1, // G-Wert
                        expression: GeneExpression::Codominant,
                    },
                    paternal: GeneVariant {
                        gene_id: "gene_hair_g".to_string(),
                        value: color.1, // Gleicher G-Wert
                        expression: GeneExpression::Codominant,
                    },
                    chromosome_type: ChromosomeType::VisualTraits,
                };

                let gene_b = GenePair {
                    maternal: GeneVariant {
                        gene_id: "gene_hair_b".to_string(),
                        value: color.2, // B-Wert
                        expression: GeneExpression::Codominant,
                    },
                    paternal: GeneVariant {
                        gene_id: "gene_hair_b".to_string(),
                        value: color.2, // Gleicher B-Wert
                        expression: GeneExpression::Codominant,
                    },
                    chromosome_type: ChromosomeType::VisualTraits,
                };

                return Some((gene_r, gene_g, gene_b));
            }
        }

        None
    }

    // Erzeugt RGB-Gene für Hautfarbe basierend auf einer Spezies
    pub fn create_eye_color_genes(&self, species: &str) -> Option<(GenePair, GenePair, GenePair)> {
        let mut rng = rand::thread_rng();

        if let Some(colors) = self.eye_colors.get(species) {
            if !colors.is_empty() {
                let index = rng.gen_range(0..colors.len());
                let color = colors[index];

                let gene_r = GenePair {
                    maternal: GeneVariant {
                        gene_id: "gene_eye_r".to_string(),
                        value: color.0, // R-Wert
                        expression: GeneExpression::Codominant,
                    },
                    paternal: GeneVariant {
                        gene_id: "gene_eye_r".to_string(),
                        value: color.0, // Gleicher R-Wert
                        expression: GeneExpression::Codominant,
                    },
                    chromosome_type: ChromosomeType::VisualTraits,
                };

                let gene_g = GenePair {
                    maternal: GeneVariant {
                        gene_id: "gene_eye_g".to_string(),
                        value: color.1, // G-Wert
                        expression: GeneExpression::Codominant,
                    },
                    paternal: GeneVariant {
                        gene_id: "gene_eye_g".to_string(),
                        value: color.1, // Gleicher G-Wert
                        expression: GeneExpression::Codominant,
                    },
                    chromosome_type: ChromosomeType::VisualTraits,
                };

                let gene_b = GenePair {
                    maternal: GeneVariant {
                        gene_id: "gene_eye_b".to_string(),
                        value: color.2, // B-Wert
                        expression: GeneExpression::Codominant,
                    },
                    paternal: GeneVariant {
                        gene_id: "gene_eye_b".to_string(),
                        value: color.2, // Gleicher B-Wert
                        expression: GeneExpression::Codominant,
                    },
                    chromosome_type: ChromosomeType::VisualTraits,
                };

                return Some((gene_r, gene_g, gene_b));
            }
        }

        None
    }
}
