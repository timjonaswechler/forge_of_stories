// src/plugins/setup_plugin.rs
use crate::config::species_gene_data::SpeciesGeneData;
use crate::simulation::resources::gene_library::GeneLibrary;
use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

// AppState Definition
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    Running,
}

// Ressource für Asset Handles
#[derive(Resource)]
struct GeneAssetHandles {
    handles: Vec<Handle<SpeciesGeneData>>,
}

// Plugin Implementation
pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<SpeciesGeneData>::new(&["ron"]))
            .init_state::<AppState>()
            .init_asset::<SpeciesGeneData>()
            .init_resource::<GeneLibrary>()
            .insert_resource(GeneAssetHandles {
                handles: Vec::new(),
            })
            .add_systems(Startup, load_gene_assets)
            .add_systems(
                Update,
                check_assets_loaded.run_if(in_state(AppState::Loading)),
            );
    }
}

// Asset Loading System
fn load_gene_assets(mut gene_handles: ResMut<GeneAssetHandles>, asset_server: Res<AssetServer>) {
    info!("Loading gene assets...");

    gene_handles.handles = vec![
        asset_server.load("genes/human.ron"),
        asset_server.load("genes/elf.ron"),
        asset_server.load("genes/ork.ron"),
    ];
}

// Asset Check System
fn check_assets_loaded(
    mut next_state: ResMut<NextState<AppState>>,
    gene_handles: Res<GeneAssetHandles>,
    asset_server: Res<AssetServer>,
    mut gene_library: ResMut<GeneLibrary>,
    species_assets: Res<Assets<SpeciesGeneData>>,
) {
    if gene_handles.handles.iter().all(|handle| {
        matches!(
            asset_server.get_load_state(handle.id()),
            Some(LoadState::Loaded)
        )
    }) {
        info!("All gene assets loaded!");

        gene_library.skin_colors.clear();
        gene_library.hair_colors.clear();
        gene_library.eye_colors.clear();
        gene_library.attribute_distributions.clear();

        for handle in &gene_handles.handles {
            if let Some(species_data) = species_assets.get(handle) {
                let species_name = &species_data.species_name;
                info!("Processing: {}", species_name);

                gene_library
                    .skin_colors
                    .insert(species_name.clone(), species_data.skin_colors.clone());
                gene_library
                    .hair_colors
                    .insert(species_name.clone(), species_data.hair_colors.clone());
                gene_library
                    .eye_colors
                    .insert(species_name.clone(), species_data.eye_colors.clone());
                gene_library.attribute_distributions.insert(
                    species_name.clone(),
                    species_data.attribute_distributions.clone(),
                );
            } else {
                warn!("Missing data for handle: {:?}", handle.id());
            }
        }

        next_state.set(AppState::Running);
    }
}
