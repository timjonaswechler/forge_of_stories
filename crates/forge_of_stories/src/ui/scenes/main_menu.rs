//! Main Menu Scene
//!
//! This module contains all components for the main menu scene:
//! - UI: 2D overlay with title, buttons, and menus
//! - World: 3D background scene with environment and effects
//! - Camera: Camera positioning (delegated to global camera system)
//! - Input: Server connection handling and state transitions
//!
//! The scene-first architecture keeps all related code together,
//! making it easy to understand and maintain the complete scene.

mod camera;
mod input;
mod ui;
mod world;

use crate::GameState;
use crate::ui::components::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use crate::utils::cleanup;
use app::{LOG_CLIENT_HOST, LOG_MAIN};
use bevy::color::palettes::basic::RED;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::InputContextAppExt;

/// Main plugin for the main menu scene
///
/// Coordinates all sub-plugins and handles cleanup on exit.
pub struct MainMenuScenePlugin;

impl Plugin for MainMenuScenePlugin {
    fn build(&self, app: &mut App) {
        app
            // Register all sub-plugins
            .add_plugins((
                ui::MainMenuUIPlugin,
                world::MainMenuWorldPlugin,
                camera::MainMenuCameraPlugin,
                input::MainMenuInputPlugin,
            ))
            // Input context registration
            .add_input_context::<MainMenuContext>()
            // Cleanup all scene entities on exit
            .add_systems(
                OnExit(GameState::MainMenu),
                (
                    cleanup::<ui::MainMenuUI>,
                    cleanup::<world::MainMenuWorld>,
                    cleanup::<MainMenuContext>,
                ),
            );
    }
}

/// Input context marker for main menu
#[derive(Component, Default)]
struct MainMenuContext;

// fn setup_main_menu(mut commands: Commands) {
//     commands
//         .spawn((
//             Node {
//                 width: Val::Percent(100.0),
//                 height: Val::Percent(100.0),
//                 align_items: AlignItems::Center,
//                 justify_content: JustifyContent::Center,
//                 flex_direction: FlexDirection::Column,
//                 row_gap: Val::Px(20.0),
//                 ..default()
//             },
//             MainMenuUI,
//             MainMenuContext,
//         ))
//         .with_children(|parent| {
//             // Title
//             parent.spawn((
//                 Text::new("Forge of Stories"),
//                 TextFont {
//                     font_size: 48.0,
//                     ..default()
//                 },
//                 TextColor(Color::srgb(0.9, 0.9, 0.9)),
//                 Node {
//                     margin: UiRect::bottom(Val::Px(40.0)),
//                     ..default()
//                 },
//             ));

//             // Singleplayer button
//             parent
//                 .spawn((
//                     Button,
//                     Node {
//                         width: Val::Px(250.0),
//                         height: Val::Px(65.0),
//                         border: UiRect::all(Val::Px(5.0)),
//                         justify_content: JustifyContent::Center,
//                         align_items: AlignItems::Center,
//                         ..default()
//                     },
//                     BorderColor::all(Color::WHITE),
//                     BorderRadius::all(Val::Px(10.0)),
//                     BackgroundColor(NORMAL_BUTTON),
//                     MenuAction::Singleplayer,
//                 ))
//                 .with_children(|parent| {
//                     parent.spawn((
//                         Text::new("Singleplayer"),
//                         TextFont {
//                             font_size: 28.0,
//                             ..default()
//                         },
//                         TextColor(Color::srgb(0.9, 0.9, 0.9)),
//                     ));
//                 });
//         });
// }

// fn handle_menu_button_interactions(
//     mut commands: Commands,
//     mut input_focus: ResMut<InputFocus>,
//     mut interaction_query: Query<
//         (
//             Entity,
//             &Interaction,
//             &MenuAction,
//             &mut BackgroundColor,
//             &mut BorderColor,
//         ),
//         Changed<Interaction>,
//     >,
//     mut next_state: ResMut<NextState<GameState>>,
// ) {
//     for (entity, interaction, action, mut color, mut border_color) in &mut interaction_query {
//         match *interaction {
//             Interaction::Pressed => {
//                 input_focus.set(entity);
//                 *color = PRESSED_BUTTON.into();
//                 *border_color = BorderColor::all(RED);

//                 match action {
//                     MenuAction::Singleplayer => {
//                         // Start embedded server
//                         let server =
//                             game_server::ServerHandle::start_embedded(game_server::Port(5000));
//                         commands.insert_resource(server);

//                         info!(target: LOG_CLIENT_HOST, "Server starting... transitioning to ConnectingToServer state");
//                         next_state.set(GameState::ConnectingToServer);
//                     }
//                 }
//             }
//             Interaction::Hovered => {
//                 input_focus.set(entity);
//                 *color = HOVERED_BUTTON.into();
//                 *border_color = BorderColor::all(Color::WHITE);
//             }
//             Interaction::None => {
//                 input_focus.clear();
//                 *color = NORMAL_BUTTON.into();
//                 *border_color = BorderColor::all(Color::BLACK);
//             }
//         }
//     }
// }

// fn wait_for_server_ready(
//     server: Option<Res<game_server::ServerHandle>>,
//     mut next_state: ResMut<NextState<GameState>>,
// ) {
//     if let Some(server) = server {
//         if server.is_ready() {
//             info!(target: LOG_CLIENT_HOST, "Server is ready! Transitioning to InGame state");
//             next_state.set(GameState::InGame);
//         }
//     } else {
//         error!(target: LOG_CLIENT_HOST, "No ServerHandle resource found while waiting for server!");
//     }
// }

// fn log_state_entry(state: Res<State<GameState>>) {
//     info!(target: LOG_MAIN, "Entered state: {:?}", state.get());
// }
