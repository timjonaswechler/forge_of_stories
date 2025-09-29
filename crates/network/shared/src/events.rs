//! Transport- und Netzwerkereignisse, die in die ECS gespiegelt werden.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    channels::ChannelKind,
    ids::{ClientId, SessionId},
    messages::NetworkMessage,
};

/// Gründe für das Trennen einer Verbindung.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DisconnectReason {
    Graceful,
    Timeout,
    Kicked,
    AuthenticationFailed,
    ProtocolMismatch,
    TransportError,
}

/// Fehler, die vom Transport gemeldet werden können.
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("transport not implemented")]
    Unimplemented,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] crate::serialization::SerializationError),
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    #[error("other: {0}")]
    Other(String),
}

/// Fähigkeitsbeschreibungen des Transports.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TransportCapabilities {
    pub supports_datagrams: bool,
    pub requires_auth: bool,
    pub max_ordered_streams: u16,
    pub max_unordered_streams: u16,
}

impl TransportCapabilities {
    pub const fn new(
        supports_datagrams: bool,
        requires_auth: bool,
        max_ordered_streams: u16,
        max_unordered_streams: u16,
    ) -> Self {
        Self {
            supports_datagrams,
            requires_auth,
            max_ordered_streams,
            max_unordered_streams,
        }
    }
}

impl Default for TransportCapabilities {
    fn default() -> Self {
        Self::new(true, false, 8, 8)
    }
}

/// Ereignisse, die ein Servertransport Richtung Gameplay schickt.
#[derive(Debug)]
pub enum TransportEvent {
    PeerConnected {
        session: SessionId,
        client: ClientId,
    },
    PeerDisconnected {
        session: SessionId,
        client: ClientId,
        reason: DisconnectReason,
    },
    Message {
        session: SessionId,
        client: ClientId,
        channel: ChannelKind,
        payload: NetworkMessage,
    },
    Datagram {
        session: SessionId,
        client: ClientId,
        payload: Vec<u8>,
    },
    Error {
        session: Option<SessionId>,
        client: Option<ClientId>,
        error: TransportError,
    },
}

/// Ereignisse, die der Client-Transport Richtung Gameplay schickt.
#[derive(Debug)]
pub enum ClientEvent {
    Connected,
    Disconnected {
        reason: DisconnectReason,
    },
    Message {
        channel: ChannelKind,
        payload: NetworkMessage,
    },
    Datagram {
        payload: Vec<u8>,
    },
    Error {
        error: TransportError,
    },
}
