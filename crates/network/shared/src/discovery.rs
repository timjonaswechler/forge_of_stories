//! Gemeinsame Hilfstypen für Discovery- und Sichtbarkeits-Workflows.
//!
//! Enthält das Nachrichtenformat für LAN-Bekanntmachungen sowie Encoder/
//! Decoder, die sowohl vom Server (Broadcast) als auch vom Client (Listener)
//! eingesetzt werden können.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    messages::{ProtocolVersion, CURRENT_PROTOCOL_VERSION},
    serialization::SerializationError,
};

/// Magic-Bytes, um Discovery-Pakete eindeutig zu identifizieren.
pub const LAN_DISCOVERY_MAGIC: &[u8; 8] = b"FOSDISC1";

/// Paket, das Server periodisch ins LAN senden, um sich sichtbar zu machen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanServerAnnouncement {
    /// Protokollversion des Servers.
    pub version: ProtocolVersion,
    /// Der Port, auf dem der Spielserver QUIC-Verbindungen akzeptiert.
    pub port: u16,
    /// Frei wählbarer Anzeigename für UI-Listen.
    pub server_name: String,
    /// Optional: Aktuelle/Maximale Spielerzahl.
    pub player_capacity: Option<PlayerCapacity>,
    /// Zusatzflags (Steam Relay aktiv, WAN-Endpunkt etc.).
    pub flags: DiscoveryFlags,
}

impl LanServerAnnouncement {
    /// Erzeugt eine Standard-Ankündigung mit aktueller Protokollversion.
    pub fn new(port: u16) -> Self {
        Self {
            version: CURRENT_PROTOCOL_VERSION,
            port,
            server_name: "Forge of Stories Server".into(),
            player_capacity: None,
            flags: DiscoveryFlags::default(),
        }
    }
}

/// Zusatzinformationen über Servermöglichkeiten.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscoveryFlags {
    /// Unterstützt der Server Steam Relay-Verbindungen?
    pub steam_relay: bool,
    /// Wird der Server als WAN-Endpunkt angeboten?
    pub wan_endpoint: bool,
}

/// Optionale Informationen zu belegten/gesamt Slots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCapacity {
    pub current: u16,
    pub max: u16,
}

impl PlayerCapacity {
    pub const fn new(current: u16, max: u16) -> Self {
        Self { current, max }
    }
}

/// Eindeutiger Identifier für eine Steam-Lobby/Session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SteamLobbyId(pub u64);

impl SteamLobbyId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Informationen zu einer via Steam sichtbaren Lobby/Relay-Session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamLobbyInfo {
    pub lobby_id: SteamLobbyId,
    pub host_steam_id: u64,
    pub name: String,
    pub player_count: u16,
    pub max_players: u16,
    pub requires_password: bool,
    pub relay_enabled: bool,
    pub wan_visible: bool,
}

impl SteamLobbyInfo {
    pub fn new(lobby_id: SteamLobbyId) -> Self {
        Self {
            lobby_id,
            host_steam_id: 0,
            name: "Forge of Stories Lobby".into(),
            player_count: 0,
            max_players: 0,
            requires_password: false,
            relay_enabled: true,
            wan_visible: false,
        }
    }
}

/// Ticket, das Clients vom Server erhalten, um Steam Relay zu nutzen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamRelayTicket {
    pub lobby_id: SteamLobbyId,
    pub app_id: u32,
    pub token: Vec<u8>,
    /// Optionaler Ablaufzeitpunkt in Unix-Sekunden.
    pub expires_at: Option<u64>,
}

/// Kodiert eine LAN-Ankündigung inkl. Magic-Bytes.
pub fn encode_lan_announcement(
    announcement: &LanServerAnnouncement,
) -> Result<Vec<u8>, SerializationError> {
    let mut payload = Vec::with_capacity(128);
    payload.extend_from_slice(LAN_DISCOVERY_MAGIC);
    let encoded = bincode::serde::encode_to_vec(announcement, bincode::config::standard())
        .map_err(SerializationError::BincodeEncode)?;
    payload.extend_from_slice(&encoded);
    Ok(payload)
}

/// Fehler, die beim Dekodieren von Discovery-Paketen auftreten können.
#[derive(Debug, Error)]
pub enum LanPacketDecodeError {
    #[error("invalid discovery magic")]
    InvalidMagic,
    #[error("serialization error: {0}")]
    Serialization(#[from] SerializationError),
}

/// Dekodiert ein Discovery-Paket (inkl. Magic-Prüfung).
pub fn decode_lan_announcement(bytes: &[u8]) -> Result<LanServerAnnouncement, LanPacketDecodeError> {
    if bytes.len() < LAN_DISCOVERY_MAGIC.len()
        || &bytes[..LAN_DISCOVERY_MAGIC.len()] != LAN_DISCOVERY_MAGIC
    {
        return Err(LanPacketDecodeError::InvalidMagic);
    }
    let slice = &bytes[LAN_DISCOVERY_MAGIC.len()..];
    let (announcement, _) = bincode::serde::decode_from_slice(slice, bincode::config::standard())
        .map_err(SerializationError::BincodeDecode)?;
    Ok(announcement)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_roundtrip() {
        let mut announcement = LanServerAnnouncement::new(5000);
        announcement.server_name = "Test".into();
        announcement.player_capacity = Some(PlayerCapacity::new(3, 8));
        announcement.flags.steam_relay = true;

        let encoded = encode_lan_announcement(&announcement).unwrap();
        let decoded = decode_lan_announcement(&encoded).unwrap();
        assert_eq!(decoded.server_name, "Test");
        assert_eq!(decoded.port, 5000);
        assert!(decoded.flags.steam_relay);
        assert_eq!(decoded.player_capacity.unwrap().max, 8);
    }
}
