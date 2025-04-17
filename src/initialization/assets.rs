// src/initialization/assets.rs
use crate::{attributes::AttributeType, ui::theme::UiTheme, AppState}; // Import UiTheme
use bevy::{asset::LoadedFolder, prelude::*}; // Add AssetPath

use bevy_common_assets::ron::RonAssetPlugin;
use std::collections::HashMap;
use std::path::Path; // Add Path

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
struct FontsFolderHandle(Handle<LoadedFolder>);

#[derive(Resource, Default)]
pub struct AssetLoadingTracker {
    pub species_templates_loaded: bool,
    pub fonts_folder_loaded: bool, // Track the folder itself
    pub fonts_processed: bool,     // Track if we've processed the handles
    pub textures_loaded: bool,
}

#[derive(Asset, TypePath, serde::Deserialize, Debug, Clone)]
pub struct SpeciesTemplate {
    pub species_name: String,
    #[serde(default)]
    pub attribute_distributions: HashMap<AttributeType, AttributeDistribution>,
}

#[derive(serde::Deserialize, Debug, Clone, Default)]
pub struct AttributeDistribution {
    pub mean: f32,
    pub std_dev: f32,
}

// --- Asset Management Logik ---

/// Bündelt die Plugins für das Laden von essentiellen und Spiel-Assets.
pub(super) struct AssetManagementPlugins;

impl Plugin for AssetManagementPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RonAssetPlugin::<SpeciesTemplate>::new(&["ron"]),
            EssentialAssetsPlugin,
            GameAssetsPlugin,
        ));
    }
}

// --- Plugin für essentielle Assets ---
struct EssentialAssetsPlugin;

impl Plugin for EssentialAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Startup), load_essential_assets)
            .add_systems(
                Update,
                check_essential_assets_loaded.run_if(in_state(AppState::Startup)),
            );
    }
}

fn load_essential_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Requesting essential assets (for loading screen)...");
    // Assuming assets/fonts/Roboto-Regular.ttf exists
    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    let logo = asset_server.load("textures/logo.png");
    commands.insert_resource(EssentialAssets { font, logo });
}

fn check_essential_assets_loaded(
    asset_server: Res<AssetServer>,
    essential_assets: Option<Res<EssentialAssets>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if let Some(essential_assets) = essential_assets {
        let font_loaded = asset_server.is_loaded_with_dependencies(&essential_assets.font);
        let logo_loaded = asset_server.is_loaded_with_dependencies(&essential_assets.logo);

        if font_loaded && logo_loaded {
            info!("Essential assets loaded. Transitioning to AppState::Loading");
            next_state.set(AppState::Loading);
        }
    }
}

// --- Plugin für Spiel-Assets ---
struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAssets>()
            .init_resource::<AssetLoadingTracker>()
            .add_systems(OnEnter(AppState::Loading), load_game_assets)
            .add_systems(
                Update,
                (
                    check_folder_loaded, // Check if the folder *itself* is loaded
                    // Process the loaded folder once AFTER it's loaded
                    process_loaded_fonts.run_if(resource_exists::<FontsFolderHandle>),
                )
                    .chain()
                    .run_if(in_state(AppState::Loading)), // Run these in sequence during Loading state
            )
            .add_systems(
                Update,
                check_all_game_assets_processed // New system to check the tracker and transition state
                    .run_if(in_state(AppState::Loading)),
            );
    }
}

fn load_game_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Requesting game assets to load...");

    // --- Load other assets ---
    let elf_template: Handle<SpeciesTemplate> = asset_server.load("species/elf.ron");
    let human_template: Handle<SpeciesTemplate> = asset_server.load("species/human.ron");
    let ork_template: Handle<SpeciesTemplate> = asset_server.load("species/ork.ron");
    let loading_icon: Handle<Image> = asset_server.load("textures/loading_icon.png");

    // --- Load the fonts folder ---
    // Assumes your fonts are in `assets/fonts/`
    let fonts_folder_handle: Handle<LoadedFolder> = asset_server.load_folder("fonts");
    commands.insert_resource(FontsFolderHandle(fonts_folder_handle)); // Store the handle

    // Initialize GameAssets (fonts map starts empty)
    commands.insert_resource(GameAssets {
        species_templates: vec![elf_template, human_template, ork_template],
        fonts: HashMap::new(), // Initialize empty map
        textures: vec![loading_icon],
    });
}

