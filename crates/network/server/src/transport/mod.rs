use std::fmt::Debug;

use bytes::Bytes;
use network_shared::{ClientId, DisconnectReason, OutgoingMessage, TransportCapabilities, TransportEvent};
use tokio::sync::mpsc::UnboundedSender;

pub mod quic;

pub use quic::{QuicServerTransport, QuicServerTransportError};
pub use crate::steam::{SteamServerTransport, SteamServerTransportError};

/// Common interface implemented by all server-side transports (e.g. QUIC, Steam Relay).
pub trait ServerTransport: Send + Sync + Debug {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Starts the transport and begins emitting events via the provided channel.
    fn start(&mut self, events: UnboundedSender<TransportEvent>) -> Result<(), Self::Error>;

    /// Stops the transport and disconnects all peers.
    fn stop(&mut self);

    /// Sends a reliable payload to a connected client.
    fn send(&self, client: ClientId, message: OutgoingMessage) -> Result<(), Self::Error>;

    /// Sends an unreliable datagram payload to a connected client, if supported.
    fn send_datagram(&self, client: ClientId, payload: Bytes) -> Result<(), Self::Error>;

    /// Requests that the given client gets disconnected.
    fn disconnect(&self, client: ClientId, reason: DisconnectReason) -> Result<(), Self::Error>;

    /// Advertises the capabilities supported by the transport implementation.
    fn capabilities(&self) -> TransportCapabilities;
}
