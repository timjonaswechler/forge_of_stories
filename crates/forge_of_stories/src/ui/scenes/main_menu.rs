use crate::GameState;
use crate::ui::components::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use crate::utils::cleanup;
use app::{LOG_CLIENT_HOST, LOG_MAIN};
use bevy::color::palettes::basic::RED;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::InputContextAppExt;

/// Plugin for managing the main menu scene
pub struct MainMenuScenePlugin;

impl Plugin for MainMenuScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::MainMenu),
            (setup_main_menu, log_state_entry),
        )
        .add_systems(
            Update,
            (
                handle_menu_button_interactions,
                wait_for_server_ready.run_if(in_state(GameState::ConnectingToServer)),
            )
                .run_if(in_state(GameState::MainMenu).or(in_state(GameState::ConnectingToServer))),
        )
        .add_systems(OnExit(GameState::MainMenu), cleanup::<MainMenuUI>)
        .add_input_context::<MainMenuContext>();
    }
}

/// Marker component for main menu UI entities
#[derive(Component)]
struct MainMenuUI;

/// Marker component for menu buttons
#[derive(Component)]
struct MainMenuContext;

/// Component identifying menu button actions
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum MenuAction {
    Singleplayer,
}

fn setup_main_menu(mut commands: Commands) {
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
            MainMenuContext,
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
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Singleplayer"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        });
}

fn handle_menu_button_interactions(
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

                match action {
                    MenuAction::Singleplayer => {
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

fn wait_for_server_ready(
    server: Option<Res<game_server::ServerHandle>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Some(server) = server {
        if server.is_ready() {
            info!(target: LOG_CLIENT_HOST, "Server is ready! Transitioning to InGame state");
            next_state.set(GameState::InGame);
        }
    } else {
        error!(target: LOG_CLIENT_HOST, "No ServerHandle resource found while waiting for server!");
    }
}

fn log_state_entry(state: Res<State<GameState>>) {
    info!(target: LOG_MAIN, "Entered state: {:?}", state.get());
}
