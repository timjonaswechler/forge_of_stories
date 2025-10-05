mod fos_app;
mod server;

use crate::fos_app::FOSApp;
use crate::server::{EmbeddedServer, ServerMode};
use app::AppBuilder;
use bevy::{color::palettes::basic::*, input_focus::InputFocus, log::LogPlugin, prelude::*};

/// Game state tracking where we are in the application flow.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    MainMenu,
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
            .init_resource::<InputFocus>()
            .init_resource::<ServerToClientEntityMap>()
            .init_state::<GameState>()
            .add_systems(Startup, setup)
            .add_systems(Update, button_system.run_if(in_state(GameState::MainMenu)))
            .add_systems(OnEnter(GameState::InGame), enter_game)
            .add_systems(
                Update,
                (handle_player_input, sync_server_state)
                    .run_if(in_state(GameState::InGame))
                    .run_if(resource_exists::<EmbeddedServer>),
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

fn setup(mut commands: Commands) {
    // ui camera
    commands.spawn(Camera2d);
    commands.spawn(button());
}

/// System that runs when entering the InGame state
fn enter_game(
    mut commands: Commands,
    server: Res<EmbeddedServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Entered game state! Server mode: {:?}", server.mode());

    // Log server world stats
    let entity_count = server.world().entities().len();
    info!("Server world has {} entities", entity_count);

    // Spawn 3D camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

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

    // Spawn a simple in-game UI showing we're in singleplayer
    commands.spawn((
        Text::new(format!(
            "In Game - Server: {:?}\nEntities: {}",
            server.mode(),
            entity_count
        )),
        TextFont {
            font_size: 24.0,
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
        .query::<(bevy::ecs::entity::Entity, (&Player, &Position, &PlayerShape))>()
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
                    ClientPlayer { server_id: player.id },
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

/// System that captures player input and sends it to the server.
fn handle_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut server: ResMut<EmbeddedServer>,
) {
    use server_logic::movement::PlayerInput;

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

fn button_system(
    mut commands: Commands,
    mut input_focus: ResMut<InputFocus>,
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Button,
            &Children,
        ),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut Text>,
) {
    for (entity, interaction, mut color, mut border_color, mut button, children) in
        &mut interaction_query
    {
        let mut text = text_query.get_mut(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                input_focus.set(entity);
                **text = "Starting...".to_string();
                *color = PRESSED_BUTTON.into();
                *border_color = BorderColor::all(RED);
                button.set_changed();

                // Start singleplayer game
                info!("Starting singleplayer game...");

                // Create embedded server with loopback mode
                match EmbeddedServer::new(ServerMode::Loopback) {
                    Ok(server) => {
                        info!("Embedded server started successfully");
                        commands.insert_resource(server);

                        // Transition to in-game state
                        next_state.set(GameState::InGame);
                    }
                    Err(e) => {
                        error!("Failed to start embedded server: {}", e);
                        **text = "Error!".to_string();
                    }
                }
            }
            Interaction::Hovered => {
                input_focus.set(entity);
                **text = "Start Singleplayer".to_string();
                *color = HOVERED_BUTTON.into();
                *border_color = BorderColor::all(Color::WHITE);
                button.set_changed();
            }
            Interaction::None => {
                input_focus.clear();
                **text = "Singleplayer".to_string();
                *color = NORMAL_BUTTON.into();
                *border_color = BorderColor::all(Color::BLACK);
            }
        }
    }
}

fn button() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Button,
            Node {
                width: Val::Px(150.0),
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(Color::WHITE),
            BorderRadius::MAX,
            BackgroundColor(Color::BLACK),
            children![(
                Text::new("Singleplayer"),
                TextFont {
                    font_size: 33.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextShadow::default(),
            )]
        )],
    )
}
