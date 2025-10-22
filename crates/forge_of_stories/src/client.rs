use crate::GameState;
use bevy::math::primitives::{Capsule3d, Cuboid};
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{
    RenetChannelsExt, RepliconRenetPlugins,
    netcode::{ClientAuthentication, NetcodeClientTransport},
    renet::{ConnectionConfig, RenetClient},
};
use game_server::{
    components::{Player, Position},
    world::{GroundPlane, GroundPlaneSize},
};
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::SystemTime;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RepliconPlugins, RepliconRenetPlugins))
            // Register replicated components (must match server!)
            .replicate::<Player>()
            .replicate::<Position>()
            .replicate::<game_server::components::Velocity>()
            .replicate::<GroundPlane>()
            .replicate::<GroundPlaneSize>()
            // Connect to embedded server when entering InGame
            .add_systems(
                OnEnter(GameState::InGame),
                (setup_client_networking, setup_client_world),
            )
            .add_systems(
                Update,
                (
                    debug_replicated_entities,
                    spawn_ground_plane_visuals,
                    spawn_player_visuals,
                    update_transforms_from_positions,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                OnExit(GameState::InGame),
                (
                    crate::utils::cleanup::<ClientWorldEntity>,
                    crate::utils::remove::<ClientRenderAssets>,
                ),
            );
    }
}

/// Cached handles for meshes we reuse while the client is in-game.
#[derive(Resource)]
struct ClientRenderAssets {
    player_mesh: Handle<Mesh>,
}

/// Marker for light and camera entities that should be cleaned up when leaving the game.
#[derive(Component)]
struct ClientWorldEntity;

/// Marker to track which entities already have visuals spawned
#[derive(Component)]
struct HasVisuals;

fn setup_client_networking(
    mut commands: Commands,
    channels: Res<RepliconChannels>,
    server_handle: Res<game_server::ServerHandle>,
) {
    const PROTOCOL_ID: u64 = 0;
    let server_port = server_handle.port();

    info!(
        "🟡 CLIENT: Setting up networking to connect to localhost:{}",
        server_port
    );

    let server_addr: SocketAddr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), server_port);

    let server_channels_config = channels.server_configs();
    let client_channels_config = channels.client_configs();

    let client = RenetClient::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Failed to get system time");

    let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).expect("Failed to bind client socket");

    let authentication = ClientAuthentication::Unsecure {
        protocol_id: PROTOCOL_ID,
        client_id: current_time.as_millis() as u64,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket)
        .expect("Failed to create client transport");

    commands.insert_resource(client);
    commands.insert_resource(transport);

    info!(
        "🟡 CLIENT: Networking setup complete, connecting to {}",
        server_addr
    );
}

fn setup_client_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    ambient_light: Option<ResMut<AmbientLight>>,
    mut camera_query: Query<(Entity, &mut Transform), With<Camera3d>>,
) {
    info!("setup Client World");

    if let Some(mut ambient_light) = ambient_light {
        ambient_light.brightness = 2000.0;
        ambient_light.color = Color::srgb(1.0, 1.0, 1.0);
    }

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 35_000.0,
            ..default()
        },
        Transform::from_xyz(-12.0, 18.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ClientWorldEntity,
    ));

    // Reuse an existing camera if we have one from the splash screen, otherwise spawn a new one.
    if let Some((entity, mut transform)) = camera_query.iter_mut().next() {
        transform.translation = Vec3::new(-12.0, 10.0, 18.0);
        transform.look_at(Vec3::ZERO, Vec3::Y);
        commands.entity(entity).insert(ClientWorldEntity);
    } else {
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(-12.0, 10.0, 18.0).looking_at(Vec3::ZERO, Vec3::Y),
            ClientWorldEntity,
        ));
    }

    let player_mesh: Handle<Mesh> = meshes.add(Mesh::from(Capsule3d::default()));

    commands.insert_resource(ClientRenderAssets { player_mesh });
}

#[cfg(debug_assertions)]
fn debug_replicated_entities(
    new_planes: Query<(Entity, &Position), Added<GroundPlane>>,
    new_players: Query<(Entity, &Player, &Position), Added<Player>>,
) {
    for (entity, position) in &new_planes {
        info!(
            "🟡 CLIENT: New GroundPlane {:?} at {:?}",
            entity, position.translation
        );
    }
    for (entity, player, position) in &new_players {
        info!(
            "🟡 CLIENT: New Player {:?} color {:?} at {:?}",
            entity, player.color, position.translation
        );
    }
}

#[cfg(not(debug_assertions))]
fn debug_replicated_entities() {
    // No-op in release builds
}

fn spawn_ground_plane_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    planes: Query<(Entity, &Position, &GroundPlaneSize), (With<GroundPlane>, Without<HasVisuals>)>,
) {
    for (entity, position, size) in &planes {
        info!(
            "Spawning visuals for ground plane at {:?} with size {}x{}x{}",
            position.translation, size.width, size.height, size.depth
        );

        // Create mesh based on replicated size from server
        let mesh = meshes.add(Mesh::from(Cuboid::new(size.width, size.height, size.depth)));

        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.25, 0.45, 0.25),
            perceptual_roughness: 0.7,
            ..default()
        });

        commands.entity(entity).insert((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(position.translation),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            HasVisuals,
        ));
    }
}

fn spawn_player_visuals(
    mut commands: Commands,
    assets: Res<ClientRenderAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    players: Query<(Entity, &Player, &Position), Without<HasVisuals>>,
) {
    for (entity, player, position) in &players {
        info!(
            "Spawning visuals for player with color {:?} at {:?}",
            player.color, position.translation
        );

        let material = materials.add(StandardMaterial {
            base_color: player.color,
            ..default()
        });

        commands.entity(entity).insert((
            Mesh3d(assets.player_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(position.translation + Vec3::Y * 0.5),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            HasVisuals,
        ));
    }
}

fn update_transforms_from_positions(
    mut query: Query<(&Position, &mut Transform), Changed<Position>>,
) {
    for (position, mut transform) in &mut query {
        transform.translation = position.translation;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_updates_when_position_changes() {
        let mut app = App::new();
        app.add_systems(Update, update_transforms_from_positions);

        let entity = app
            .world_mut()
            .spawn((
                Position {
                    translation: Vec3::new(1.0, 2.0, 3.0),
                },
                Transform::default(),
            ))
            .id();

        app.update();

        {
            let world = app.world();
            let transform = world.get::<Transform>(entity).expect("transform missing");
            assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
        }

        {
            let world = app.world_mut();
            let mut position = world.get_mut::<Position>(entity).expect("position missing");
            position.translation = Vec3::new(-4.0, 0.5, 9.0);
        }

        app.update();

        let world = app.world();
        let transform = world.get::<Transform>(entity).expect("transform missing");
        assert_eq!(transform.translation, Vec3::new(-4.0, 0.5, 9.0));
    }
}
