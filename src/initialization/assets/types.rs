// src/initialization/assets/types.rs
use crate::{attributes::AttributeType, ui::theme::UiTheme}; // Ensure UiTheme and AttributeType are correctly imported from crate root or other modules
use bevy::{asset::LoadedFolder, prelude::*};

use serde::Deserialize;
use std::collections::HashMap;

// --- Asset Typ Definitionen ---
#[derive(Resource)]
pub struct EssentialAssets {
    pub font: Handle<Font>, // Keep your essential font separate if needed for loading screen
    pub logo: Handle<Image>,
}

#[derive(Resource, Default)]
pub struct GameAssets {
    pub species_templates: Vec<Handle<SpeciesTemplate>>,
    // Store loaded fonts by name (e.g., "Roboto-Regular")
    pub fonts: HashMap<String, Handle<Font>>,
    pub textures: Vec<Handle<Image>>,
    // Add other asset types here as needed
}

// Temporary resource to track the folder loading
#[derive(Resource)]
pub struct FontsFolderHandle(pub Handle<LoadedFolder>); // Make pub to access in systems

#[derive(Resource, Default)]
pub struct AssetLoadingTracker {
    pub species_templates_loaded: bool,
    pub fonts_folder_loaded: bool, // Track the folder itself
    pub fonts_processed: bool,     // Track if we've processed the handles
    pub textures_loaded: bool,
}

#[derive(Asset, TypePath, Deserialize, Debug, Clone)] // Use Deserialize directly
pub struct SpeciesTemplate {
    pub species_name: String,
    #[serde(default)]
    pub attribute_distributions: HashMap<AttributeType, AttributeDistribution>,
}

#[derive(Deserialize, Debug, Clone, Default)] // Use Deserialize directly
pub struct AttributeDistribution {
    pub mean: f32,
    pub std_dev: f32,
}
