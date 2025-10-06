mod fos_app;
mod server;

use crate::fos_app::FOSApp;
use crate::server::{EmbeddedServer, ServerMode};
use app::AppBuilder;
use bevy::{color::palettes::basic::*, input_focus::InputFocus, log::LogPlugin, prelude::*};
use client::transport::{ClientTransport, ConnectTarget, QuicClientTransport};
use network_shared::{TransportCapabilities, channels::ChannelsConfiguration};

/// Game state tracking where we are in the application flow.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    MainMenu,
    JoinMenu,
    InGame,
}

fn main() {
    let mut app = AppBuilder::<FOSApp>::new(env!("CARGO_PKG_VERSION"))
        .expect("Failed to initialize application")
        .build_with_bevy(|mut app, _ctx| {
            // Insert save path as resource (in current directory for now)
            let save_path = std::env::current_dir()
                .unwrap_or_default()
                .join("world.save.ron");
            app.insert_resource(SavePath(save_path));
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
            .init_resource::<InputFocus>()
            .init_resource::<ServerToClientEntityMap>()
            .init_resource::<JoinMenuState>()
            .init_state::<GameState>()
            .add_systems(OnEnter(GameState::MainMenu), setup)
            .add_systems(Update, button_system.run_if(in_state(GameState::MainMenu)))
            .add_systems(OnExit(GameState::MainMenu), cleanup::<MainMenuUI>)
            .add_systems(OnEnter(GameState::JoinMenu), setup_join_menu)
            .add_systems(
                Update,
                (
                    handle_join_keyboard_input,
                    join_menu_system,
                    update_join_address_display,
                )
                    .run_if(in_state(GameState::JoinMenu)),
            )
            .add_systems(OnExit(GameState::JoinMenu), cleanup::<JoinMenuUI>)
            .add_systems(OnEnter(GameState::InGame), enter_game)
            .add_systems(
                Update,
                (handle_player_input, sync_server_state)
                    .run_if(in_state(GameState::InGame))
                    .run_if(resource_exists::<EmbeddedServer>),
            )
            .add_systems(
                Update,
                (
                    handle_player_input_networked,
                    receive_server_messages,
                    interpolate_remote_players,
                )
                    .run_if(in_state(GameState::InGame))
                    .run_if(resource_exists::<GameClient>),
            )
            .add_systems(
                FixedUpdate,
                crate::server::embedded::tick_embedded_server
                    .run_if(resource_exists::<EmbeddedServer>),
            );
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

fn setup(mut commands: Commands) {
    // Spawn 3D camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Main menu UI
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

            // Host LAN button
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
                    MenuAction::HostLAN,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Host Game (LAN)"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });

            // Join button
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
                    MenuAction::Join,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Join Game"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        });
}

/// Marker for main menu UI (to despawn when entering game)
#[derive(Component)]
struct MainMenuUI;

/// Menu button actions
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum MenuAction {
    Singleplayer,
    HostLAN,
    Join,
}

/// System that runs when entering the InGame state
fn enter_game(
    mut commands: Commands,
    server: Option<Res<EmbeddedServer>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (server_mode_text, entity_count) = if let Some(server) = server.as_ref() {
        info!("Entered game state! Server mode: {:?}", server.mode());
        let count = server.world().entities().len();
        info!("Server world has {} entities", count);
        (Some(server.mode().clone()), count)
    } else {
        info!("Entered game state as client (no server)");
        (None, 0)
    };

    // Spawn light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Spawn ground plane (client-side rendering)
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ClientGroundPlane,
    ));

    // Spawn in-game UI with server info
    let ui_text = if let Some(mode) = server_mode_text {
        match mode {
            ServerMode::Loopback => {
                format!("Singleplayer\nEntities: {}", entity_count)
            }
            ServerMode::Quic { port, .. } => {
                // Get local IP address for LAN hosting
                let local_ip = get_local_ip().unwrap_or_else(|| "Unknown".to_string());
                format!(
                    "Hosting LAN Game\nServer: {}:{}\nEntities: {}\n\nShare this IP with other players!",
                    local_ip, port, entity_count
                )
            }
            #[cfg(feature = "steamworks")]
            ServerMode::Steam { .. } => {
                format!("Steam Multiplayer\nEntities: {}", entity_count)
            }
        }
    } else {
        // Client-only mode (joined someone else's server)
        "Connected to Server\n\nPlaying as client".to_string()
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
    ));
}

