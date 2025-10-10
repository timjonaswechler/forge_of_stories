mod fos_app;
mod server;

use crate::fos_app::FOSApp;
use crate::server::{
    LoopbackClient, create_default_quic_transport, start_multiplayer_server,
    start_singleplayer_server,
};
use app::AppBuilder;
use bevy::{color::palettes::basic::*, input_focus::InputFocus, log::LogPlugin, prelude::*};
use client::transport::{ClientTransport, ConnectTarget, QuicClientTransport};
use game_server::ServerHandle;
use shared::TransportCapabilities;
use std::cell::BorrowError;
use std::collections::HashSet;
use uuid::Uuid;

/// Game state tracking where we are in the application flow.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Slapshscreen,
    MainMenu,
    JoinMenu,
    InGame,
}

fn main() {
    let mut app = AppBuilder::<FOSApp>::new(env!("CARGO_PKG_VERSION"))
        .expect("Failed to initialize application")
        .build_with_bevy(|mut app, ctx| {
            // Insert save path as resource (in current directory for now)
            let save_path = ctx.path_context().saves_dir().join("world.save.ron");
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
            .init_resource::<RemotePlayerRegistry>()
            .init_resource::<JoinMenuState>()
            .init_resource::<InGameMenuState>()
            .init_state::<GameState>()
            .add_systems(OnEnter(GameState::Slapshscreen), setup_slapshscreen)
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
                toggle_in_game_menu.run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                spawn_in_game_menu_ui.run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                handle_in_game_menu_buttons.run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                handle_player_input_networked
                    .run_if(in_state(GameState::InGame))
                    .run_if(resource_exists::<GameClient>)
                    .run_if(menu_closed),
            )
            .add_systems(
                Update,
                receive_server_messages
                    .run_if(in_state(GameState::InGame))
                    .run_if(resource_exists::<GameClient>),
            )
            .add_systems(
                Update,
                receive_loopback_messages
                    .run_if(in_state(GameState::InGame))
                    .run_if(resource_exists::<LoopbackClient>),
            )
            .add_systems(
                Update,
                interpolate_remote_players.run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnExit(GameState::InGame), cleanup_in_game_entities);
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

fn setup_slapshscreen(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    // Spawn 3D camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    )); // Spawn 3D camera
    next_state.set(GameState::MainMenu);
}

fn setup(mut commands: Commands) {
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
    server: Option<Res<ServerHandle>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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

    // Spawn ground plane (client-side rendering)
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ClientGroundPlane,
        InGameCleanup,
    ));

    // Spawn in-game UI with server info
    let ui_text = match server {
        Some(server) => {
            format!("Game Running\nServer: {:?}", server.mode_info())
        }
        None => {
            // Client-only mode (joined someone else's server)
            "Connected to Server\n\nPlaying as client".to_string()
        }
    };

    commands.insert_resource(RemotePlayerRegistry::default());

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

#[derive(Resource, Default)]
struct RemotePlayerRegistry {
    known_players: HashSet<Uuid>,
}

#[derive(Resource, Default)]
struct InGameMenuState {
    open: bool,
    external_open: bool,
}

#[derive(Component)]
struct InGameCleanup;

#[derive(Component)]
struct InGameMenuUI;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum InGameMenuButtonAction {
    Resume,
    LeaveSession,
    OpenLan,
    CloseLan,
}

/// Marker component for client-side player rendering.
#[derive(Component)]
struct ClientPlayer {
    server_id: shared::ClientId,
}

/// Resource holding the save file path.
#[derive(Resource)]
struct SavePath(std::path::PathBuf);

/// System that captures player input and sends it to the server (singleplayer).
/// Handles player input for non-host players.
///
/// NOTE: This system is currently disabled because we need to refactor it
/// to send input via the appropriate client transport.
///
/// TODO: Refactor to use client transport for player input
#[allow(dead_code)]
fn handle_player_input(_keyboard: Res<ButtonInput<KeyCode>>, _save_path: Res<SavePath>) {
    // NOTE: This code is disabled and needs refactoring to send input via client transport
    /*
    use game_server::movement::PlayerInput;

    // Handle save/load hotkeys
    if keyboard.just_pressed(KeyCode::F5) {
        // TODO: Implement save via server command
    }

    if keyboard.just_pressed(KeyCode::F9) {
        // TODO: Implement load via server command
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

    // Send input to server (singleplayer also uses HOST_CLIENT_ID)
    let input = PlayerInput { direction, jump };
    server.send_player_input(game_server::HOST_CLIENT_ID, input);
    */
}

/// System that captures player input and sends it to the server over the network.
fn handle_player_input_networked(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut client: ResMut<GameClient>,
) {
    use game_server::protocol::{GameplayMessage, PlayerInputMessage, channels};

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
                        let (server_handle, loopback_client) = start_singleplayer_server();
                        info!("Embedded server started successfully in separate thread");
                        commands.insert_resource(server_handle);
                        commands.insert_resource(loopback_client);
                        next_state.set(GameState::InGame);
                    }
                    MenuAction::HostLAN => {
                        info!("Hosting LAN game...");
                        // TODO: Show server config UI (P6.2)
                        // For now, use default settings
                        match start_multiplayer_server("0.0.0.0:7777") {
                            Ok((server_handle, loopback_client)) => {
                                info!("LAN server started in separate thread");
                                commands.insert_resource(server_handle);
                                commands.insert_resource(loopback_client);
                                next_state.set(GameState::InGame);
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
                                    game_server::protocol::channels::create_gameplay_channels();
                                let capabilities = TransportCapabilities {
                                    supports_reliable_streams: true,
                                    supports_unreliable_streams: false,
                                    supports_datagrams: false,
                                    max_channels: 8,
                                };
                                let mut transport =
                                    QuicClientTransport::new(channels, capabilities);

                                match transport.connect(ConnectTarget::Quic { host, port }) {
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
                                        *border_color = BorderColor::all(RED);
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
    player_id: Uuid,
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

fn apply_gameplay_message(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    remote_players: &mut Query<(Entity, &RemotePlayer, &mut NetworkedTransform)>,
    registry: &mut RemotePlayerRegistry,
    message: game_server::protocol::GameplayMessage,
) {
    use game_server::protocol::{PlayerDespawnMessage, PlayerSpawnMessage};

    match message {
        game_server::protocol::GameplayMessage::PlayerSpawn(PlayerSpawnMessage {
            player_id,
            color,
            shape,
            position,
        }) => {
            info!("Client received PlayerSpawn for player {}", player_id);

            let bevy_color: Color = color.into();
            let bevy_position: Vec3 = position.into();
            let mesh_handle = match shape {
                game_server::protocol::PlayerShape::Cube => meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                game_server::protocol::PlayerShape::Capsule => meshes.add(Capsule3d::new(0.5, 1.0)),
            };
            let material_handle = materials.add(bevy_color);
            let transform = Transform::from_translation(bevy_position);

            let is_new = registry.known_players.insert(player_id);

            if !is_new {
                let mut updated_existing = false;
                for (entity, remote_player, mut networked_transform) in remote_players.iter_mut() {
                    if remote_player.player_id == player_id {
                        networked_transform.current = bevy_position;
                        networked_transform.target = bevy_position;
                        networked_transform.velocity = Vec3::ZERO;

                        commands.entity(entity).insert((
                            Mesh3d(mesh_handle.clone()),
                            MeshMaterial3d(material_handle.clone()),
                            transform,
                        ));

                        info!("Updated remote player {} from spawn message", player_id);
                        updated_existing = true;
                        break;
                    }
                }

                if updated_existing {
                    return;
                }
            }

            commands.spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(material_handle),
                transform,
                RemotePlayer { player_id },
                NetworkedTransform {
                    current: bevy_position,
                    target: bevy_position,
                    velocity: Vec3::ZERO,
                    interpolation_speed: 0.15,
                },
                InGameCleanup,
            ));

            info!(
                "Spawned remote player {} at {:?} with color {:?}",
                player_id, bevy_position, bevy_color
            );
        }
        game_server::protocol::GameplayMessage::PlayerDespawn(PlayerDespawnMessage {
            player_id,
        }) => {
            info!("Client received PlayerDespawn for player {}", player_id);

            for (entity, remote_player, _) in remote_players.iter() {
                if remote_player.player_id == player_id {
                    commands.entity(entity).despawn();
                    info!("Despawned remote player {}", player_id);
                    break;
                }
            }

            registry.known_players.remove(&player_id);
        }
        game_server::protocol::GameplayMessage::WorldState(state) => {
            for player_state in state.players {
                let target_pos: Vec3 = player_state.position.into();
                let target_vel: Vec3 = player_state.velocity.into();

                let mut found = false;
                for (_entity, remote_player, mut networked_transform) in remote_players.iter_mut() {
                    if remote_player.player_id == player_state.player_id {
                        networked_transform.target = target_pos;
                        networked_transform.velocity = target_vel;
                        found = true;
                        break;
                    }
                }

                if found {
                    continue;
                }

                if !registry.known_players.insert(player_state.player_id) {
                    // Await the actual entity spawn (already queued earlier).
                    continue;
                }

                info!(
                    "Spawning player {} from WorldState (missed spawn message)",
                    player_state.player_id
                );

                let mesh_handle = meshes.add(Capsule3d::new(0.4, 1.0));
                let default_color = Color::srgb(0.8, 0.8, 0.8);

                commands.spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(materials.add(default_color)),
                    Transform::from_translation(target_pos),
                    RemotePlayer {
                        player_id: player_state.player_id,
                    },
                    NetworkedTransform {
                        current: target_pos,
                        target: target_pos,
                        velocity: target_vel,
                        interpolation_speed: 0.15,
                    },
                    InGameCleanup,
                ));
            }
        }
        game_server::protocol::GameplayMessage::PlayerInput(_) => {
            warn!("Client received PlayerInput message (unexpected)");
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
    mut registry: ResMut<RemotePlayerRegistry>,
    mut remote_players: Query<(Entity, &RemotePlayer, &mut NetworkedTransform)>,
) {
    use game_server::protocol::GameplayMessage;

    let Some(client) = client.as_mut() else {
        return; // No client connection
    };

    // Poll for incoming messages
    loop {
        match client.transport.receive_message::<GameplayMessage>() {
            Ok(Some((channel, message))) => {
                debug!(
                    "Client received message on channel {}: {:?}",
                    channel, message
                );

                apply_gameplay_message(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut remote_players,
                    registry.as_mut(),
                    message,
                );
            }
            Ok(None) => {
                // No more messages
                break;
            }
            Err(e) => {
                warn!("Error receiving message: {:?}", e);
                break;
            }
        }
    }
}

fn receive_loopback_messages(
    loopback: Option<ResMut<LoopbackClient>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut registry: ResMut<RemotePlayerRegistry>,
    mut remote_players: Query<(Entity, &RemotePlayer, &mut NetworkedTransform)>,
) {
    use game_server::protocol::GameplayMessage;
    use shared::{ClientEvent, TransportError};

    let Some(mut loopback) = loopback else {
        return;
    };

    let transport = &mut loopback.0;

    let mut direct_messages = Vec::new();
    match transport.poll_direct::<GameplayMessage>(&mut direct_messages) {
        Ok(()) => {}
        Err(TransportError::NotReady) => {
            // Loopback not connected yet; skip
        }
        Err(e) => {
            warn!("Failed to poll loopback direct messages: {:?}", e);
        }
    }

    for (channel, message) in direct_messages {
        debug!(
            "Loopback client received direct message on channel {}: {:?}",
            channel, message
        );

        apply_gameplay_message(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut remote_players,
            registry.as_mut(),
            message,
        );
    }

    let mut events = Vec::new();
    transport.poll_events(&mut events);

    for event in events {
        match event {
            ClientEvent::Message { channel, payload } => {
                match bincode::serde::decode_from_slice::<GameplayMessage, _>(
                    &payload,
                    bincode::config::standard(),
                ) {
                    Ok((message, _)) => {
                        debug!(
                            "Loopback client received serialized message on channel {}: {:?}",
                            channel, message
                        );
                        apply_gameplay_message(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            &mut remote_players,
                            registry.as_mut(),
                            message,
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Failed to deserialize loopback message on channel {}: {:?}",
                            channel, e
                        );
                    }
                }
            }
            ClientEvent::Datagram { payload } => {
                debug!(
                    "Loopback client received datagram ({} bytes) - ignoring",
                    payload.len()
                );
            }
            ClientEvent::Connected { .. } => {
                debug!("Loopback client connected");
            }
            ClientEvent::Disconnected { reason } => {
                info!("Loopback client disconnected: {:?}", reason);
            }
            ClientEvent::Error { error } => {
                warn!("Loopback client error: {:?}", error);
            }
            ClientEvent::Discovery(event) => {
                debug!("Loopback client discovery event ignored: {:?}", event);
            }
            ClientEvent::AuthResult {
                client,
                steam_id,
                owner_steam_id,
                result,
            } => {
                debug!(
                    "Loopback client auth result ignored: client={:?} steam_id={} owner={} result={:?}",
                    client, steam_id, owner_steam_id, result
                );
            }
        }
    }
}

fn toggle_in_game_menu(keys: Res<ButtonInput<KeyCode>>, mut menu: ResMut<InGameMenuState>) {
    if keys.just_pressed(KeyCode::Escape) {
        menu.open = !menu.open;
    }
}

fn spawn_in_game_menu_ui(
    mut commands: Commands,
    menu: Res<InGameMenuState>,
    server_handle: Option<Res<ServerHandle>>,
    existing: Query<Entity, With<InGameMenuUI>>,
) {
    if !menu.is_changed() {
        return;
    }

    for entity in &existing {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }

    if !menu.open {
        return;
    }

    let is_host = server_handle.is_some();
    let status_text = if is_host {
        if menu.external_open {
            "LAN/Steam access: OPEN"
        } else {
            "LAN/Steam access: CLOSED"
        }
    } else {
        "Connected as client"
    };

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
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BorderColor::all(Color::WHITE),
                    BorderRadius::all(Val::Px(12.0)),
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                    InGameCleanup,
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("In-Game Menu"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        InGameCleanup,
                    ));

                    panel.spawn((
                        Text::new(status_text.to_string()),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        InGameCleanup,
                    ));

                    let mut spawn_button = |label: &str, action: InGameMenuButtonAction| {
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
                                InGameCleanup,
                                action,
                            ))
                            .with_children(|button| {
                                button.spawn((
                                    Text::new(label.to_string()),
                                    TextFont {
                                        font_size: 22.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                    InGameCleanup,
                                ));
                            });
                    };

                    spawn_button("Resume", InGameMenuButtonAction::Resume);

                    if is_host {
                        if menu.external_open {
                            spawn_button(
                                "Close LAN/Steam access",
                                InGameMenuButtonAction::CloseLan,
                            );
                        } else {
                            spawn_button("Open to LAN", InGameMenuButtonAction::OpenLan);
                        }
                        spawn_button("Return to Main Menu", InGameMenuButtonAction::LeaveSession);
                    } else {
                        spawn_button("Disconnect", InGameMenuButtonAction::LeaveSession);
                    }
                });
        });
}

