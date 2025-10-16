mod client;
mod fos_app;
mod server;

use crate::client::*;
use crate::fos_app::FOSApp;
use crate::server::{DEFAULT_PORT, start_singleplayer_server};
use app::AppBuilder;
use bevy::{color::palettes::basic::*, input_focus::InputFocus, log::LogPlugin, prelude::*};
use game_server::{GameServerState, Player, PlayerInput, Position, ServerHandle, Velocity};
use loopback::{LoopbackBackendPlugins, LoopbackClient};
use networking::prelude::*;
use std::{io, net::SocketAddr};

/// Game state tracking where we are in the application flow.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Splashscreen,
    MainMenu,
    ConnectingSingleplayer,
    InGame,
}

fn main() {
    let mut app = AppBuilder::<FOSApp>::new(env!("CARGO_PKG_VERSION"))
        .expect("Failed to initialize application")
        .build_with_bevy(|mut app, _ctx| {
            app.add_plugins(
                DefaultPlugins
                    .build()
                    .disable::<LogPlugin>()
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            title: "Forge of Stories".to_string(),
                            ..default()
                        }),
                        ..default()
                    }),
            )
            // Add bevy_replicon networking
            .add_plugins(RepliconPlugins)
            .add_plugins(LoopbackBackendPlugins)
            // Register replicated components (must match server!)
            .replicate::<Player>()
            .replicate::<Position>()
            .replicate::<Velocity>()
            // Register client messages (must match server!)
            .add_client_message::<PlayerInput>(Channel::Ordered)
            // Game state
            .init_resource::<InputFocus>()
            .init_resource::<InGameMenuState>()
            .init_state::<GameState>()
            .add_message::<SingleplayerRequested>()
            .add_observer(start_singleplayer_host)
            // Splash & Menu
            .add_systems(OnEnter(GameState::Splashscreen), setup_splashscreen)
            .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
            .add_systems(
                Update,
                main_menu_buttons.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(OnExit(GameState::MainMenu), cleanup::<MainMenuUI>)
            .add_systems(
                Update,
                poll_loopback_connection.run_if(in_state(GameState::ConnectingSingleplayer)),
            )
            .add_systems(
                OnExit(GameState::ConnectingSingleplayer),
                cleanup_pending_connection,
            )
            // In-Game
            .add_systems(OnEnter(GameState::InGame), enter_game)
            .add_systems(
                Update,
                (
                    // Input
                    send_player_input
                        .run_if(in_state(GameState::InGame))
                        .run_if(in_state(ClientState::Connected))
                        .run_if(menu_closed),
                    // Rendering
                    spawn_player_meshes
                        .run_if(in_state(GameState::InGame))
                        .after(ClientSystems::Receive),
                    update_player_positions.run_if(in_state(GameState::InGame)),
                    despawn_player_meshes.run_if(in_state(GameState::InGame)),
                    follow_local_player.run_if(in_state(GameState::InGame)),
                    // Menu
                    toggle_in_game_menu.run_if(in_state(GameState::InGame)),
                    spawn_in_game_menu_ui.run_if(in_state(GameState::InGame)),
                    handle_in_game_menu_buttons.run_if(in_state(GameState::InGame)),
                ),
            )
            .add_systems(OnExit(GameState::InGame), cleanup::<InGameCleanup>);

            app
        });

    app.run();
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ============================================================================
// Splashscreen & Main Menu
// ============================================================================

fn setup_splashscreen(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    // Spawn 3D camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    next_state.set(GameState::MainMenu);
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

#[derive(Component)]
struct MainMenuUI;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum MenuAction {
    Singleplayer,
}

#[derive(Message, Event)]
struct SingleplayerRequested;

#[derive(Resource)]
struct PendingConnection {
    addr: SocketAddr,
    retry: Timer,
    attempts: u32,
}

impl PendingConnection {
    fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            retry: Timer::from_seconds(0.25, TimerMode::Repeating),
            attempts: 0,
        }
    }
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
                        commands.trigger(SingleplayerRequested);
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

