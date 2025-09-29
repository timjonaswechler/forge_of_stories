//! Kernnachrichten zwischen Server und Client.

use serde::{Deserialize, Serialize};

use crate::{channels::ChannelKind, ids::ClientId};

/// Version des Netzwerkprotokolls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
}

impl ProtocolVersion {
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

/// Nachrichten für Handshake und Sitzungsverwaltung.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum SessionMessage {
    ClientHello {
        version: ProtocolVersion,
        user_name: String,
    },
    ServerWelcome {
        assigned_id: ClientId,
        version: ProtocolVersion,
    },
    AuthChallenge {
        nonce: u64,
    },
    AuthResult {
        success: bool,
        message: Option<String>,
    },
    VisibilityChanged {
        visible: bool,
    },
    LobbyUpdate {
        slots_taken: u16,
        slots_total: u16,
    },
}

/// Gameplay-bezogene Befehle.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", content = "payload")]
pub enum GameplayCommand {
    InputFrame { frame: u32, data: Vec<u8> },
    Action { action_id: u32, data: Vec<u8> },
    Chat { message: String },
    Custom { payload: Vec<u8> },
}

/// Zustandsupdates (Snapshots oder Deltas).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorldStateUpdate {
    Delta { sequence: u64, data: Vec<u8> },
    Full { data: Vec<u8> },
}

/// Steuerungsframes für Pings, Zeitsync etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlFrame {
    Ping {
        timestamp_micros: u64,
    },
    Pong {
        timestamp_micros: u64,
    },
    BandwidthEstimate {
        bytes_per_second: u32,
    },
    TimeSync {
        server_time_micros: u64,
        client_send_micros: u64,
    },
}

/// Diagnostik-Events (nur Debug).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsEvent {
    pub label: String,
    pub details: String,
}

/// Umschlag für alle logischen Nachrichten.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "class", content = "data")]
pub enum NetworkMessage {
    Session(SessionMessage),
    Command(GameplayCommand),
    State(WorldStateUpdate),
    Control(ControlFrame),
    Diagnostics(DiagnosticsEvent),
}

/// Ausgehende Nachricht plus Kanal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutgoingMessage {
    pub channel: ChannelKind,
    pub message: NetworkMessage,
}

impl OutgoingMessage {
    pub fn new(channel: ChannelKind, message: NetworkMessage) -> Self {
        Self { channel, message }
    }
}