/// Marker component for client-side ground plane rendering.
#[derive(Component)]
struct ClientGroundPlane;

/// Resource mapping server entity IDs to client entity IDs.
#[derive(Resource, Default)]
struct ServerToClientEntityMap {
    map: std::collections::HashMap<bevy::ecs::entity::Entity, bevy::ecs::entity::Entity>,
}

/// Client transport resource for networked connections.
#[derive(Resource)]
struct GameClient {
    transport: QuicClientTransport,
    /// Client tick counter for input messages (for lag compensation).
    tick_counter: u64,
}

/// System that syncs server world state to client rendering.
///
/// Runs every frame, reads server entities and spawns/updates corresponding client entities.
fn sync_server_state(
    mut commands: Commands,
    mut server: ResMut<EmbeddedServer>,
    mut entity_map: ResMut<ServerToClientEntityMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut client_transforms: Query<&mut Transform>,
) {
    use server_logic::world::{Player, PlayerShape, Position};

    let server_world = server.world_mut();

    // Sync players
    for (server_entity, (player, position, shape)) in server_world
        .query::<(
            bevy::ecs::entity::Entity,
            (&Player, &Position, &PlayerShape),
        )>()
        .iter(server_world)
    {
        // Check if we already have a client entity for this server entity
        if let Some(&client_entity) = entity_map.map.get(&server_entity) {
            // Update existing client entity position
            if let Ok(mut transform) = client_transforms.get_mut(client_entity) {
                transform.translation = position.translation;
            }
        } else {
            // Spawn new client entity for this player
            let mesh = match shape {
                PlayerShape::Cube => meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                PlayerShape::Sphere => meshes.add(Sphere::new(0.5).mesh().ico(5).unwrap()),
                PlayerShape::Capsule => meshes.add(Capsule3d::new(0.3, 1.0)),
            };

            let client_entity = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(materials.add(player.color)),
                    Transform::from_translation(position.translation),
                    ClientPlayer {
                        server_id: player.id,
                    },
                ))
                .id();

            entity_map.map.insert(server_entity, client_entity);
            info!(
                "Spawned client entity for player {} (shape: {:?})",
                player.id, shape
            );
        }
    }
}

/// Marker component for client-side player rendering.
#[derive(Component)]
struct ClientPlayer {
    server_id: u64,
}

/// Resource holding the save file path.
#[derive(Resource)]
struct SavePath(std::path::PathBuf);

/// System that captures player input and sends it to the server.
fn handle_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut server: ResMut<EmbeddedServer>,
    save_path: Res<SavePath>,
) {
    use server_logic::movement::PlayerInput;

    // Handle save/load hotkeys
    if keyboard.just_pressed(KeyCode::F5) {
        match server.save_world(&save_path.0) {
            Ok(_) => info!("World saved to {:?}", save_path.0),
            Err(e) => error!("Failed to save world: {}", e),
        }
    }

    if keyboard.just_pressed(KeyCode::F9) {
        match server.load_world(&save_path.0) {
            Ok(_) => info!("World loaded from {:?}", save_path.0),
            Err(e) => error!("Failed to load world: {}", e),
        }
    }

    // Calculate movement direction from WASD/Arrow keys
    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    // Normalize diagonal movement
    if direction.length() > 0.01 {
        direction = direction.normalize();
    }

    let jump = keyboard.pressed(KeyCode::Space);

    // Send input to server (player ID 1 for now - the test player)
    let input = PlayerInput { direction, jump };
    server.send_player_input(1, input);
}

/// System that captures player input and sends it to the server over the network.
fn handle_player_input_networked(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut client: ResMut<GameClient>,
) {
    use server_logic::protocol::{GameplayMessage, PlayerInputMessage, channels};

    // Calculate movement direction from WASD/Arrow keys
    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    // Normalize diagonal movement
    if direction.length() > 0.01 {
        direction = direction.normalize();
    }

    // Convert 2D input to 3D movement (Y-up in Bevy)
    let movement = Vec3::new(direction.x, 0.0, direction.y);

    // Create input message
    let message = GameplayMessage::PlayerInput(PlayerInputMessage {
        movement: movement.into(),
        client_tick: client.tick_counter,
    });

    // Increment tick counter
    client.tick_counter += 1;

    // Send to server
    if let Err(e) = client
        .transport
        .send_message_on(channels::PLAYER_INPUT, message)
    {
        warn!("Failed to send player input: {:?}", e);
    }
}

