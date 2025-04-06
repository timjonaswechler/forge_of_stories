// src/config/gene_data.rs
use crate::components::gene_types::AttributeGene; // Importiere den Enum
use crate::components::visual_traits::EyeColor;
use crate::resources::gene_library::GeneDistribution;
use bevy::asset::Asset;
use bevy::prelude::*;
use bevy::reflect::TypePath; // Nötig für Asset Trait
use serde::Deserialize; // Für Deserialisierung
use std::collections::HashMap;

// Diese Struktur repräsentiert den Inhalt EINER Spezies-RON-Datei
#[derive(Deserialize, Debug, Asset, TypePath)] // Asset + TypePath hinzufügen
pub struct SpeciesGeneData {
    pub species_name: String, // Name der Spezies (optional, zur Info)
    #[serde(default)] // Wenn nicht vorhanden, leere Map nehmen
    pub skin_colors: Vec<(f32, f32, f32)>,
    #[serde(default)]
    pub hair_colors: Vec<(f32, f32, f32)>,
    #[serde(default)]
    pub eye_colors: Vec<EyeColor>, // Enum direkt deserialisieren
    #[serde(default)]
    pub attribute_distributions: HashMap<AttributeGene, GeneDistribution>, // Enum als Key
}
