// src/initialization/assets/systems.rs
use bevy::{asset::LoadedFolder, prelude::*};
use std::{collections::HashMap, path::Path};

// Import types/resources from our types file
use super::types::{
    AssetLoadingTracker, EssentialAssets, FontsFolderHandle, GameAssets, SpeciesTemplate,
};
// Import necessary types from outside this module
use crate::{attributes::AttributeType, ui::theme::UiTheme, AppState}; // AppState for state transitions

// --- Essential Asset Systems ---

pub fn load_essential_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Requesting essential assets (for loading screen)...");
    // Assuming assets/fonts/Roboto-Regular.ttf exists
    let font = asset_server.load("fonts/Roboto-Regular.ttf");
    let logo = asset_server.load("textures/logo.png");
    commands.insert_resource(EssentialAssets { font, logo });
}

pub fn check_essential_assets_loaded(
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

// --- Game Asset Systems ---

pub fn load_game_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
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
pub fn check_folder_loaded(
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
    }
    // No need for the `else if` block from before, the system just won't do anything
    // if the resource isn't present yet.
}

// System 2: Process the loaded folder *after* check_folder_loaded confirms it's ready
pub fn process_loaded_fonts(
    mut commands: Commands,
    folder_handle: Res<FontsFolderHandle>, // Now we require it, runs only if it exists
    loaded_folders: Res<Assets<LoadedFolder>>,
    // We need GameAssets to store the processed handles
    mut game_assets: ResMut<GameAssets>,
    // We need UiTheme to potentially set the default font
    mut ui_theme: ResMut<UiTheme>,
    mut tracker: ResMut<AssetLoadingTracker>,
) {
    // Only proceed if the folder is loaded but fonts haven't been processed yet
    // The run_if check resource_exists::<FontsFolderHandle> ensures the handle resource exists.
    // We *also* need the tracker flag because this system runs *each frame* if the resource exists,
    // but we only want to process the folder content *once*.
    if !tracker.fonts_folder_loaded || tracker.fonts_processed {
        return;
    }

    info!("Processing loaded fonts folder...");
    // We can unwrap here because fonts_folder_loaded is true, implying the handle is valid
    // and the folder is loaded and accessible via the Assets resource.
    let loaded_folder = loaded_folders.get(&folder_handle.0).unwrap();

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

                // Store in GameAssets
                game_assets
                    .fonts
                    .insert(file_stem.clone(), font_handle.clone());

                // Setze nur dann den Default-Font, wenn es Roboto-Regular ist (or your chosen default)
                if file_stem == "Roboto-Regular" {
                    ui_theme.default_font = Some(font_handle);
                }
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
    info!("Game Asset Check: Fonts processed and stored in GameAssets and UiTheme.");
}

// System 3: Check if game assets (templates, textures) are loaded and update tracker
// This system NO LONGER transitions the state. It only updates the tracker.
pub fn check_all_game_assets_processed(
    asset_server: Res<AssetServer>,
    game_assets: Res<GameAssets>,
    mut tracker: ResMut<AssetLoadingTracker>,
    // Removed `mut next_state`
) {
    // Only update if not already marked as loaded
    if !tracker.species_templates_loaded {
        let templates_loaded = game_assets
            .species_templates
            .iter()
            .all(|handle| asset_server.is_loaded_with_dependencies(handle));

        if templates_loaded {
            info!("Game Asset Check: Species templates loaded.");
            tracker.species_templates_loaded = true;
        }
    }

    if !tracker.textures_loaded {
        let textures_loaded = game_assets
            .textures
            .iter()
            .all(|handle| asset_server.is_loaded_with_dependencies(handle));

        if textures_loaded {
            info!("Game Asset Check: Textures loaded.");
            tracker.textures_loaded = true;
        }
    }

    // NOTE: The state transition to AppState::Running/MainMenu should happen
    // in a separate system that checks if *all* loading is complete (fonts_processed
    // AND species_templates_loaded AND textures_loaded). This makes the
    // loading logic more modular. Let's add a new system for this.
}

// New System: Check if *all* loading is complete and transition state
pub fn check_loading_complete(
    tracker: Res<AssetLoadingTracker>,
    mut next_state: ResMut<NextState<AppState>>,
    // Add any other resources/conditions needed before transitioning
    // For example, maybe you need to wait for UI setup to complete too.
) {
    if tracker.species_templates_loaded
        && tracker.fonts_processed // Check that fonts were processed
        && tracker.textures_loaded
    {
        info!("All game assets processed. Transitioning to AppState::MainMenu (or Running)");
        // Choose the appropriate next state
        next_state.set(AppState::MainMenu);
        // Consider removing the tracker resource here if it's no longer needed
        // commands.remove_resource::<AssetLoadingTracker>(); // Needs `mut commands: Commands`
    }
}
