// src/resources/gene_library.rs
use crate::components::gene_types::{AttributeGene, GeneType, VisualGene};
use crate::components::genetics::{ChromosomeType, GeneExpression, GenePair, GeneVariant};
use crate::components::visual_traits::EyeColor;
use bevy::prelude::*;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::collections::HashMap;

// Struktur zur Beschreibung der Verteilung eines Gens (Mittelwert und Standardabweichung für 0.0-1.0 Skala)
#[derive(Debug, Clone, Copy)]
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
    pub skin_colors: HashMap<String, Vec<(f32, f32, f32)>>,
    pub hair_colors: HashMap<String, Vec<(f32, f32, f32)>>,
    pub eye_colors: HashMap<String, Vec<EyeColor>>, // Wird für initiale Allele genutzt

    // --- Attribut-Merkmale (Verteilungen auf 0.0-1.0 Skala) ---
    // Spezies -> Attribut-Typ -> Verteilung
    pub attribute_distributions: HashMap<String, HashMap<AttributeGene, GeneDistribution>>,
}

// Default Implementierung sollte leer sein, wenn TODO 4 gemacht wird
impl Default for GeneLibrary {
    fn default() -> Self {
        let mut lib = Self {
            skin_colors: HashMap::new(),
            hair_colors: HashMap::new(),
            eye_colors: HashMap::new(),
            attribute_distributions: HashMap::new(),
        };
        // Temporär, bis TODO 4: Fülle die Daten hier.
        // Diese Methoden sollten dann entfernt werden.
        lib.populate_color_palettes();
        lib.populate_attribute_distributions();
        lib
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

    // --- Methoden zur Initialisierung ---

    fn populate_color_palettes(&mut self) {
        // Kopiere hier die langen Farbvektoren aus deinem alten Code rein
        self.skin_colors.insert(
            "Mensch".to_string(),
            vec![
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
        self.skin_colors.insert(
            "Elf".to_string(),
            vec![
                (1.000, 0.922, 0.804),
                (1.000, 0.855, 0.725),
                (0.961, 0.784, 0.569),
                (0.706, 0.784, 0.902),
            ],
        );
        self.skin_colors.insert(
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

        self.hair_colors.insert(
            "Mensch".to_string(),
            vec![
                (0.1, 0.1, 0.1),
                (0.3, 0.2, 0.1),
                (0.6, 0.4, 0.2),
                (0.8, 0.7, 0.3),
                (0.6, 0.3, 0.1),
                (0.8, 0.1, 0.1),
            ],
        );
        self.hair_colors.insert(
            "Elf".to_string(),
            vec![
                (0.9, 0.9, 0.8),
                (0.8, 0.8, 0.6),
                (0.7, 0.6, 0.3),
                (0.3, 0.2, 0.1),
                (0.1, 0.1, 0.1),
            ],
        );
        self.hair_colors.insert(
            "Ork".to_string(),
            vec![
                (0.1, 0.1, 0.1),
                (0.25, 0.15, 0.05),
                (0.4, 0.2, 0.1),
                (0.5, 0.3, 0.2),
            ],
        );

        self.eye_colors.insert(
            "Mensch".to_string(),
            vec![
                EyeColor::Brown,
                EyeColor::Green,
                EyeColor::Blue,
                EyeColor::Gray,
            ],
        );
        self.eye_colors.insert(
            "Elf".to_string(),
            vec![
                EyeColor::Green,
                EyeColor::Blue,
                EyeColor::Gray,
                EyeColor::Brown,
            ],
        );
        self.eye_colors.insert(
            "Ork".to_string(),
            vec![
                EyeColor::Red,
                EyeColor::Black,
                EyeColor::Yellow,
                EyeColor::Gray,
            ],
        );
    }

    fn populate_attribute_distributions(&mut self) {
        // --- Mensch (Alle Werte um 0.5 ± 0.1 bis 0.2) ---
        let mut human_dist = HashMap::new();
        human_dist.insert(
            AttributeGene::Strength,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.1,
            },
        );
        human_dist.insert(
            AttributeGene::Agility,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.1,
            },
        );
        human_dist.insert(
            AttributeGene::Toughness,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.1,
            },
        );
        human_dist.insert(
            AttributeGene::Endurance,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.1,
            },
        );
        human_dist.insert(
            AttributeGene::Recuperation,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.1,
            },
        );
        human_dist.insert(
            AttributeGene::DiseaseResistance,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.1,
            },
        );
        human_dist.insert(
            AttributeGene::Focus,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.15,
            },
        );
        human_dist.insert(
            AttributeGene::Creativity,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.15,
            },
        );
        human_dist.insert(
            AttributeGene::Willpower,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.15,
            },
        );
        human_dist.insert(
            AttributeGene::AnalyticalAbility,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.15,
            },
        );
        human_dist.insert(
            AttributeGene::Intuition,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.15,
            },
        );
        human_dist.insert(
            AttributeGene::Memory,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.15,
            },
        );
        human_dist.insert(
            AttributeGene::Patience,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.15,
            },
        );
        human_dist.insert(
            AttributeGene::SpatialSense,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.15,
            },
        );
        human_dist.insert(
            AttributeGene::Empathy,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.2,
            },
        );
        human_dist.insert(
            AttributeGene::Leadership,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.2,
            },
        );
        human_dist.insert(
            AttributeGene::SocialAwareness,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.2,
            },
        );
        human_dist.insert(
            AttributeGene::LinguisticAbility,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.2,
            },
        );
        human_dist.insert(
            AttributeGene::Negotiation,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.2,
            },
        );
        human_dist.insert(
            AttributeGene::Musicality,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.2,
            },
        );
        self.attribute_distributions
            .insert("Mensch".to_string(), human_dist);

        // --- Elf (Angepasste Mittelwerte/StdAbw auf 0-1 Skala) ---
        let mut elf_dist = HashMap::new();
        elf_dist.insert(
            AttributeGene::Strength,
            GeneDistribution {
                mean: 0.35,
                std_dev: 0.08,
            },
        ); // Unterdurchschnittlich stark
        elf_dist.insert(
            AttributeGene::Agility,
            GeneDistribution {
                mean: 0.7,
                std_dev: 0.1,
            },
        ); // Sehr agil
        elf_dist.insert(
            AttributeGene::Toughness,
            GeneDistribution {
                mean: 0.35,
                std_dev: 0.08,
            },
        ); // Zerbrechlicher
        elf_dist.insert(
            AttributeGene::Endurance,
            GeneDistribution {
                mean: 0.6,
                std_dev: 0.1,
            },
        ); // Gute Ausdauer
        elf_dist.insert(
            AttributeGene::Recuperation,
            GeneDistribution {
                mean: 0.6,
                std_dev: 0.1,
            },
        );
        elf_dist.insert(
            AttributeGene::DiseaseResistance,
            GeneDistribution {
                mean: 0.65,
                std_dev: 0.1,
            },
        );
        elf_dist.insert(
            AttributeGene::Focus,
            GeneDistribution {
                mean: 0.7,
                std_dev: 0.1,
            },
        ); // Sehr fokussiert
        elf_dist.insert(
            AttributeGene::Creativity,
            GeneDistribution {
                mean: 0.65,
                std_dev: 0.15,
            },
        );
        elf_dist.insert(
            AttributeGene::Willpower,
            GeneDistribution {
                mean: 0.6,
                std_dev: 0.15,
            },
        );
        elf_dist.insert(
            AttributeGene::AnalyticalAbility,
            GeneDistribution {
                mean: 0.6,
                std_dev: 0.1,
            },
        );
        elf_dist.insert(
            AttributeGene::Intuition,
            GeneDistribution {
                mean: 0.65,
                std_dev: 0.15,
            },
        );
        elf_dist.insert(
            AttributeGene::Memory,
            GeneDistribution {
                mean: 0.7,
                std_dev: 0.1,
            },
        );
        elf_dist.insert(
            AttributeGene::Patience,
            GeneDistribution {
                mean: 0.7,
                std_dev: 0.1,
            },
        );
        elf_dist.insert(
            AttributeGene::SpatialSense,
            GeneDistribution {
                mean: 0.6,
                std_dev: 0.15,
            },
        );
        elf_dist.insert(
            AttributeGene::Empathy,
            GeneDistribution {
                mean: 0.4,
                std_dev: 0.15,
            },
        ); // Etwas weniger empathisch?
        elf_dist.insert(
            AttributeGene::Leadership,
            GeneDistribution {
                mean: 0.45,
                std_dev: 0.15,
            },
        );
        elf_dist.insert(
            AttributeGene::SocialAwareness,
            GeneDistribution {
                mean: 0.5,
                std_dev: 0.15,
            },
        );
        elf_dist.insert(
            AttributeGene::LinguisticAbility,
            GeneDistribution {
                mean: 0.75,
                std_dev: 0.1,
            },
        ); // Sehr sprachbegabt
        elf_dist.insert(
            AttributeGene::Negotiation,
            GeneDistribution {
                mean: 0.55,
                std_dev: 0.15,
            },
        );
        elf_dist.insert(
            AttributeGene::Musicality,
            GeneDistribution {
                mean: 0.8,
                std_dev: 0.1,
            },
        ); // Sehr musikalisch
        self.attribute_distributions
            .insert("Elf".to_string(), elf_dist);

        // --- Ork (Angepasste Mittelwerte/StdAbw auf 0-1 Skala) ---
        let mut orc_dist = HashMap::new();
        orc_dist.insert(
            AttributeGene::Strength,
            GeneDistribution {
                mean: 0.75,
                std_dev: 0.12,
            },
        ); // Sehr stark
        orc_dist.insert(
            AttributeGene::Agility,
            GeneDistribution {
                mean: 0.3,
                std_dev: 0.1,
            },
        ); // Unbeholfener
        orc_dist.insert(
            AttributeGene::Toughness,
            GeneDistribution {
                mean: 0.8,
                std_dev: 0.1,
            },
        ); // Sehr zäh
        orc_dist.insert(
            AttributeGene::Endurance,
            GeneDistribution {
                mean: 0.7,
                std_dev: 0.1,
            },
        ); // Gute Ausdauer (kämpferisch)
        orc_dist.insert(
            AttributeGene::Recuperation,
            GeneDistribution {
                mean: 0.65,
                std_dev: 0.1,
            },
        );
        orc_dist.insert(
            AttributeGene::DiseaseResistance,
            GeneDistribution {
                mean: 0.75,
                std_dev: 0.1,
            },
        ); // Robust
        orc_dist.insert(
            AttributeGene::Focus,
            GeneDistribution {
                mean: 0.3,
                std_dev: 0.15,
            },
        ); // Wenig Fokus
        orc_dist.insert(
            AttributeGene::Creativity,
            GeneDistribution {
                mean: 0.2,
                std_dev: 0.1,
            },
        ); // Geringe Kreativität
        orc_dist.insert(
            AttributeGene::Willpower,
            GeneDistribution {
                mean: 0.75,
                std_dev: 0.15,
            },
        ); // Hohe Willenskraft
        orc_dist.insert(
            AttributeGene::AnalyticalAbility,
            GeneDistribution {
                mean: 0.25,
                std_dev: 0.1,
            },
        );
        orc_dist.insert(
            AttributeGene::Intuition,
            GeneDistribution {
                mean: 0.4,
                std_dev: 0.15,
            },
        );
        orc_dist.insert(
            AttributeGene::Memory,
            GeneDistribution {
                mean: 0.3,
                std_dev: 0.1,
            },
        );
        orc_dist.insert(
            AttributeGene::Patience,
            GeneDistribution {
                mean: 0.25,
                std_dev: 0.1,
            },
        );
        orc_dist.insert(
            AttributeGene::SpatialSense,
            GeneDistribution {
                mean: 0.4,
                std_dev: 0.1,
            },
        );
        orc_dist.insert(
            AttributeGene::Empathy,
            GeneDistribution {
                mean: 0.2,
                std_dev: 0.1,
            },
        ); // Sehr geringe Empathie
        orc_dist.insert(
            AttributeGene::Leadership,
            GeneDistribution {
                mean: 0.7,
                std_dev: 0.15,
            },
        ); // Führungsstark (auf ihre Art)
        orc_dist.insert(
            AttributeGene::SocialAwareness,
            GeneDistribution {
                mean: 0.3,
                std_dev: 0.1,
            },
        );
        orc_dist.insert(
            AttributeGene::LinguisticAbility,
            GeneDistribution {
                mean: 0.3,
                std_dev: 0.1,
            },
        );
        orc_dist.insert(
            AttributeGene::Negotiation,
            GeneDistribution {
                mean: 0.2,
                std_dev: 0.1,
            },
        ); // Kaum Verhandlung
        orc_dist.insert(
            AttributeGene::Musicality,
            GeneDistribution {
                mean: 0.1,
                std_dev: 0.05,
            },
        ); // Kaum musikalisch
        self.attribute_distributions
            .insert("Ork".to_string(), orc_dist);

        // Füge hier bei Bedarf weitere Spezies hinzu
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
        // <- Signatur geändert
        &self,
        species: &str,
        rng: &mut R, // <- RNG Parameter
    ) -> Option<(GenePair, GenePair, GenePair)> {
        if let Some(colors) = self.skin_colors.get(species) {
            if !colors.is_empty() {
                let color = colors[rng.gen_range(0..colors.len())];
                let (r_val, g_val, b_val) = color;

                // Nutze die Enum-Varianten für create_color_gene_pair
                let gene_r = Self::create_color_gene_pair(VisualGene::SkinColorR, r_val);
                let gene_g = Self::create_color_gene_pair(VisualGene::SkinColorG, g_val);
                let gene_b = Self::create_color_gene_pair(VisualGene::SkinColorB, b_val);
                return Some((gene_r, gene_g, gene_b));
            }
        }
        None
    }

    pub fn create_hair_color_genes<R: Rng + ?Sized>(
        // <- Signatur geändert
        &self,
        species: &str,
        rng: &mut R, // <- RNG Parameter
    ) -> Option<(GenePair, GenePair, GenePair)> {
        if let Some(colors) = self.hair_colors.get(species) {
            if !colors.is_empty() {
                let color = colors[rng.gen_range(0..colors.len())];
                let (r_val, g_val, b_val) = color;

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
