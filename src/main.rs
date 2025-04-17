// src/main.rs
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use forge_of_stories::ui::theme::ThemeAsset;

use forge_of_stories::{
    attributes::AttributesPlugin,         // Dein Attribut-Plugin
    initialization::AppState,             // Wird von lib.rs re-exportiert
    initialization::InitializationPlugin, // <- Haupt-Setup-Plugin
    ui::load_theme,
    ui::UiPlugin,
    SimulationSystemSet,
};

fn main() {
    App::new()
        .add_plugins((
            InitializationPlugin, // Fügt Core, Events, Assets, Debug, State hinzu
            AttributesPlugin,     // Gameplay: Attribute
            UiPlugin,             // Gameplay: UI (Loading, MainMenu, etc.)
            // ... andere Top-Level Gameplay-Plugins ...
            RonAssetPlugin::<ThemeAsset>::new(&["ron"]),
        ))
        // init_resource<UiDebugOptions>() // Entfernt (macht DebugPlugin)
        // init_state::<AppState>() // Entfernt (macht StatePlugin in InitializationPlugin)
        .init_asset::<ThemeAsset>()
        .configure_sets(
            Update,
            (
                SimulationSystemSet::GenotypePhenotype,
                SimulationSystemSet::AttributeApplication,
                SimulationSystemSet::VisualTraitApplication,
                SimulationSystemSet::AttributeCalculation,
            )
                .chain()
                .run_if(in_state(AppState::Running)),
        )
        .add_systems(Startup, load_theme)
        .run();
}
