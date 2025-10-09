//! Transport layer abstractions shared across client and server implementations.

use bytes::Bytes;

use crate::{ClientEvent, ClientId, TransportError, TransportEvent, channels::ChannelId};

pub mod loopback;
pub mod orchestrator;

pub use loopback::{LoopbackClientTransport, LoopbackError, LoopbackPair, LoopbackServerTransport};
pub use orchestrator::{OrchestratorError, TransportOrchestrator};

/// Result alias for transport operations.
pub type TransportResult<T> = Result<T, TransportError>;

/// Payload description for transport send operations.
#[derive(Debug, Clone)]
pub enum TransportPayload {
    /// Reliable message routed over a logical channel.
    Message { channel: ChannelId, payload: Bytes },
    /// Unreliable datagram payload (channel-less).
    Datagram { payload: Bytes },
}

impl TransportPayload {
    /// Creates a message payload targeted at a specific logical channel.
    pub fn message(channel: ChannelId, payload: impl Into<Bytes>) -> Self {
        Self::Message {
            channel,
            payload: payload.into(),
        }
    }

    /// Creates an unreliable datagram payload.
    pub fn datagram(payload: impl Into<Bytes>) -> Self {
        Self::Datagram {
            payload: payload.into(),
        }
    }

    /// Returns the underlying bytes slice.
    pub fn bytes(&self) -> &Bytes {
        match self {
            Self::Message { payload, .. } | Self::Datagram { payload } => payload,
        }
    }

    /// Returns the logical channel if the payload is a message.
    pub fn channel(&self) -> Option<ChannelId> {
        match self {
            Self::Message { channel, .. } => Some(*channel),
            Self::Datagram { .. } => None,
        }
    }
}

impl From<(ChannelId, Bytes)> for TransportPayload {
    fn from(value: (ChannelId, Bytes)) -> Self {
        Self::Message {
            channel: value.0,
            payload: value.1,
        }
    }
}

/// Unified trait implemented by all server-side transports.
pub trait ServerTransport: Send + Sync {
    /// Polls the transport for pending events, appending them to the provided buffer.
    fn poll_events(&mut self, output: &mut Vec<TransportEvent>);

    /// Helper that returns a freshly collected vector of events.
    fn poll_events_vec(&mut self) -> Vec<TransportEvent> {
        let mut events = Vec::new();
        self.poll_events(&mut events);
        events
    }

    /// Sends a payload to a specific client.
    fn send(&mut self, client: ClientId, payload: TransportPayload) -> TransportResult<()>;

    /// Broadcasts a payload to all connected clients.
    fn broadcast(&mut self, payload: TransportPayload) -> TransportResult<()> {
        self.broadcast_excluding(&[], payload)
    }

    /// Broadcasts a payload to all clients except the provided list.
    fn broadcast_excluding(
        &mut self,
        exclude: &[ClientId],
        payload: TransportPayload,
    ) -> TransportResult<()>;
}

/// Unified trait implemented by all client-side transports.
pub trait ClientTransport: Send + Sync {
    /// Transport-specific connection target description.
    type ConnectTarget;

    /// Polls the transport for pending client events, appending them to the provided buffer.
    fn poll_events(&mut self, output: &mut Vec<ClientEvent>);

    /// Helper that returns a freshly collected vector of client events.
    fn poll_events_vec(&mut self) -> Vec<ClientEvent> {
        let mut events = Vec::new();
        self.poll_events(&mut events);
        events
    }

    /// Initiates a connection to the provided target.
    fn connect(&mut self, target: Self::ConnectTarget) -> TransportResult<()>;

    /// Terminates the current connection, if any.
    fn disconnect(&mut self) -> TransportResult<()>;

    /// Sends a payload to the connected peer.
    fn send(&mut self, payload: TransportPayload) -> TransportResult<()>;
}
