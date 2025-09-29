//! Transportabstraktion und Adapter für serverseitige Netzwerkeingänge.

use std::fmt::Debug;

use network_shared::{
    events::TransportCapabilities,
    events::{DisconnectReason, TransportEvent},
    ids::ClientId,
    messages::OutgoingMessage,
};
use tokio::sync::mpsc::UnboundedSender;

pub mod quic;

/// Trait, das von allen Server-Transportimplementierungen (QUIC, Steam, etc.) umgesetzt wird.
pub trait ServerTransport: Send + Sync + Debug {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Startet den Transport und leitet Ereignisse in den gegebenen Sender.
    fn start(&mut self, events: UnboundedSender<TransportEvent>) -> Result<(), Self::Error>;

    /// Stoppt den Transport sowie alle aktiven Verbindungen.
    fn stop(&mut self);

    /// Sendet eine Nachricht an den angegebenen Client.
    fn send(&self, client: ClientId, message: OutgoingMessage) -> Result<(), Self::Error>;

    /// Trennt einen Client mit dem angegebenen Grund.
    fn disconnect(&self, client: ClientId, reason: DisconnectReason) -> Result<(), Self::Error>;

    /// Liefert an, welche Fähigkeiten der Transport unterstützt.
    fn capabilities(&self) -> TransportCapabilities;
}
