use crate::ServerHandle;
use bevy::ecs::schedule::common_conditions::{not, resource_exists};
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{RenetChannelsExt, RepliconRenetPlugins};
use bevy_replicon_renet::{
    netcode::{ClientAuthentication, NetcodeClientTransport},
    renet::*,
};
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant, SystemTime};

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RepliconPlugins, RepliconRenetPlugins))
            .add_systems(
                Update,
                connect
                    .run_if(resource_exists::<ClientConnectRequest>)
                    .run_if(not(resource_exists::<RenetClient>)),
            );
    }
}

/// Request to start a local client connection once the target server is ready.
#[derive(Resource, Clone)]
pub struct ClientConnectRequest {
    server_addr: SocketAddr,
    wait_for_server_ready: bool,
    created_at: Instant,
    logged_waiting: bool,
}

impl ClientConnectRequest {
    pub fn singleplayer() -> Self {
        let server_addr = "127.0.0.1:7777"
            .parse()
            .expect("failed to parse singleplayer server address");
        Self {
            server_addr,
            wait_for_server_ready: true,
            created_at: Instant::now(),
            logged_waiting: false,
        }
    }
}

fn connect(
    mut commands: Commands,
    channels: Res<RepliconChannels>,
    mut request: ResMut<ClientConnectRequest>,
    server_handle: Option<Res<ServerHandle>>,
) {
    if request.wait_for_server_ready {
        let Some(handle) = server_handle else {
            if !request.logged_waiting {
                info!("Waiting for embedded server handle to be registered...");
                request.logged_waiting = true;
            }
            return;
        };

        if !handle.is_ready() {
            if request.created_at.elapsed() < Duration::from_millis(500) {
                if !request.logged_waiting {
                    info!("Server still booting, waiting before connecting…");
                    request.logged_waiting = true;
                }
                return;
            } else {
                warn!(
                    "Server still not reporting ready after 500ms, attempting client connection anyway."
                );
            }
        }

        // Server is ready (or timeout elapsed) so reset wait logging for potential reuse.
        request.logged_waiting = false;
    }

    let socket = UdpSocket::bind("0.0.0.0:0").expect("failed to bind client UDP socket");

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system time is before UNIX_EPOCH");
    let client_id = current_time.as_millis() as u64;

    let connection_config = ConnectionConfig {
        client_channels_config: channels.client_configs(),
        ..Default::default()
    };

    let client = RenetClient::new(connection_config);
    let transport = NetcodeClientTransport::new(
        current_time,
        ClientAuthentication::Unsecure {
            client_id,
            protocol_id: 0,
            server_addr: request.server_addr,
            user_data: None,
        },
        socket,
    )
    .expect("failed to create client transport");

    commands.insert_resource(client);
    commands.insert_resource(transport);
    commands.remove_resource::<ClientConnectRequest>();
    info!("Client verbunden");
}
