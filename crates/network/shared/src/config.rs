//! Gemeinsame Konfigurationsstrukturen für Server und Client.

use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::channels::{ChannelDescriptor, ChannelKind, ChannelRegistry};
use crate::events::TransportCapabilities;

/// Konfiguration für Discovery/Sichtbarkeit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub lan_broadcast: bool,
    pub lan_port: u16,
    pub steam_enabled: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            lan_broadcast: false,
            lan_port: 50_000,
            steam_enabled: false,
        }
    }
}

/// Basiskonfiguration für einen Transport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    pub listen_addr: SocketAddr,
    pub max_connections: u32,
    pub idle_timeout_secs: u64,
    pub max_datagram_size: u16,
    pub channels: ChannelRegistry,
}

impl Default for TransportConfig {
    fn default() -> Self {
        let channels = ChannelRegistry::new([
            ChannelDescriptor::new(
                0,
                ChannelKind::ReliableOrdered,
                "reliable-ordered".into(),
                10,
                false,
            ),
            ChannelDescriptor::new(
                1,
                ChannelKind::ReliableUnordered,
                "reliable-unordered".into(),
                5,
                false,
            ),
            ChannelDescriptor::new(
                2,
                ChannelKind::UnreliableSequenced,
                "unreliable-sequenced".into(),
                1,
                true,
            ),
            ChannelDescriptor::new(3, ChannelKind::Control, "control".into(), 15, false),
        ]);

        Self {
            listen_addr: SocketAddr::new(IpAddr::from([0, 0, 0, 0]), 0),
            max_connections: 64,
            idle_timeout_secs: 30,
            max_datagram_size: 1_200,
            channels,
        }
    }
}

/// Serverseitige Netzwerkkonfiguration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerNetworkingConfig {
    pub transport: TransportConfig,
    pub discovery: DiscoveryConfig,
    pub capabilities: TransportCapabilities,
    pub steam_app_id: Option<u32>,
    pub steam_log_on: Option<String>,
    pub tls: ServerTlsConfig,
}

impl Default for ServerNetworkingConfig {
    fn default() -> Self {
        Self {
            transport: TransportConfig::default(),
            discovery: DiscoveryConfig::default(),
            capabilities: TransportCapabilities::default(),
            steam_app_id: None,
            steam_log_on: None,
            tls: ServerTlsConfig::default(),
        }
    }
}

/// Clientseitige Netzwerkkonfiguration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientNetworkingConfig {
    pub transport: TransportConfig,
    pub discovery: DiscoveryConfig,
    pub reconnect_delay_secs: u64,
    pub retry_attempts: u32,
    pub capabilities: TransportCapabilities,
    pub tls: ClientTlsConfig,
}

impl Default for ClientNetworkingConfig {
    fn default() -> Self {
        Self {
            transport: TransportConfig::default(),
            discovery: DiscoveryConfig::default(),
            reconnect_delay_secs: 5,
            retry_attempts: 3,
            capabilities: TransportCapabilities::default(),
            tls: ClientTlsConfig::default(),
        }
    }
}

/// TLS-Identität für den Server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTlsConfig {
    pub mode: ServerTlsMode,
}

impl Default for ServerTlsConfig {
    fn default() -> Self {
        Self {
            mode: ServerTlsMode::SelfSigned {
                subject: "forge-of-stories.local".into(),
            },
        }
    }
}

/// Betriebsmodi für serverseitige TLS-Zertifikate.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerTlsMode {
    /// Entwicklungsmodus: Zertifikat wird beim Start erzeugt.
    SelfSigned { subject: String },
    /// Produktivmodus: Zertifikat + Key werden aus Dateien geladen.
    CertificateFiles {
        certificate: PathBuf,
        private_key: PathBuf,
    },
}

/// TLS-Trust-Store-Konfiguration für Clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientTlsConfig {
    pub trust: ClientTlsTrust,
}

impl Default for ClientTlsConfig {
    fn default() -> Self {
        Self {
            trust: ClientTlsTrust::InsecureSkipVerification,
        }
    }
}

/// Vertrauensstellungen, die der Client akzeptiert.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientTlsTrust {
    /// Nutze das Betriebssystem Trust-Store.
    System,
    /// Lade ein einzelnes CA-Zertifikat von der Platte.
    CertificateFile { ca_certificate: PathBuf },
    /// Entwicklungsmodus: Server-Zertifikate werden nicht geprüft.
    InsecureSkipVerification,
}
