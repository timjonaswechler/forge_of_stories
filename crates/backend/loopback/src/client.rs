use std::{
    io,
    net::{SocketAddr, TcpStream},
};

/// Adds a client messaging backend made for examples to `bevy_replicon`.
pub struct LoopbackClientPlugin;

impl Plugin for LoopbackClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                    update_statistics.run_if(resource_exists::<LoopbackClient>),
                    // Run after since the resource might be removed after receiving packets.
                    set_disconnected.run_if(resource_removed::<LoopbackClient>),
                )
                    .chain(),
                set_connected.run_if(resource_added::<LoopbackClient>),
            )
                .in_set(ClientSystems::ReceivePackets),
        )
        .add_systems(
            PostUpdate,
            send_packets
                .run_if(resource_exists::<LoopbackClient>)
                .in_set(ClientSystems::SendPackets),
        );
    }
}

fn set_connected(mut state: ResMut<NextState<ClientState>>) {
    state.set(ClientState::Connected);
}

fn set_disconnected(mut state: ResMut<NextState<ClientState>>) {
    state.set(ClientState::Disconnected);
}

fn receive_packets(
    mut commands: Commands,
    mut client: ResMut<LoopbackClient>,
    mut messages: ResMut<ClientMessages>,
) {
    loop {
                // Track received bytes
                client.stats.bytes_received += message.len();

                messages.insert_received(channel_id, message);
            }
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => break,
                io::ErrorKind::UnexpectedEof => {
                    debug!("server closed the connection");
                    commands.remove_resource::<LoopbackClient>();
                    break;
                }
                _ => {
                    error!("disconnecting due to message read error: {e}");
                    commands.remove_resource::<LoopbackClient>();
                    break;
                }
            },
        }
    }
}

fn send_packets(
    mut commands: Commands,
    mut client: ResMut<LoopbackClient>,
    mut messages: ResMut<ClientMessages>,
        // Track sent bytes
        client.stats.bytes_sent += message.len();

        if let Err(e) = crate::tcp::send_message(&mut client.stream, channel_id, &message) {
            error!("disconnecting due message write error: {e}");
            commands.remove_resource::<LoopbackClient>();
            return;
        }
    }
fn update_statistics(
    mut bps_timer: Local<f64>,
    mut client: ResMut<LoopbackClient>,
    mut client_stats: ResMut<ClientStats>,
    time: Res<Time>,
) {
    *bps_timer += time.delta_secs_f64();
    if *bps_timer >= BYTES_PER_SEC_PERIOD {
        *bps_timer = 0.0;

        // Calculate BPS from tracked bytes
        client_stats.received_bps = client.stats.bytes_received as f64 / BYTES_PER_SEC_PERIOD;
        client_stats.sent_bps = client.stats.bytes_sent as f64 / BYTES_PER_SEC_PERIOD;

        // Reset counters
        client.stats.bytes_received = 0;
        client.stats.bytes_sent = 0;

        // TCP doesn't expose RTT/packet loss easily, set to defaults
        client_stats.rtt = 0.0;
        client_stats.packet_loss = 0.0;
    }
}

/// The socket used by the client.
#[derive(Resource)]
pub struct LoopbackClient {
    stream: TcpStream,
    stats: ConnectionStats,
}

#[derive(Default)]
struct ConnectionStats {
    bytes_sent: usize,
    bytes_received: usize,
}

impl LoopbackClient {
    /// Opens an example client socket connected to a server on the specified port.
    pub fn new(addr: impl Into<SocketAddr>) -> io::Result<Self> {
        let stream = TcpStream::connect(addr.into())?;
        Ok(Self {
            stream,
            stats: ConnectionStats::default(),
        })
    }

    /// Returns local address if connected.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.stream.local_addr()
    }

    /// Returns true if the client is connected.
    pub fn is_connected(&self) -> bool {
        self.local_addr().is_ok()
    }
}
