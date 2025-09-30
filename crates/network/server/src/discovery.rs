//! Sichtbarkeits- und Discovery-Logik für den Server.
//!
//! Aktuell wird ein einfacher LAN-Broadcaster umgesetzt, der periodisch ein
//! Discovery-Paket per UDP-Broadcast versendet. Die Struktur ist so angelegt,
//! dass später weitere Modi (Steam Relay, WAN) ergänzt werden können.

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, RwLock},
    time::Duration,
};

use network_shared::{
    config::{DiscoveryConfig, ServerDeployment},
    discovery::{LanServerAnnouncement, PlayerCapacity, encode_lan_announcement},
    serialization::SerializationError,
};
use thiserror::Error;
use tokio::{net::UdpSocket, task::JoinHandle};
use tracing::{debug, warn};

use crate::{
    runtime::NetworkRuntime,
    steam::SteamDiscoveryController,
};

/// Intervall, in dem LAN-Broadcasts ausgesendet werden.
const LAN_BROADCAST_INTERVAL: Duration = Duration::from_millis(750);

/// Aktuelle Sichtbarkeit des Servers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibilityState {
    Hidden,
    Lan,
}

/// Zentrale Verwaltung des Discovery-Zustands.
#[derive(Debug)]
pub struct ServerDiscovery {
    runtime: NetworkRuntime,
    config: DiscoveryConfig,
    deployment: ServerDeployment,
    announcement: Arc<RwLock<LanServerAnnouncement>>,
    payload: Arc<RwLock<Vec<u8>>>,
    state: VisibilityState,
    broadcaster: Option<LanBroadcaster>,
    steam: SteamDiscoveryController,
}

impl ServerDiscovery {
    /// Erstellt eine neue Discovery-Instanz. Standardmäßig wird der Modus aus
    /// der Konfiguration übernommen.
    pub fn new(
        runtime: NetworkRuntime,
        config: DiscoveryConfig,
        deployment: ServerDeployment,
        listen_port: u16,
    ) -> Result<Self, DiscoveryError> {
        let announcement = Arc::new(RwLock::new(LanServerAnnouncement::new(listen_port)));
        let payload = Arc::new(RwLock::new(Vec::new()));
        let steam_mode = config.steam.mode.clone();
        let deployment_mode = deployment.clone();

        let mut discovery = Self {
            runtime,
            config,
            deployment,
            announcement,
            payload,
            state: VisibilityState::Hidden,
            broadcaster: None,
            steam: SteamDiscoveryController::new(deployment_mode, steam_mode),
        };

        if discovery.config.lan_broadcast {
            discovery.enable_lan()?;
        }

        Ok(discovery)
    }

    /// Gibt die aktuelle Sichtbarkeit zurück.
    pub fn visibility(&self) -> VisibilityState {
        self.state
    }

    /// Passt den Listen-Port des QUIC-Servers an (wird im Broadcast verteilt).
    pub fn set_listen_port(&self, port: u16) -> Result<(), DiscoveryError> {
        {
            let mut announcement = self.announcement.write().unwrap();
            announcement.port = port;
        }
        self.refresh_payload()
    }

    /// Setzt den Servernamen, der in Discovery-Listen angezeigt wird.
    pub fn set_server_name<S: Into<String>>(&self, name: S) -> Result<(), DiscoveryError> {
        {
            let mut announcement = self.announcement.write().unwrap();
            announcement.server_name = name.into();
        }
        self.refresh_payload()
    }

    /// Aktualisiert die belegten Slots (z. B. für UI-Lobbys).
    pub fn update_player_capacity(
        &self,
        capacity: Option<(u16, u16)>,
    ) -> Result<(), DiscoveryError> {
        {
            let mut announcement = self.announcement.write().unwrap();
            announcement.player_capacity = capacity.map(|(current, max)| PlayerCapacity::new(current, max));
        }
        self.refresh_payload()
    }

    /// Wendet eine neue Discovery-Konfiguration an und startet/stoppt Broadcasts.
    pub fn reconfigure(
        &mut self,
        config: DiscoveryConfig,
        deployment: ServerDeployment,
    ) -> Result<(), DiscoveryError> {
        self.config = config;
        self.deployment = deployment;
        self.steam
            .set_mode(self.deployment.clone(), self.config.steam.mode.clone());
        if self.config.lan_broadcast {
            self.enable_lan()?;
        } else {
            self.disable_lan();
        }
        Ok(())
    }

