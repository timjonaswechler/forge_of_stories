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