fn start_singleplayer_host(
    _request: On<SingleplayerRequested>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    existing_handle: Option<Res<ServerHandle>>,
) {
    if existing_handle.is_none() {
        info!("Starting embedded server…");
        let server_handle = start_singleplayer_server();
        commands.insert_resource(server_handle);
    } else {
        debug!("Reusing existing embedded server handle");
    }

    commands.remove_resource::<LoopbackClient>();
    commands.remove_resource::<PendingConnection>();
    commands.insert_resource(PendingConnection::new(SocketAddr::from((
        [127, 0, 0, 1],
        DEFAULT_PORT,
    ))));

    next_state.set(GameState::ConnectingSingleplayer);
}

fn poll_loopback_connection(
    mut commands: Commands,
    time: Res<Time>,
    pending_connection: Option<ResMut<PendingConnection>>,
    server_handle: Option<Res<ServerHandle>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(mut pending) = pending_connection else {
        return;
    };

    let Some(handle) = server_handle else {
        warn!("Server handle missing while establishing singleplayer connection");
        commands.remove_resource::<PendingConnection>();
        next_state.set(GameState::MainMenu);
        return;
    };

    if !pending.retry.tick(time.delta()).just_finished() {
        return;
    }

    let server_state = handle.state();

    match server_state {
        GameServerState::Running | GameServerState::Paused => {
            pending.attempts += 1;
            match LoopbackClient::new(pending.addr) {
                Ok(client) => {
                    info!(
                        "Loopback client connected to embedded server after {} attempt(s)",
                        pending.attempts
                    );
                    commands.insert_resource(client);
                    commands.remove_resource::<PendingConnection>();
                    next_state.set(GameState::InGame);
                }
                Err(err) if err.kind() == io::ErrorKind::ConnectionRefused => {
                    debug!("Embedded server not accepting connections yet, retrying…");
                }
                Err(err) => {
                    error!("Failed to connect loopback client: {err}");
                    commands.remove_resource::<PendingConnection>();
                    commands.remove_resource::<ServerHandle>();
                    next_state.set(GameState::MainMenu);
                }
            }
        }
        GameServerState::Starting => {
            debug!("Embedded server still starting, waiting before connecting…");
        }
        GameServerState::ShuttingDown | GameServerState::Stopped => {
            error!(
                "Embedded server stopped while waiting for loopback connection (state: {:?})",
                server_state
            );
            commands.remove_resource::<PendingConnection>();
            commands.remove_resource::<ServerHandle>();
            next_state.set(GameState::MainMenu);
        }
    }
}

fn cleanup_pending_connection(mut commands: Commands) {
    commands.remove_resource::<PendingConnection>();
}

// ============================================================================
// In-Game
// ============================================================================

fn enter_game(
    mut commands: Commands,
    server: Option<Res<ServerHandle>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Entering game...");

    // Spawn light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        InGameCleanup,
    ));

    // Spawn ground plane (client-side rendering only)
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        InGameCleanup,
    ));

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
        InGameCleanup,
    ));
}

#[derive(Component)]
struct InGameCleanup;

#[derive(Resource, Default)]
struct InGameMenuState {
    open: bool,
}

fn menu_closed(menu: Res<InGameMenuState>) -> bool {
    !menu.open
}

fn toggle_in_game_menu(keys: Res<ButtonInput<KeyCode>>, mut menu: ResMut<InGameMenuState>) {
    if keys.just_pressed(KeyCode::Escape) {
        menu.open = !menu.open;
    }
}

#[derive(Component)]
struct InGameMenuUI;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum InGameMenuAction {
    Resume,
    LeaveGame,
}

fn spawn_in_game_menu_ui(
    mut commands: Commands,
    menu: Res<InGameMenuState>,
    existing: Query<Entity, With<InGameMenuUI>>,
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
            InGameMenuUI,
            InGameCleanup,
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
    server_handle: Option<Res<ServerHandle>>,
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
                        if let Some(ref server) = server_handle {
                            server.shutdown();
                            commands.remove_resource::<ServerHandle>();
                        }

                        // Disconnect client
                        commands.remove_resource::<LoopbackClient>();

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

fn cleanup_in_game(
    mut commands: Commands,
    mut menu: ResMut<InGameMenuState>,
    cleanup_entities: Query<Entity, With<InGameCleanup>>,
) {
    menu.open = false;

    for entity in &cleanup_entities {
        commands.entity(entity).despawn();
    }
}
