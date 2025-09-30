//! Clientseitige Ereignis-Typen für Netzwerk & Discovery.
//!
//! Dieses Modul fasst Transport- und Discovery-Ereignisse in einer gemeinsamen
//! Event-Queue zusammen. Gameplay- oder UI-Systeme können so eine einzige
//! `UnboundedReceiver` konsumieren und sowohl Verbindungsänderungen als auch
//! neue LAN-Server beobachten.

use network_shared::events::ClientEvent;
use tokio::sync::mpsc::UnboundedSender;

pub use crate::discovery::{DiscoveryEvent as LanDiscoveryEvent, LanServerInfo};

/// Oberflächen-Eventtyp für den Client-Netzwerkstack.
#[derive(Debug)]
pub enum ClientNetworkEvent {
    Transport(ClientEvent),
    Discovery(LanDiscoveryEvent),
}

/// Convenience-Alias für Event-Sender.
pub type ClientEventSender = UnboundedSender<ClientNetworkEvent>;
