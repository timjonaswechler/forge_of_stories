use bytes::Bytes;
use thiserror::Error;

use crate::{ClientId, channels::ChannelId, steam::SteamDiscoveryEvent};

/// Reasons why a peer might be disconnected from the transport layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisconnectReason {
    Graceful,
    Timeout,
    Kicked,
    AuthenticationFailed,
    ProtocolMismatch,
    TransportError,
}

/// Generic transport level error surfaced to higher layers.
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("transport not ready")]
    NotReady,
    #[error("configuration error: {0}")]
    InvalidConfig(&'static str),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("other: {0}")]
    Other(String),
}

/// Capability description for a concrete transport implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransportCapabilities {
    pub supports_reliable_streams: bool,
    pub supports_unreliable_streams: bool,
    pub supports_datagrams: bool,
    pub max_channels: u16,
}

impl TransportCapabilities {
    pub const fn new(
        supports_reliable_streams: bool,
        supports_unreliable_streams: bool,
        supports_datagrams: bool,
        max_channels: u16,
    ) -> Self {
        Self {
            supports_reliable_streams,
            supports_unreliable_streams,
            supports_datagrams,
            max_channels,
        }
    }
}

impl Default for TransportCapabilities {
    fn default() -> Self {
        Self::new(true, true, true, u8::MAX as u16)
    }
}

/// Server-side events emitted by a transport implementation.
#[derive(Debug)]
pub enum TransportEvent {
    PeerConnected {
        client: ClientId,
    },
    PeerDisconnected {
        client: ClientId,
        reason: DisconnectReason,
    },
    Message {
        client: ClientId,
        channel: ChannelId,
        payload: Bytes,
    },
    Datagram {
        client: ClientId,
        payload: Bytes,
    },
    Error {
        client: Option<ClientId>,
        error: TransportError,
    },
    AuthResult {
        client: Option<ClientId>,
        steam_id: u64,
        owner_steam_id: u64,
        result: Result<(), String>,
    },
}

/// Client-side events emitted by a transport implementation.
#[derive(Debug)]
pub enum ClientEvent {
    Connected {
        client_id: Option<ClientId>,
    },
    Disconnected {
        reason: DisconnectReason,
    },
    Message {
        channel: ChannelId,
        payload: Bytes,
    },
    Datagram {
        payload: Bytes,
    },
    Error {
        error: TransportError,
    },
    Discovery(SteamDiscoveryEvent),
    AuthResult {
        client: Option<ClientId>,
        steam_id: u64,
        owner_steam_id: u64,
        result: Result<(), String>,
    },
}