fn handle_in_game_menu_buttons(
    mut commands: Commands,
    mut interaction_query: Query<
        (
            &Interaction,
            &InGameMenuButtonAction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    mut menu: ResMut<InGameMenuState>,
    mut next_state: ResMut<NextState<GameState>>,
    server_handle: Option<Res<ServerHandle>>,
    mut game_client: Option<ResMut<GameClient>>,
) {
    for (interaction, action, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                *border_color = BorderColor::all(RED);

                match action {
                    InGameMenuButtonAction::Resume => {
                        menu.open = false;
                    }
                    InGameMenuButtonAction::LeaveSession => {
                        if let Some(server) = server_handle.as_ref() {
                            server.shutdown();
                        }
                        if server_handle.is_some() {
                            commands.remove_resource::<ServerHandle>();
                            menu.external_open = false;
                        }

                        let mut remove_client = false;
                        if let Some(mut client) = game_client.take() {
                            if let Err(e) = client.transport.disconnect() {
                                warn!("Failed to disconnect client: {:?}", e);
                            }
                            remove_client = true;
                        }
                        if remove_client {
                            commands.remove_resource::<GameClient>();
                        }

                        commands.remove_resource::<LoopbackClient>();

                        menu.open = false;
                        next_state.set(GameState::MainMenu);
                    }
                    InGameMenuButtonAction::OpenLan => {
                        if let Some(server) = server_handle.as_ref() {
                            match create_default_quic_transport() {
                                Ok((external, addr)) => match server.add_external(external) {
                                    Ok(_) => {
                                        info!("LAN/Steam access opened on {}", addr);
                                        menu.external_open = true;
                                    }
                                    Err(e) => warn!("Failed to add external transport: {}", e),
                                },
                                Err(e) => warn!("Failed to create external transport: {}", e),
                            }
                        } else {
                            warn!("Cannot open LAN/Steam access without hosting server");
                        }
                    }
                    InGameMenuButtonAction::CloseLan => {
                        if let Some(server) = server_handle.as_ref() {
                            match server.remove_external() {
                                Ok(_) => {
                                    info!("LAN/Steam access closed");
                                    menu.external_open = false;
                                }
                                Err(e) => warn!("Failed to remove external transport: {}", e),
                            }
                        } else {
                            warn!("Cannot close LAN/Steam access without hosting server");
                        }
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

fn menu_closed(menu: Res<InGameMenuState>) -> bool {
    !menu.open
}

fn cleanup_in_game_entities(
    mut commands: Commands,
    mut menu: ResMut<InGameMenuState>,
    mut registry: ResMut<RemotePlayerRegistry>,
    cleanup_entities: Query<Entity, With<InGameCleanup>>,
) {
    menu.open = false;
    menu.external_open = false;
    registry.known_players.clear();
    for entity in &cleanup_entities {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.despawn();
        }
    }
    commands.remove_resource::<LoopbackClient>();
    commands.remove_resource::<GameClient>();
}
