//! Transportabstraktion und Backends für den Client.

use std::fmt::Debug;

use network_shared::{
    events::TransportCapabilities,
    events::{ClientEvent, DisconnectReason},
    messages::OutgoingMessage,
};
use tokio::sync::mpsc::UnboundedSender;

pub mod quic;

/// Gemeinsamer Trait für alle Client-Transporte (QUIC, Steam Relay, ...).
pub trait ClientTransport: Send + Sync + Debug {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Startet eine Verbindung zum Ziel und übergibt Ereignisse an den Sender.
    fn connect(
        &mut self,
        target: ConnectTarget,
        events: UnboundedSender<ClientEvent>,
    ) -> Result<(), Self::Error>;

    /// Trennt die aktuelle Verbindung.
    fn disconnect(&mut self, reason: DisconnectReason) -> Result<(), Self::Error>;

    /// Sendet eine Nachricht an den Server.
    fn send(&self, message: OutgoingMessage) -> Result<(), Self::Error>;

    /// Meldet die Fähigkeiten des Transports.
    fn capabilities(&self) -> TransportCapabilities;
}

/// Abstraktion über die möglichen Verbindungsziele.
#[derive(Debug, Clone)]
pub enum ConnectTarget {
    /// QUIC-Endpunkt (Host/Port) für LAN/WAN.
    Quic { host: String, port: u16 },
    /// Platzhalter für künftige Steam-Relay-Ziele.
    Steam { lobby_id: u64 },
}
