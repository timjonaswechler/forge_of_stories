use crate::GameState;
use app::LOG_CLIENT;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{
    RenetChannelsExt, RepliconRenetPlugins,
    netcode::{ClientAuthentication, NetcodeClientTransport},
    renet::{ConnectionConfig, RenetClient},
};
use game_server::{
    components::{Player, PlayerIdentity, Position, Velocity},
    world::{GroundPlane, GroundPlaneSize},
};
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::SystemTime;

/// Resource storing the local client's session-scoped identifier.
///
/// This ID is generated during the Renet authentication handshake and is replicated
/// back to us inside [`PlayerIdentity`] so we can recognise our own player entity.
#[derive(Resource, Debug, Clone, Copy)]
pub struct LocalClientId(pub u64);

/// Marker component inserted on the replicated entity that represents this client.
#[derive(Component)]
pub struct LocalPlayer;

/// Client plugin responsible for networking and replication
pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RepliconPlugins, RepliconRenetPlugins))
            // Register replicated components (must match server!)
            .replicate::<Player>()
            .replicate::<PlayerIdentity>()
            .replicate::<Position>()
            .replicate::<Velocity>()
            .replicate::<GroundPlane>()
            .replicate::<GroundPlaneSize>()
            // Connect to embedded server when entering InGame
            .add_systems(OnEnter(GameState::InGame), setup_client_networking)
            .add_systems(
                Update,
                mark_local_player.run_if(in_state(GameState::InGame)),
            );
    }
}

fn setup_client_networking(
    mut commands: Commands,
    channels: Res<RepliconChannels>,
    server_handle: Res<game_server::ServerHandle>,
) {
    const PROTOCOL_ID: u64 = 0;
    let server_port = server_handle.port();

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
    let client_id = current_time.as_millis() as u64;

    let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).expect("Failed to bind client socket");

    let authentication = ClientAuthentication::Unsecure {
        protocol_id: PROTOCOL_ID,
        client_id,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket)
        .expect("Failed to create client transport");

    commands.insert_resource(client);
    commands.insert_resource(transport);
    commands.insert_resource(LocalClientId(client_id));

    info!(
        target: LOG_CLIENT,
       "Networking setup complete, connecting to {}",
        server_addr
    );
}

fn mark_local_player(
    mut commands: Commands,
    local_id: Option<Res<LocalClientId>>,
    players: Query<(Entity, &PlayerIdentity), With<Player>>,
    current_local: Query<Entity, With<LocalPlayer>>,
) {
    let Some(local_id) = local_id else { return };

    // Find the replicated entity that belongs to this client (if any)
    let target = players
        .iter()
        .find_map(|(entity, identity)| (identity.client_id == local_id.0).then_some(entity));

    // Remove the marker from entities that no longer match
    for entity in &current_local {
        if Some(entity) != target {
            commands.entity(entity).remove::<LocalPlayer>();
        }
    }

    // Ensure the marker is present on the correct entity
    if let Some(target) = target {
        if current_local.get(target).is_err() {
            commands.entity(target).insert(LocalPlayer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_server::components::Position;

    #[test]
    fn transform_updates_when_position_changes() {
        let mut app = App::new();

        // This test would need the update_transforms_from_positions system
        // which is now in rendering/visual_spawners.rs
        // Keep this test structure for future integration tests

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

        let world = app.world();
        let transform = world.get::<Transform>(entity).expect("transform missing");
        assert_eq!(transform.translation, Vec3::new(1.0, 2.0, 3.0));
    }
}