// System 1: Check if the folder handle itself is loaded
fn check_folder_loaded(
    asset_server: Res<AssetServer>,
    folder_handle: Option<Res<FontsFolderHandle>>, // Use Option<> for safety
    mut tracker: ResMut<AssetLoadingTracker>,
) {
    if let Some(handle) = folder_handle {
        // Check if the folder and all its *direct* dependencies (the assets inside) are loaded
        let folder_loaded = asset_server.is_loaded_with_dependencies(&handle.0);

        if !tracker.fonts_folder_loaded && folder_loaded {
            info!("Game Asset Check: Fonts folder loaded.");
            tracker.fonts_folder_loaded = true;
        }
    } else if !tracker.fonts_folder_loaded {
        // This case might happen if the system runs before load_game_assets finishes inserting the resource.
        // It's generally fine, it will just check again next frame.
        // You could add a warning here if it persists unexpectedly.
    }
}

// System 2: Process the loaded folder *after* check_folder_loaded confirms it's ready
fn process_loaded_fonts(
    mut commands: Commands,
    folder_handle: Res<FontsFolderHandle>, // Now we require it, runs only if it exists
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut ui_theme: ResMut<UiTheme>,
    mut tracker: ResMut<AssetLoadingTracker>,
) {
    // Only proceed if the folder is loaded but fonts haven't been processed yet
    if !tracker.fonts_folder_loaded || tracker.fonts_processed {
        return;
    }

    info!("Processing loaded fonts folder...");
    let loaded_folder = loaded_folders.get(&folder_handle.0).unwrap(); // Should be safe due to run_if

    for handle in loaded_folder.handles.iter() {
        // Get the asset path (e.g., "fonts/Roboto-Regular.ttf")
        let asset_path = handle.path().unwrap(); // Assuming path exists
        let path_str = asset_path.path().to_str().unwrap(); // Convert to string

        // Check if it's a font file (e.g., by extension)
        if path_str.ends_with(".ttf") || path_str.ends_with(".otf") {
            // Extract a useful name (e.g., "Roboto-Regular")
            let file_stem = Path::new(path_str)
                .file_stem() // Gets "Roboto-Regular"
                .unwrap_or_default() // Handle potential errors
                .to_string_lossy() // Convert OsStr to String
                .to_string();

            if !file_stem.is_empty() {
                debug!("-> Found font: '{}', handle: {:?}", file_stem, handle.id());
                let font_handle: Handle<Font> = handle.clone().typed();
                // Setze nur dann den Default-Font, wenn es Roboto-Regular ist
                if file_stem == "Roboto-Regular" {
                    ui_theme.default_font = Some(font_handle.clone());
                }
                ui_theme.fonts.insert(file_stem, font_handle);
            } else {
                warn!("Could not extract file stem from font path: {}", path_str);
            }
        } else {
            // Optional: Log if other file types are found in the folder
            debug!("Skipping non-font file in fonts folder: {}", path_str);
        }
    }

    // Mark fonts as processed and remove the temporary handle resource
    tracker.fonts_processed = true;
    commands.remove_resource::<FontsFolderHandle>();
    info!("Game Asset Check: Fonts processed and stored in GameAssets.");
}

// System 3: Check if game assets (templates, textures) are loaded and update tracker
// This system NO LONGER transitions the state. It only updates the tracker.
fn check_all_game_assets_processed(
    asset_server: Res<AssetServer>,
    game_assets: Res<GameAssets>, // Need this to check templates/textures
    mut tracker: ResMut<AssetLoadingTracker>, // Make tracker mutable
                                  // mut next_state: ResMut<NextState<AppState>>, // No longer needed here
) {
    // Only update if not already marked as loaded, to avoid redundant checks/logs
    if !tracker.species_templates_loaded {
        let templates_loaded = game_assets
            .species_templates
            .iter()
            .all(|handle| asset_server.is_loaded_with_dependencies(handle));

        if templates_loaded {
            // info!("Game Asset Check: Species templates loaded."); // Optional Log
            tracker.species_templates_loaded = true;
        }
    }

    if !tracker.textures_loaded {
        let textures_loaded = game_assets
            .textures
            .iter()
            .all(|handle| asset_server.is_loaded_with_dependencies(handle));

        if textures_loaded {
            // info!("Game Asset Check: Textures loaded."); // Optional Log
            tracker.textures_loaded = true;
        }
    }
}
