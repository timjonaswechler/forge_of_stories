mod client;
mod fos_app;
mod server;
mod ui;
mod utils;

use crate::client::*;
use crate::fos_app::FOSApp;
use crate::server::{DEFAULT_PORT, start_singleplayer_server};
use app::AppBuilder;
use bevy::{input_focus::InputFocus, log::LogPlugin, prelude::*};
use game_server::{GameServerState, Player, PlayerInput, Position, ServerHandle, Velocity};
use loopback::{LoopbackBackendPlugins, LoopbackClient};
use networking::prelude::*;
use std::{io, net::SocketAddr};
use ui::UIMenuPlugin;
use utils::cleanup;

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
            // Add UiPlugin
            .add_plugins(UIMenuPlugin)
            // Register replicated components (must match server!)
            .replicate::<Player>()
            .replicate::<Position>()
            .replicate::<Velocity>()
            // Register client messages (must match server!)
            .add_client_message::<PlayerInput>(Channel::Ordered)
            // Game state
            .init_resource::<InputFocus>()
            .init_resource::<ui::InGameMenuState>()
            .init_state::<GameState>()
            .add_message::<SingleplayerRequested>()
            .add_observer(start_singleplayer_host)
            // In-Game
            .add_systems(OnEnter(GameState::InGame), enter_game)
            .add_systems(
                Update,
                (
                    // Input
                    send_player_input
                        .run_if(in_state(GameState::InGame))
                        .run_if(in_state(ClientState::Connected))
                        .run_if(ui::menu_closed),
                    // Rendering
                    spawn_player_meshes
                        .run_if(in_state(GameState::InGame))
                        .after(ClientSystems::Receive),
                    update_player_positions.run_if(in_state(GameState::InGame)),
                    despawn_player_meshes.run_if(in_state(GameState::InGame)),
                    follow_local_player.run_if(in_state(GameState::InGame)),
                    cleanup::<DebugArrows>.run_if(in_state(GameState::InGame)),
                ),
            )
            .add_systems(OnExit(GameState::InGame), cleanup::<InGame>);

            app
        });

    app.run();
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

// ============================================================================
// In-Game
// ============================================================================

#[derive(Component)]
struct InGame;

fn enter_game(
    mut commands: Commands,
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
        InGame,
    ));

    // Spawn ground plane (client-side rendering only)
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        InGame,
    ));
}
