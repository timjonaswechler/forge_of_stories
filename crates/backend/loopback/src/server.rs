use std::{
    io,
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
};
/// Adds a server messaging backend made for examples to `bevy_replicon`.
pub struct LoopbackServerPlugin;

impl Plugin for LoopbackServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(close_stream_on_despawn)
            .add_systems(
                PreUpdate,
                (
                    (
                        receive_packets.run_if(resource_exists::<LoopbackServer>),
                        update_statistics.run_if(resource_exists::<LoopbackServer>),
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
                (
                    send_packets.in_set(ServerSystems::SendPackets),
                    disconnect_by_request.after(ServerSystems::SendPackets),
                )
                    .run_if(resource_exists::<LoopbackServer>),
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
        match server.endpoint().accept() {
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
                        LoopbackConnection::new(stream),
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
                    // Track received bytes
                    connection.stats.bytes_received += message.len();

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

    mut messages: ResMut<ServerMessages>,
    mut clients: Query<&mut LoopbackConnection>,
) {
    for (client, channel_id, message) in messages.drain_sent() {
        let mut connection = clients

        // Track sent bytes
        connection.stats.bytes_sent += message.len();

        if let Err(e) = crate::tcp::send_message(&mut connection.stream, channel_id, &message) {
            commands.entity(client).despawn();
            error!("disconnecting client `{client}` due to error: {e}");
        }
    }
}

fn disconnect_by_request(
    mut commands: Commands,
    mut disconnects: MessageReader<DisconnectRequest>,
) {
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

    mut clients: Query<(&mut LoopbackConnection, &mut ClientStats)>,
    time: Res<Time>,
) {
    *bps_timer += time.delta_secs_f64();
    if *bps_timer >= BYTES_PER_SEC_PERIOD {
        *bps_timer = 0.0;

        for (mut connection, mut client_stats) in &mut clients {
            // Calculate BPS from tracked bytes
            client_stats.received_bps =
                connection.stats.bytes_received as f64 / BYTES_PER_SEC_PERIOD;
            client_stats.sent_bps = connection.stats.bytes_sent as f64 / BYTES_PER_SEC_PERIOD;

            // Reset counters
            connection.stats.bytes_received = 0;
            connection.stats.bytes_sent = 0;

            // TCP doesn't expose RTT/packet loss easily, set to defaults
            client_stats.rtt = 0.0;
            client_stats.packet_loss = 0.0;
        }
    }
}

#[derive(Component)]
struct LoopbackConnection {
    stream: TcpStream,
    stats: ConnectionStats,
}

#[derive(Default)]
struct ConnectionStats {
    bytes_sent: usize,
    bytes_received: usize,
}

impl LoopbackConnection {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            stats: ConnectionStats::default(),
        }
    }
}

pub struct LoopbackServer {
    endpoint: Option<TcpListener>,
}

impl LoopbackServer {
    /// Opens an example server socket on the specified port.
    pub fn new(port: u16) -> io::Result<Self> {
        Ok(Self {
            endpoint: Some(listener),
        })
    }

    /// Returns a reference to the server's endpoint.
    ///
    /// **Panics** if the endpoint is not opened
    pub fn endpoint(&self) -> &TcpListener {
        self.endpoint.as_ref().unwrap()
    }

    /// Returns a mutable reference to the server's endpoint
    ///
    /// **Panics** if the endpoint is not opened
    pub fn endpoint_mut(&mut self) -> &mut TcpListener {
        self.endpoint.as_mut().unwrap()
    }

    /// Returns an optional reference to the server's endpoint
    pub fn get_endpoint(&self) -> Option<&TcpListener> {
        self.endpoint.as_ref()
    }

    /// Returns an optional mutable reference to the server's endpoint
    pub fn get_endpoint_mut(&mut self) -> Option<&mut TcpListener> {
        self.endpoint.as_mut()
    }

    /// Returns local address if the server is running.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.endpoint().local_addr()
    }
}
