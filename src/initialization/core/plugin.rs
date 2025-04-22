// src/initialization/core/plugin.rs
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_rand::prelude::{EntropyPlugin, WyRand};

use crate::{FIXED_SEED, USE_FIXED_SEED}; // Assuming these are defined at crate root

use super::systems::setup_camera; // Import the system

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..default()
                })
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

        // --- Kamera hinzufügen ---
        // Füge ein Startup-System hinzu, das die Kamera spawnt.
        // Die IsDefaultUiCamera Komponente stellt sicher, dass die UI an diese Kamera gebunden wird.
        app.add_systems(Startup, setup_camera);
    }
}
