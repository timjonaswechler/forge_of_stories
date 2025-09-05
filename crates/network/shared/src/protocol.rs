pub const PROTOCOL_VERSION: u16 = 1;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ClientHello {
    Version { version: u16 },
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ServerHello {
    VersionAccepted { version: u16 },
    VersionMismatch { server_version: u16 },
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum AuthRequest {
    Token(String), // Access-Token/Join-Code
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum AuthResponse {
    Ok { session_id: u64 },
    Denied { reason: String },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Rpc {
    // Beispiele – später ausdifferenzieren
    JoinWorld,
    PerformAction { action_id: u32, payload: Vec<u8> },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Event {
    // Server->Client Events, auch broadcastbar
    WorldDelta(Vec<u8>),
    Chat(String),
}
