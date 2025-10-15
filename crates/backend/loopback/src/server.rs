use std::{
    io,
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
};

use bevy::prelude::*;
use networking::{prelude::*, shared::backend::connected_client::NetworkId};

/// Adds a server messaging backend made for examples to `bevy_replicon`.
pub struct LoopbackServerPlugin;

impl Plugin for LoopbackServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                (
                    receive_packets.run_if(resource_exists::<LoopbackServer>),
                    // Run after since the resource might be removed after receiving packets.
                    set_stopped.run_if(resource_removed::<LoopbackServer>),
                )
                    .chain(),
                set_running.run_if(resource_added::<LoopbackServer>),
            )
                .in_set(ServerSystems::ReceivePackets),
        )
        .add_systems(
            PostUpdate,
            send_packets
                .run_if(resource_exists::<LoopbackServer>)
                .in_set(ServerSystems::SendPackets),
        );
    }
}

fn set_running(mut state: ResMut<NextState<ServerState>>) {
    state.set(ServerState::Running);
}

fn set_stopped(mut state: ResMut<NextState<ServerState>>) {
    state.set(ServerState::Stopped);
}

fn receive_packets(
    mut commands: Commands,
    mut messages: ResMut<ServerMessages>,
    server: Res<LoopbackServer>,
    mut clients: Query<(Entity, &mut LoopbackConnection)>,
) {
    loop {
        match server.0.accept() {
            Ok((stream, addr)) => {
                if let Err(e) = stream.set_nodelay(true) {
                    error!("unable to disable buffering for `{addr}`: {e}");
                    continue;
                }
                if let Err(e) = stream.set_nonblocking(true) {
                    error!("unable to enable non-blocking for `{addr}`: {e}");
                    continue;
                }
                let network_id = NetworkId::new(addr.port().into());
                let client = commands
                    .spawn((
                        ConnectedClient { max_size: 1200 },
                        network_id,
                        LoopbackConnection { stream },
                    ))
                    .id();
                debug!("connecting client `{client}` with `{network_id:?}`");
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    error!("stopping server due to network error: {e}");
                    commands.remove_resource::<LoopbackServer>();
                }
                break;
            }
        }
    }

    for (client, mut connection) in &mut clients {
        loop {
            match crate::tcp::read_message(&mut connection.stream) {
                Ok((channel_id, message)) => {
                    messages.insert_received(client, channel_id, message);
                }
                Err(e) => {
                    match e.kind() {
                        io::ErrorKind::WouldBlock => (),
                        io::ErrorKind::UnexpectedEof => {
                            commands.entity(client).despawn();
                            debug!("`client {client}` closed the connection");
                        }
                        _ => {
                            commands.entity(client).despawn();
                            error!(
                                "disconnecting client `{client}` due to message read error: {e}"
                            );
                        }
                    }
                    break;
                }
            }
        }
    }
}

fn send_packets(
    mut commands: Commands,
    mut disconnects: MessageReader<DisconnectRequest>,
    mut messages: ResMut<ServerMessages>,
    mut clients: Query<&mut LoopbackConnection>,
) {
    for (client, channel_id, message) in messages.drain_sent() {
        let mut connection = clients
            .get_mut(client)
            .expect("all connected clients should have streams");
        if let Err(e) = crate::tcp::send_message(&mut connection.stream, channel_id, &message) {
            commands.entity(client).despawn();
            error!("disconnecting client `{client}` due to error: {e}");
        }
    }

    for disconnect in disconnects.read() {
        debug!("disconnecting client `{}` by request", disconnect.client);
        commands.entity(disconnect.client).despawn();
    }
}

fn close_stream_on_despawn(
    trigger: On<Remove, ConnectedClient>,
    clients: Query<&LoopbackConnection>,
) {
    if let Ok(connection) = clients.get(trigger.entity) {
        debug!("closing stream for despawned client `{}`", trigger.entity);
        // TCP Stream wird automatisch geclosed beim Drop
        // Aber du könntest explizit connection.stream.shutdown(...) callen
    }
}

fn update_statistics(
    mut bps_timer: Local<f64>,
    mut clients: Query<(&NetworkId, &mut ConnectedClient, &mut ClientStats)>,
    mut loopback_server: ResMut<LoopbackServer>,
    time: Res<Time>,
) {
    let Some(endpoint) = loopback_server.get_endpoint_mut() else {
        return;
    };
    for (network_id, mut client, mut client_stats) in clients.iter_mut() {
        let Some(connection) = endpoint.connection_mut(network_id.get()) else {
            continue;
        };

        if let Some(max_size) = connection.max_datagram_size() {
            client.max_size = max_size;
        }

        let quinn_stats = connection.quinn_connection_stats();
        client_stats.rtt = quinn_stats.path.rtt.as_secs_f64();
        client_stats.packet_loss = if quinn_stats.path.sent_packets == 0 {
            0.0
        } else {
            100.0 * (quinn_stats.path.lost_packets as f64 / quinn_stats.path.sent_packets as f64)
        };

        *bps_timer += time.delta_secs_f64();
        if *bps_timer >= BYTES_PER_SEC_PERIOD {
            *bps_timer = 0.;
            let stats = connection.stats_mut();
            let received_bytes_count = stats.clear_received_bytes_count() as f64;
            let sent_bytes_count = stats.clear_sent_bytes_count() as f64;
            client_stats.received_bps = received_bytes_count / BYTES_PER_SEC_PERIOD;
            client_stats.sent_bps = sent_bytes_count / BYTES_PER_SEC_PERIOD;
        }
    }
}

/// The socket used by the server.
#[derive(Resource)]
pub struct LoopbackServer(TcpListener);

impl LoopbackServer {
    /// Opens an example server socket on the specified port.
    pub fn new(port: u16) -> io::Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;
        listener.set_nonblocking(true)?;
        Ok(Self(listener))
    }

    /// Returns local address if the server is running.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.0.local_addr()
    }
}

/// A connected for a client.
#[derive(Component)]
struct LoopbackConnection {
    stream: TcpStream,
}
