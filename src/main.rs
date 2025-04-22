// src/main.rs
use bevy::prelude::*;

use bevy_common_assets::ron::RonAssetPlugin;

use forge_of_stories::simulation::SimulationPlugin;
use forge_of_stories::ui::theme::ThemeAsset; // <- Importieren

use forge_of_stories::{
    attributes::AttributesPlugin, initialization::AppState, initialization::InitializationPlugin,
    ui::load_theme, ui::UiPlugin, SimulationSystemSet,
};

fn main() {
    App::new()
        // Aktiviert Hot‑Reload fürs Theme (und alle anderen Assets)
        .add_plugins((
            InitializationPlugin,
            AttributesPlugin,
            UiPlugin,
            SimulationPlugin, // <- HIER HINZUFÜGEN
        ))
        // ... Rest der Konfiguration ...
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