    /// Erzwingt einen bestimmten Sichtbarkeitsmodus (z. B. aus Gameplay-Menüs).
    pub fn set_visibility(&mut self, visibility: VisibilityState) -> Result<(), DiscoveryError> {
        match visibility {
            VisibilityState::Hidden => {
                self.config.lan_broadcast = false;
                self.disable_lan();
            }
            VisibilityState::Lan => {
                self.config.lan_broadcast = true;
                self.enable_lan()?;
            }
        }
        Ok(())
    }

    /// Aktualisiert den Deployment-Modus (z. B. wenn ein Server vom lokalen Host in einen
    /// dedizierten Betrieb wechselt).
    pub fn set_deployment(&mut self, deployment: ServerDeployment) -> Result<(), DiscoveryError> {
        self.deployment = deployment;
        self.steam
            .set_mode(self.deployment.clone(), self.config.steam.mode.clone());
        self.refresh_payload()
    }

    /// Liefert eine Kopie der aktuellen Ankündigung (z. B. für Tests oder UI).
    pub fn announcement(&self) -> LanServerAnnouncement {
        self.announcement.read().unwrap().clone()
    }

    fn enable_lan(&mut self) -> Result<(), DiscoveryError> {
        if self.config.lan_port == 0 {
            return Err(DiscoveryError::InvalidConfig(
                "lan_port darf nicht 0 sein".into(),
            ));
        }

        self.refresh_payload()?;

        let payload = self.payload.clone();
        let broadcaster = LanBroadcaster::spawn(
            &self.runtime,
            self.config.lan_port,
            payload,
        )?;

        self.state = VisibilityState::Lan;
        self.broadcaster = Some(broadcaster);
        debug!(target = "network::discovery", "LAN visibility enabled on port {}", self.config.lan_port);
        Ok(())
    }

    fn disable_lan(&mut self) {
        if let Some(broadcaster) = self.broadcaster.take() {
            broadcaster.abort();
        }
        if self.state != VisibilityState::Hidden {
            self.state = VisibilityState::Hidden;
            debug!(target = "network::discovery", "LAN visibility disabled");
        }
    }

    fn refresh_payload(&self) -> Result<(), DiscoveryError> {
        let encoded = {
            let mut announcement = self.announcement.write().unwrap();
            announcement.flags.steam_relay = self.steam_lan_available();
            encode_lan_announcement(&announcement).map_err(DiscoveryError::from)?
        };

        let mut payload = self.payload.write().unwrap();
        *payload = encoded;
        Ok(())
    }

    fn steam_lan_available(&self) -> bool {
        self.steam.is_active()
    }
}
/// Hintergrundaufgabe, die Discovery-Pakete sendet.
#[derive(Debug)]
struct LanBroadcaster {
    handle: JoinHandle<()>,
}

impl LanBroadcaster {
    fn spawn(
        runtime: &NetworkRuntime,
        lan_port: u16,
        payload: Arc<RwLock<Vec<u8>>>,
    ) -> Result<Self, DiscoveryError> {
        let socket = create_broadcast_socket()?;
        let broadcast_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), lan_port);
        let handle = runtime.spawn(async move {
            let mut ticker = tokio::time::interval(LAN_BROADCAST_INTERVAL);
            loop {
                ticker.tick().await;
                let bytes = {
                    let guard = payload.read().unwrap();
                    guard.clone()
                };
                if bytes.is_empty() {
                    continue;
                }
                if let Err(err) = socket.send_to(&bytes, broadcast_addr).await {
                    warn!(target = "network::discovery", "LAN broadcast failed: {err}");
                }
            }
        });

        Ok(Self { handle })
    }

    fn abort(self) {
        self.handle.abort();
    }
}

impl Drop for LanBroadcaster {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

fn create_broadcast_socket() -> Result<UdpSocket, DiscoveryError> {
    let std_socket = std::net::UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0))?;
    std_socket.set_nonblocking(true)?;
    std_socket.set_broadcast(true)?;
    UdpSocket::from_std(std_socket).map_err(DiscoveryError::from)
}

/// Fehler, die während Discovery-Abläufen auftreten können.
#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] SerializationError),
    #[error("invalid config: {0}")]
    InvalidConfig(String),
}
