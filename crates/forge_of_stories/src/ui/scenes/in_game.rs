//! In-Game Scene
//!
//! Main gameplay scene with HUD, world, cameras, and input handling.

mod cameras;
mod hud;
mod input;
mod world;

use crate::GameState;
use crate::utils::cleanup;
use bevy::prelude::*;

/// Main plugin for the in-game scene
pub struct InGameScenePlugin;

impl Plugin for InGameScenePlugin {
    fn build(&self, app: &mut App) {
        app
            // Register all sub-plugins
            .add_plugins((
                hud::InGameHUDPlugin,
                // world::InGameWorldPlugin,
                cameras::InGameCamerasPlugin, // <-- Hier werden die Cameras registriert!
                input::InGameInputPlugin,
            ))
            // Cleanup on exit
            .add_systems(OnExit(GameState::InGame), (cleanup::<hud::InGameHUD>));
    }
}

// use crate::GameState;
// use crate::utils::cleanup;
// use bevy::prelude::*;

// /// Plugin for managing the in-game HUD/UI
// pub struct InGameScenePlugin;

// impl Plugin for InGameScenePlugin {
//     fn build(&self, app: &mut App) {
//         app.add_systems(OnEnter(GameState::InGame), setup_in_game_ui)
//             .add_systems(OnExit(GameState::InGame), cleanup::<InGameUI>);
//     }
// }

// /// Marker component for in-game UI entities (HUD, not the ESC menu)
// #[derive(Component)]
// struct InGameUI;

// fn setup_in_game_ui(mut commands: Commands) {
//     // Spawn in-game HUD
//     let ui_text = "Singleplayer\nPress ESC for menu";

//     commands.spawn((
//         Text::new(ui_text),
//         TextFont {
//             font_size: 20.0,
//             ..default()
//         },
//         TextColor(Color::srgb(0.9, 0.9, 0.9)),
//         Node {
//             position_type: PositionType::Absolute,
//             top: Val::Px(10.0),
//             left: Val::Px(10.0),
//             ..default()
//         },
//         InGameUI,
//         Name::new("InGame HUD"),
//     ));
// }
