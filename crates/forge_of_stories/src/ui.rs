pub mod normal_vector;

use crate::GameState;
use crate::ServerHandle;
use crate::client::ClientConnectRequest;
use crate::server::start_singleplayer_server;
use crate::utils::cleanup;
use bevy::{color::palettes::basic::RED, input_focus::InputFocus, prelude::*};
use normal_vector::draw_normal_arrows_system;
use tracing::info;

pub struct UIMenuPlugin;

impl Plugin for UIMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputFocus>()
            .init_resource::<InGameMenuState>()
            .add_systems(OnEnter(GameState::Splashscreen), setup_splashscreen)
            .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
            .add_systems(OnEnter(GameState::InGame), render_in_game_ui)
            .add_systems(
                Update,
                main_menu_buttons.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(OnExit(GameState::MainMenu), cleanup::<MainMenuUI>)
            .add_systems(
                Update,
                (
                    toggle_in_game_menu.run_if(in_state(GameState::InGame)),
                    spawn_in_game_menu_ui.run_if(in_state(GameState::InGame)),
                    handle_in_game_menu_buttons.run_if(in_state(GameState::InGame)),
                ),
            )
            .add_systems(OnExit(GameState::InGame), cleanup::<InGameUI>)
            .add_systems(Update, draw_normal_arrows_system)
            .add_systems(OnEnter(GameState::Splashscreen), display_game_state)
            .add_systems(OnEnter(GameState::MainMenu), display_game_state)
            .add_systems(OnEnter(GameState::InGame), display_game_state);
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct SplashscreenUI;

fn setup_splashscreen(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    // Spawn 3D camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    //load Animation of 3D Logo into this scene

    next_state.set(GameState::MainMenu);
}

#[derive(Component)]
struct MainMenuUI;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum MenuAction {
    Singleplayer,
}

#[derive(Component)]
struct InGameUI;

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

fn main_menu_buttons(
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
    existing_server: Option<Res<ServerHandle>>,
) {
    for (entity, interaction, action, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                input_focus.set(entity);
                *color = PRESSED_BUTTON.into();
                *border_color = BorderColor::all(RED);

                match action {
                    MenuAction::Singleplayer => {
                        info!("Singleplayer game requested");
                        if existing_server.is_some() {
                            info!("Server already running, skipping new startup");
                        } else {
                            let handle = start_singleplayer_server();
                            commands.insert_resource(handle);
                        }
                        commands.insert_resource(ClientConnectRequest::singleplayer());
                        next_state.set(GameState::InGame);
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

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum InGameMenuAction {
    Resume,
    LeaveGame,
}

fn spawn_in_game_menu_ui(
    mut commands: Commands,
    menu: Res<InGameMenuState>,
    existing: Query<Entity, With<InGameUI>>,
) {
    if !menu.is_changed() {
        return;
    }

    // Despawn existing menu
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    if !menu.open {
        return;
    }

    // Spawn menu
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
            InGameUI,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(24.0)),
                        row_gap: Val::Px(16.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderRadius::all(Val::Px(12.0)),
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("In-Game Menu"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));

                    // Resume button
                    panel
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(260.0),
                                height: Val::Px(56.0),
                                border: UiRect::all(Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor::all(Color::WHITE),
                            BorderRadius::all(Val::Px(10.0)),
                            BackgroundColor(NORMAL_BUTTON),
                            InGameMenuAction::Resume,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Resume"),
                                TextFont {
                                    font_size: 22.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ));
                        });

                    // Leave game button
                    panel
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(260.0),
                                height: Val::Px(56.0),
                                border: UiRect::all(Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor::all(Color::WHITE),
                            BorderRadius::all(Val::Px(10.0)),
                            BackgroundColor(NORMAL_BUTTON),
                            InGameMenuAction::LeaveGame,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Leave Game"),
                                TextFont {
                                    font_size: 22.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ));
                        });
                });
        });
}

fn render_in_game_ui(mut commands: Commands, server: Option<Res<ServerHandle>>) {
    // Spawn in-game UI
    let ui_text = if server.is_some() {
        "Singleplayer\nPress ESC for menu"
    } else {
        "Connected as client\nPress ESC for menu"
    };

    commands.spawn((
        Text::new(ui_text),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        InGameUI,
    ));
}

fn handle_in_game_menu_buttons(
    mut commands: Commands,
    mut interaction_query: Query<
        (
            &Interaction,
            &InGameMenuAction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    mut menu: ResMut<InGameMenuState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut server_handle: Option<ResMut<ServerHandle>>,
) {
    for (interaction, action, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                *border_color = BorderColor::all(RED);

                match action {
                    InGameMenuAction::Resume => {
                        menu.open = false;
                    }
                    InGameMenuAction::LeaveGame => {
                        info!("Leaving game...");

                        // Shutdown server if hosting
                        if let Some(ref mut server) = server_handle {
                            server.shutdown();
                            commands.remove_resource::<ServerHandle>();
                        }

                        menu.open = false;
                        next_state.set(GameState::MainMenu);
                    }
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                *border_color = BorderColor::all(Color::WHITE);
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                *border_color = BorderColor::all(Color::BLACK);
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct InGameMenuState {
    open: bool,
}

pub fn menu_closed(menu: Res<InGameMenuState>) -> bool {
    !menu.open
}

fn toggle_in_game_menu(keys: Res<ButtonInput<KeyCode>>, mut menu: ResMut<InGameMenuState>) {
    if keys.just_pressed(KeyCode::Escape) {
        menu.open = !menu.open;
    }
}

fn display_game_state(state: Res<State<GameState>>) {
    info!("Aktueller Zustand: {:?}", state.get());
}
