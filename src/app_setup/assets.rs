// src/app_setup/assets.rs
use crate::{attributes::AttributeType, AppState};
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use std::collections::HashMap;

// Ressource zum Verwalten der Ladefortschritt
#[derive(Resource, Default)]
pub struct AssetLoadingTracker {
    pub species_templates_loaded: bool,
    pub fonts_loaded: bool,
    pub textures_loaded: bool, // Dieser Tracker deckt ALLE Texturen ab
}

// Plugin zum Laden und Initialisieren der Assets
pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<SpeciesTemplate>::new(&["ron"]))
            .init_resource::<AssetLoadingTracker>()
            .add_systems(Startup, load_assets)
            .add_systems(
                Update,
                check_assets_loaded.run_if(in_state(AppState::Loading)),
            );
    }
}

// --- Angepasste Definition an deine RONs ---
#[derive(Asset, TypePath, serde::Deserialize, Debug, Clone)]
pub struct SpeciesTemplate {
    pub species_name: String,
    #[serde(default)]
    // Die Keys in der HashMap müssen zum Enum passen!
    pub attribute_distributions: HashMap<AttributeType, AttributeDistribution>,
}

// --- WICHTIG: Dieses Enum muss #[derive(Deserialize)] haben, um als Key zu funktionieren ---
// (Bereits in attributes/components.rs vorhanden, aber stelle sicher, dass Deserialize da ist)
// use crate::attributes::AttributeType;

#[derive(serde::Deserialize, Debug, Clone, Default)]
pub struct AttributeDistribution {
    pub mean: f32,
    pub std_dev: f32,
}
// -------------------------------------------

// System zum Laden der Assets
fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Requesting assets to load...");
    // Lade genetische Templates (Pfade anpassen, falls sie in 'species' liegen)
    let elf_template: Handle<SpeciesTemplate> = asset_server.load("species/elf.ron");
    let human_template: Handle<SpeciesTemplate> = asset_server.load("species/human.ron");
    let ork_template: Handle<SpeciesTemplate> = asset_server.load("species/ork.ron");

    // Lade Schriften
    let font: Handle<Font> = asset_server.load("fonts/Roboto_Condensed-Medium.ttf"); // Dein Font

    // Lade Texturen
    let loading_icon: Handle<Image> = asset_server.load("textures/loading_icon.png");
    let logo: Handle<Image> = asset_server.load("textures/logo.png"); // Lade das Logo

    // Speichere Handles in einer Ressource für späteren Zugriff
    commands.insert_resource(GameAssets {
        species_templates: vec![elf_template, human_template, ork_template],
        fonts: vec![font],
        // Füge BEIDE Textur-Handles hinzu
        textures: vec![loading_icon, logo.clone()], // Klone den Logo-Handle für die Ressource
        logo: logo, // Speichere den Logo-Handle separat für einfachen Zugriff
    });
}

// System zum Überprüfen, ob alle Assets geladen wurden und Tracker aktualisieren
fn check_assets_loaded(
    asset_server: Res<AssetServer>,
    game_assets: Option<Res<GameAssets>>,
    mut tracker: ResMut<AssetLoadingTracker>,
) {
    let Some(game_assets) = game_assets else {
        return;
    };

    // Überprüfe genetische Templates
    let templates_loaded = game_assets
        .species_templates
        .iter()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle));

    // Überprüfe Schriften
    let fonts_loaded = game_assets
        .fonts
        .iter()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle));

    // Überprüfe ALLE Texturen in der Vec
    let textures_loaded = game_assets
        .textures
        .iter()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle));

    // Tracker Aktualisierung (unverändert)
    if tracker.species_templates_loaded != templates_loaded {
        tracker.species_templates_loaded = templates_loaded;
        if templates_loaded {
            info!("Species templates loaded.");
        }
    }
    if tracker.fonts_loaded != fonts_loaded {
        tracker.fonts_loaded = fonts_loaded;
        if fonts_loaded {
            info!("Fonts loaded.");
        }
    }
    // Wichtig: Dieser Status wird erst true, wenn ALLE Texturen (icon UND logo) geladen sind
    if tracker.textures_loaded != textures_loaded {
        tracker.textures_loaded = textures_loaded;
        if textures_loaded {
            info!("All textures loaded.");
        }
    }
}

// Ressource zum Speichern von Asset-Handles (erweitert um Logo)
#[derive(Resource)]
pub struct GameAssets {
    pub species_templates: Vec<Handle<SpeciesTemplate>>,
    pub fonts: Vec<Handle<Font>>,
    pub textures: Vec<Handle<Image>>, // Liste aller Texturen für den Tracker
    pub logo: Handle<Image>,          // Separater Handle für einfachen Zugriff auf das Logo
}
