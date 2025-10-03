use std::fmt::Debug;

use bytes::Bytes;
use network_shared::{ClientEvent, DisconnectReason, OutgoingMessage, TransportCapabilities};
use tokio::sync::mpsc::UnboundedSender;

pub mod quic;

pub use crate::steam::{SteamClientTransport, SteamTransportError as SteamClientTransportError};
pub use quic::{QuicClientTransport, QuicClientTransportError};

/// Possible endpoints a client can connect to.
#[derive(Debug, Clone)]
pub enum ConnectTarget {
    /// QUIC endpoint reachable via host/port pair.
    Quic { host: String, port: u16 },
    /// Steam lobby or relay identifier.
    SteamLobby { lobby_id: u64 },
}

/// Common interface implemented by all client-side transports (e.g. QUIC, Steam Relay).
pub trait ClientTransport: Send + Sync + Debug {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Connects to the given target and starts emitting events via the provided channel.
    fn connect(
        &mut self,
        target: ConnectTarget,
        events: UnboundedSender<ClientEvent>,
    ) -> Result<(), Self::Error>;

    /// Disconnects from the current server.
    fn disconnect(&mut self, reason: DisconnectReason) -> Result<(), Self::Error>;

    /// Sends a reliable payload to the connected server.
    fn send(&self, message: OutgoingMessage) -> Result<(), Self::Error>;

    /// Sends an unreliable datagram payload to the server, if supported.
    fn send_datagram(&self, payload: Bytes) -> Result<(), Self::Error>;

    /// Advertises the capabilities supported by the transport implementation.
    fn capabilities(&self) -> TransportCapabilities;
}
