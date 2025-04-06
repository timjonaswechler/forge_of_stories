// src/config/gene_data.rs
use crate::genetics::AttributeGene;
use crate::visuals::components::EyeColor;
use bevy::asset::Asset;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Asset, TypePath)]
pub struct SpeciesGeneData {
    pub species_name: String,
    #[serde(default)]
    pub skin_colors: Vec<Color>,
    #[serde(default)]
    pub hair_colors: Vec<Color>,
    #[serde(default)]
    pub eye_colors: Vec<EyeColor>,
    #[serde(default)]
    pub attribute_distributions: HashMap<AttributeGene, GeneDistribution>,
}

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
