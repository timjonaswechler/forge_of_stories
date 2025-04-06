// src/config/gene_data.rs
use crate::components::gene_types::AttributeGene;
use crate::components::visual_traits::EyeColor;
use crate::resources::gene_library::GeneDistribution;
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
