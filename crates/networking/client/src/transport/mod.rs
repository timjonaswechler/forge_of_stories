pub mod quic;

pub use crate::steam::{SteamClientTransport, SteamTransportError as SteamClientTransportError};
pub use quic::QuicClientTransport;
pub use shared::transport::{ClientTransport, TransportPayload, TransportResult};

/// Possible endpoints a client can connect to.
#[derive(Debug, Clone)]
pub enum ConnectTarget {
    /// In-memory loopback connection (singleplayer, no network).
    Loopback,
    /// QUIC endpoint reachable via host/port pair.
    Quic { host: String, port: u16 },
    /// Steam lobby or relay identifier.
    SteamLobby { lobby_id: u64 },
}