fn button_system(
    mut commands: Commands,
    mut input_focus: ResMut<InputFocus>,
    mut next_state: ResMut<NextState<GameState>>,
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
                        info!("Starting singleplayer game...");
                        match EmbeddedServer::new(ServerMode::Loopback) {
                            Ok(server) => {
                                info!("Embedded server started successfully");
                                commands.insert_resource(server);
                                next_state.set(GameState::InGame);
                            }
                            Err(e) => {
                                error!("Failed to start embedded server: {}", e);
                            }
                        }
                    }
                    MenuAction::HostLAN => {
                        info!("Hosting LAN game...");
                        // TODO: Show server config UI (P6.2)
                        // For now, use default settings
                        match EmbeddedServer::new(ServerMode::Quic {
                            bind_address: "0.0.0.0".to_string(),
                            port: 7777,
                        }) {
                            Ok(server) => {
                                info!("LAN server started on port 7777");
                                commands.insert_resource(server);

                                // Connect client to own server via loopback
                                let channels =
                                    server_logic::protocol::channels::create_gameplay_channels();
                                let capabilities = TransportCapabilities {
                                    supports_reliable_streams: true,
                                    supports_unreliable_streams: false,
                                    supports_datagrams: false,
                                    max_channels: 8,
                                };
                                let mut transport =
                                    QuicClientTransport::new(channels, capabilities);

                                let (event_tx, _event_rx) = tokio::sync::mpsc::unbounded_channel();
                                match transport.connect(
                                    ConnectTarget::Quic {
                                        host: "127.0.0.1".to_string(),
                                        port: 7777,
                                    },
                                    event_tx,
                                ) {
                                    Ok(_) => {
                                        info!("Host client connected to own server");
                                        commands.insert_resource(GameClient {
                                            transport,
                                            tick_counter: 0,
                                        });
                                        next_state.set(GameState::InGame);
                                    }
                                    Err(e) => {
                                        error!("Failed to connect to own server: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to start LAN server: {}", e);
                            }
                        }
                    }
                    MenuAction::Join => {
                        info!("Join game clicked");
                        next_state.set(GameState::JoinMenu);
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

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Get the local IP address for LAN hosting.
fn get_local_ip() -> Option<String> {
    use std::net::{IpAddr, UdpSocket};

    // Try to connect to a public DNS server to determine local IP
    // We don't actually send data, just use the socket to get our local address
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let local_addr = socket.local_addr().ok()?;

    match local_addr.ip() {
        IpAddr::V4(ip) => Some(ip.to_string()),
        IpAddr::V6(ip) => Some(ip.to_string()),
    }
}

// ============================================================================
// JOIN MENU
// ============================================================================

/// Marker for join menu UI elements
#[derive(Component)]
struct JoinMenuUI;

/// Marker for the address display text
#[derive(Component)]
struct AddressDisplayText;

/// Resource to track the join menu input state
#[derive(Resource, Default)]
struct JoinMenuState {
    address: String,
}

/// Join menu button actions
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum JoinMenuAction {
    Connect,
    Back,
}

/// Setup the join menu UI
fn setup_join_menu(mut commands: Commands, mut join_state: ResMut<JoinMenuState>) {
    // Reset the address to default
    join_state.address = "127.0.0.1:7777".to_string();

    // Spawn join menu UI
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            JoinMenuUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Join Game"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // IP:Port input label
            parent.spawn((
                Text::new("Server Address (IP:Port):"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // IP:Port display (editable with keyboard)
            parent.spawn((
                Text::new("127.0.0.1:7777"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Node {
                    padding: UiRect::all(Val::Px(10.0)),
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                BorderRadius::all(Val::Px(5.0)),
                AddressDisplayText,
            ));

            // Info text
            parent.spawn((
                Text::new("Type to edit address, Backspace to delete"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Connect button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                    BorderColor::all(Color::WHITE),
                    BorderRadius::all(Val::Px(10.0)),
                    BackgroundColor(NORMAL_BUTTON),
                    JoinMenuAction::Connect,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Connect"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });

            // Back button
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
                    JoinMenuAction::Back,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Back"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
                });
        });
}

/// Handle join menu button interactions
fn join_menu_system(
    mut commands: Commands,
    mut input_focus: ResMut<InputFocus>,
    mut next_state: ResMut<NextState<GameState>>,
    join_state: Res<JoinMenuState>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &JoinMenuAction,
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
                *border_color = BorderColor::all(GREEN);

                match action {
                    JoinMenuAction::Connect => {
                        info!("Connecting to server at: {}", join_state.address);

                        // Parse IP and port
                        let parts: Vec<&str> = join_state.address.split(':').collect();
                        if parts.len() == 2 {
                            let host = parts[0].to_string();
                            if let Ok(port) = parts[1].parse::<u16>() {
                                // Create QUIC client transport
                                let channels =
                                    server_logic::protocol::channels::create_gameplay_channels();
                                let capabilities = TransportCapabilities {
                                    supports_reliable_streams: true,
                                    supports_unreliable_streams: false,
                                    supports_datagrams: false,
                                    max_channels: 8,
                                };
                                let mut transport =
                                    QuicClientTransport::new(channels, capabilities);

                                let (event_tx, _event_rx) = tokio::sync::mpsc::unbounded_channel();
                                match transport
                                    .connect(ConnectTarget::Quic { host, port }, event_tx)
                                {
                                    Ok(_) => {
                                        info!("Connected to server");
                                        commands.insert_resource(GameClient {
                                            transport,
                                            tick_counter: 0,
                                        });
                                        next_state.set(GameState::InGame);
                                    }
                                    Err(e) => {
                                        error!("Failed to connect to server: {}", e);
                                    }
                                }
                            } else {
                                error!("Invalid port number");
                            }
                        } else {
                            error!("Invalid address format. Use IP:PORT (e.g., 192.168.1.10:7777)");
                        }
                    }
                    JoinMenuAction::Back => {
                        info!("Back to main menu");
                        next_state.set(GameState::MainMenu);
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

/// Handle keyboard input for address editing
fn handle_join_keyboard_input(
    mut join_state: ResMut<JoinMenuState>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // Handle backspace
    if keys.just_pressed(KeyCode::Backspace) {
        join_state.address.pop();
    }

    // Handle number keys (0-9)
    for (key, char) in [
        (KeyCode::Digit0, '0'),
        (KeyCode::Digit1, '1'),
        (KeyCode::Digit2, '2'),
        (KeyCode::Digit3, '3'),
        (KeyCode::Digit4, '4'),
        (KeyCode::Digit5, '5'),
        (KeyCode::Digit6, '6'),
        (KeyCode::Digit7, '7'),
        (KeyCode::Digit8, '8'),
        (KeyCode::Digit9, '9'),
    ] {
        if keys.just_pressed(key) {
            join_state.address.push(char);
        }
    }

    // Handle period (.)
    if keys.just_pressed(KeyCode::Period) {
        join_state.address.push('.');
    }

    // Handle colon (:) - Shift + Semicolon on most keyboards
    if keys.just_pressed(KeyCode::Semicolon)
        && (keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight))
    {
        join_state.address.push(':');
    }
}

/// Update the address display text to match the current state
fn update_join_address_display(
    join_state: Res<JoinMenuState>,
    mut query: Query<&mut Text, With<AddressDisplayText>>,
) {
    if join_state.is_changed() {
        for mut text in &mut query {
            **text = join_state.address.clone();
        }
    }
}

/// Component marking a client-rendered remote player entity.
#[derive(Component)]
struct RemotePlayer {
    player_id: u64,
}

/// Component for interpolating remote player positions smoothly.
#[derive(Component)]
struct NetworkedTransform {
    /// Current interpolated position.
    current: Vec3,
    /// Target position from server.
    target: Vec3,
    /// Current interpolated velocity (for prediction).
    velocity: Vec3,
    /// Interpolation speed (0-1, higher = faster catch-up).
    interpolation_speed: f32,
}

/// System that interpolates remote player positions smoothly.
///
/// Runs every frame to smoothly move remote players toward their target positions.
fn interpolate_remote_players(
    mut query: Query<(&mut Transform, &mut NetworkedTransform), With<RemotePlayer>>,
    time: Res<Time>,
) {
    for (mut transform, mut networked) in &mut query {
        // Interpolate toward target position
        let delta = networked.target - networked.current;
        let distance = delta.length();

        if distance > 0.01 {
            // Use lerp for smooth movement
            let interpolation_amount =
                (networked.interpolation_speed * time.delta_secs() * 60.0).min(1.0);
            networked.current = networked
                .current
                .lerp(networked.target, interpolation_amount);

            // Update visual transform
            transform.translation = networked.current;
        } else {
            // Snap to target if very close
            networked.current = networked.target;
            transform.translation = networked.target;
        }
    }
}

/// System that receives network messages from server and renders remote players.
///
/// Runs for clients connected via QUIC (not loopback).
fn receive_server_messages(
    mut client: Option<ResMut<GameClient>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut remote_players: Query<(Entity, &RemotePlayer, &mut NetworkedTransform)>,
) {
    use server_logic::protocol::{GameplayMessage, PlayerDespawnMessage, PlayerSpawnMessage};

    let Some(client) = client.as_mut() else {
        return; // No client connection
    };

    // Poll for incoming messages
    while let Some((_channel, message)) = client
        .transport
        .receive_message::<GameplayMessage>()
        .ok()
        .flatten()
    {
        debug!("Client received message: {:?}", message);
        match message {
            GameplayMessage::PlayerSpawn(PlayerSpawnMessage {
                player_id,
                color,
                shape,
                position,
            }) => {
                info!("Client received PlayerSpawn for player {}", player_id);

                // Convert serializable types back to Bevy types
                let bevy_color: Color = color.into();
                let bevy_position: Vec3 = position.into();

                // Choose mesh based on shape
                let mesh_handle = match shape {
                    server_logic::world::PlayerShape::Cube => {
                        meshes.add(Cuboid::new(1.0, 1.0, 1.0))
                    }
                    server_logic::world::PlayerShape::Sphere => {
                        meshes.add(Sphere::new(0.5).mesh().ico(5).unwrap())
                    }
                    server_logic::world::PlayerShape::Capsule => {
                        meshes.add(Capsule3d::new(0.4, 1.0))
                    }
                };

                // Spawn visual entity with networked transform for interpolation
                commands.spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(materials.add(bevy_color)),
                    Transform::from_translation(bevy_position),
                    RemotePlayer { player_id },
                    NetworkedTransform {
                        current: bevy_position,
                        target: bevy_position,
                        velocity: Vec3::ZERO,
                        interpolation_speed: 0.15, // Smooth interpolation
                    },
                ));

                info!(
                    "Spawned remote player {} at {:?} with color {:?}",
                    player_id, bevy_position, bevy_color
                );
            }
            GameplayMessage::PlayerDespawn(PlayerDespawnMessage { player_id }) => {
                info!("Client received PlayerDespawn for player {}", player_id);

                // Find and despawn the entity
                for (entity, remote_player, _) in &remote_players {
                    if remote_player.player_id == player_id {
                        commands.entity(entity).despawn();
                        info!("Despawned remote player {}", player_id);
                        break;
                    }
                }
            }
            GameplayMessage::WorldState(state) => {
                // Update target positions for all remote players
                for player_state in state.players {
                    let target_pos: Vec3 = player_state.position.into();
                    let target_vel: Vec3 = player_state.velocity.into();

                    // Find the corresponding remote player entity
                    for (entity, remote_player, mut networked_transform) in
                        remote_players.iter_mut()
                    {
                        if remote_player.player_id == player_state.player_id {
                            networked_transform.target = target_pos;
                            networked_transform.velocity = target_vel;
                            break;
                        }
                    }
                }
            }
            GameplayMessage::PlayerInput(_input) => {
                // Server should never send PlayerInput to client
                warn!("Client received PlayerInput message (unexpected)");
            }
        }
    }
}
