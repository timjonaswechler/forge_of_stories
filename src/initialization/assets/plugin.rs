// src/initialization/assets/plugin.rs
use bevy::{asset::LoadedFolder, prelude::*};
use bevy_common_assets::ron::RonAssetPlugin;

// Import systems and types from within the assets module
use super::{
    systems::{
        check_all_game_assets_processed, check_essential_assets_loaded, check_folder_loaded,
        check_loading_complete, load_essential_assets, load_game_assets, process_loaded_fonts,
    },
    types::{AssetLoadingTracker, EssentialAssets, FontsFolderHandle, GameAssets, SpeciesTemplate},
};

// Import AppState from the parent module's exports
use crate::AppState;

/// Bündelt die Plugins für das Laden von essentiellen und Spiel-Assets.
pub struct AssetManagementPlugins;

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
struct EssentialAssetsPlugin; // Keep internal to this module

impl Plugin for EssentialAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Startup), load_essential_assets)
            .add_systems(
                Update,
                check_essential_assets_loaded.run_if(in_state(AppState::Startup)),
            );
    }
}

// --- Plugin für Spiel-Assets ---
struct GameAssetsPlugin; // Keep internal to this module

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app // Register necessary types/resources
            .init_resource::<GameAssets>()
            .init_resource::<AssetLoadingTracker>()
            // Add systems for loading and checking progress
            .add_systems(OnEnter(AppState::Loading), load_game_assets)
            .add_systems(
                Update,
                (
                    // Check folder loaded (updates tracker)
                    check_folder_loaded,
                    // Process folder contents *only* if folder handle resource exists
                    // and it's marked as loaded by check_folder_loaded.
                    process_loaded_fonts
                        .run_if(resource_exists::<FontsFolderHandle>)
                        .run_if(|tracker: Res<AssetLoadingTracker>| tracker.fonts_folder_loaded),
                    // Check other assets (updates tracker)
                    check_all_game_assets_processed,
                    // Check if *all* loading is complete and transition state
                    check_loading_complete,
                )
                    .chain() // Chain the systems to run in this order
                    .run_if(in_state(AppState::Loading)), // Only run this chain in Loading state
            );
    }
}
