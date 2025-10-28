//! Main Menu 2D UI Layer
//!
//! Contains all 2D UI elements for the main menu (title, buttons, panels).

use crate::GameState;
use crate::ui::components::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use app::LOG_CLIENT_HOST;
use bevy::color::palettes::basic::RED;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;

/// Plugin for main menu UI elements
pub(super) struct MainMenuUIPlugin;

impl Plugin for MainMenuUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), spawn_ui)
            .add_systems(
                Update,
                handle_button_interactions.run_if(in_state(GameState::MainMenu)),
            );
    }
}

/// Marker component for main menu UI entities
#[derive(Component)]
pub(super) struct MainMenuUI;

/// Component identifying menu button actions
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Singleplayer,
    // Future: Multiplayer, Settings, Quit
}

/// Event sent when a menu action is triggered
#[derive(Event)]
pub struct MenuActionEvent {
    pub action: MenuAction,
}

/// Spawns the main menu UI (title, buttons)
fn spawn_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            MainMenuUI,
            Name::new("Main Menu UI Root"),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Forge of Stories"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Singleplayer button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(Color::WHITE),
                    BorderRadius::all(Val::Px(10.0)),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuAction::Singleplayer,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Singleplayer"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
            // Future buttons:
            // spawn_menu_button(parent, "Multiplayer", MenuAction::Multiplayer);
            // spawn_menu_button(parent, "Settings", MenuAction::Settings);
            // spawn_menu_button(parent, "Quit", MenuAction::Quit);
        });
}

/// Handles button interactions (hover, press) and triggers menu actions
fn handle_button_interactions(
    mut commands: Commands,
    mut input_focus: ResMut<InputFocus>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &MenuAction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (entity, interaction, action, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                input_focus.set(entity);
                *color = PRESSED_BUTTON.into();
                *border_color = BorderColor::all(RED);

                // Handle the button action
                match action {
                    MenuAction::Singleplayer => {
                        info!(target: LOG_CLIENT_HOST, "Singleplayer button pressed");

                        // Start embedded server
                        let server =
                            game_server::ServerHandle::start_embedded(game_server::Port(5000));
                        commands.insert_resource(server);

                        info!(target: LOG_CLIENT_HOST, "Server starting... transitioning to ConnectingToServer state");
                        next_state.set(GameState::ConnectingToServer);
                    }
                }
            }
            Interaction::Hovered => {
                input_focus.set(entity);
                *color = HOVERED_BUTTON.into();
                *border_color = BorderColor::all(Color::WHITE);
            }
            Interaction::None => {
                input_focus.clear();
                *color = NORMAL_BUTTON.into();
                *border_color = BorderColor::all(Color::BLACK);
            }
        }
    }
}
