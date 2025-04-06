// Neue Datei: src/plugins/core_plugin.rs
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_rand::prelude::{EntropyPlugin, WyRand};

use crate::{FIXED_SEED, USE_FIXED_SEED}; // Importiere Konstanten aus main.rs (oder lagere sie aus)

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Forge of Stories".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin {
                    level: bevy::log::Level::INFO,
                    filter: "wgpu=error,naga=warn,bevy_render=info,bevy_app=info".to_string(),
                    ..default()
                }),
        );

        // --- bevy_rand Plugin hinzufügen ---
        if USE_FIXED_SEED {
            app.add_plugins(EntropyPlugin::<WyRand>::with_seed(FIXED_SEED.to_le_bytes()));
            info!("Using fixed RNG seed: {}", FIXED_SEED);
        } else {
            app.add_plugins(EntropyPlugin::<WyRand>::default());
            info!("Using system entropy for RNG seed.");
        }
    }
}
