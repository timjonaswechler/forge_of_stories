use std::{
    io,
    net::{SocketAddr, TcpStream},
};

use bevy::prelude::*;
use networking::prelude::*;

/// Adds a client messaging backend made for examples to `bevy_replicon`.
pub struct LoopbackClientPlugin;

impl Plugin for LoopbackClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                (
                    receive_packets.run_if(resource_exists::<LoopbackClient>),
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
        match crate::tcp::read_message(&mut client.stream) {
            Ok((channel_id, message)) => {
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
) {
    for (channel_id, message) in messages.drain_sent() {
        if let Err(e) = crate::tcp::send_message(&mut client.stream, channel_id, &message) {
            error!("disconnecting due message write error: {e}");
            commands.remove_resource::<LoopbackClient>();
            return;
        }
    }
}

/// The socket used by the client.
#[derive(Resource)]
pub struct LoopbackClient {
    stream: TcpStream,
}

impl LoopbackClient {
    /// Opens an example client socket connected to a server on the specified port.
    pub fn new(addr: impl Into<SocketAddr>) -> io::Result<Self> {
        let stream = TcpStream::connect(addr.into())?;
        stream.set_nonblocking(true)?;
        stream.set_nodelay(true)?;
        Ok(Self { stream })
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
