//! Discovery-Logik für verfügbare Server.
//!
//! Der Fokus liegt aktuell auf LAN-Sichtbarkeit. Ein Listener empfängt
//! Broadcast-Pakete der Server, führt ein Timeout-Tracking und meldet
//! Änderungen über eine dedizierte Event-Queue. Die Struktur lässt sich
//! später um Steam Relay oder WAN-Suchen erweitern.

use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use network_shared::{
    config::{DiscoveryConfig, SteamDiscoveryMode},
    discovery::{
        LanPacketDecodeError, LanServerAnnouncement, decode_lan_announcement,
    },
};
use thiserror::Error;
use tokio::{
    net::UdpSocket,
    sync::mpsc::UnboundedSender,
    task::JoinHandle,
};
use tracing::{debug, warn};

use crate::runtime::ClientNetworkRuntime;

const LAN_BUFFER_SIZE: usize = 512;
const LAN_ENTRY_TTL: Duration = Duration::from_secs(6);
const LAN_PRUNE_INTERVAL: Duration = Duration::from_secs(2);

/// Ereignisse, die der Discovery-Listener emittiert.
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    LanServerDiscovered(LanServerInfo),
    LanServerUpdated(LanServerInfo),
    LanServerExpired(SocketAddr),
}

/// Informationen zu einem via LAN gefundenen Server.
#[derive(Debug, Clone)]
pub struct LanServerInfo {
    pub endpoint: SocketAddr,
    pub announcement: LanServerAnnouncement,
}

/// Fehler, die vom Discovery-Subsystem ausgegeben werden können.
#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error while decoding discovery packet: {0}")]
    Serialization(String),
    #[error("invalid config: {0}")]
    InvalidConfig(String),
}

impl From<LanPacketDecodeError> for DiscoveryError {
    fn from(value: LanPacketDecodeError) -> Self {
        match value {
            LanPacketDecodeError::InvalidMagic =>
                DiscoveryError::Serialization("invalid magic".into()),
            LanPacketDecodeError::Serialization(err) =>
                DiscoveryError::Serialization(err.to_string()),
        }
    }
}

/// Stateful Discovery-Komponente für den Client.
#[derive(Debug)]
pub struct ClientDiscovery {
    runtime: ClientNetworkRuntime,
    config: DiscoveryConfig,
    state: DiscoveryState,
    steam: SteamDiscoveryHandle,
}

#[derive(Debug)]
enum DiscoveryState {
    Idle,
    Lan(LanListener),
}

impl ClientDiscovery {
    pub fn new(runtime: ClientNetworkRuntime, config: DiscoveryConfig) -> Self {
        let steam_mode = config.steam.mode.clone();
        Self {
            runtime,
            config,
            state: DiscoveryState::Idle,
            steam: SteamDiscoveryHandle::new(steam_mode),
        }
    }

    /// Liefert die aktuelle Konfiguration.
    pub fn config(&self) -> &DiscoveryConfig {
        &self.config
    }

    /// Gibt eine Momentaufnahme der bekannten LAN-Server zurück.
    pub fn lan_servers(&self) -> Vec<LanServerInfo> {
        match &self.state {
            DiscoveryState::Idle => Vec::new(),
            DiscoveryState::Lan(listener) => listener.snapshot(),
        }
    }

    /// Gibt den aktiven Steam-Discovery-Modus zurück (zur UI-Anzeige oder Debugging).
    pub fn steam_mode(&self) -> &SteamDiscoveryMode {
        self.steam.mode()
    }

    /// Aktualisiert Discovery entsprechend einer neuen Konfiguration.
    pub fn reconfigure(
        &mut self,
        config: DiscoveryConfig,
        events: UnboundedSender<DiscoveryEvent>,
    ) -> Result<(), DiscoveryError> {
        self.config = config;
        self.steam.set_mode(self.config.steam.mode.clone());
        if self.config.lan_broadcast {
            self.start_lan_listener(events)?;
        } else {
            self.stop_lan_listener();
        }
        Ok(())
    }

    /// Beendet alle Listener und bleibt im Hidden-Modus.
    pub fn stop(&mut self) {
        self.stop_lan_listener();
    }

    fn start_lan_listener(
        &mut self,
        events: UnboundedSender<DiscoveryEvent>,
    ) -> Result<(), DiscoveryError> {
        if self.config.lan_port == 0 {
            return Err(DiscoveryError::InvalidConfig(
                "lan_port darf nicht 0 sein".into(),
            ));
        }

        self.stop_lan_listener();
        let listener = LanListener::spawn(&self.runtime, self.config.lan_port, events)?;
        self.state = DiscoveryState::Lan(listener);
        Ok(())
    }

    fn stop_lan_listener(&mut self) {
        if let DiscoveryState::Lan(listener) = std::mem::replace(&mut self.state, DiscoveryState::Idle) {
            listener.abort();
        }
    }
}

/// Verwaltung der LAN-Listener-Task.
#[derive(Debug)]
struct LanListener {
    handle: JoinHandle<()>,
    known: Arc<Mutex<HashMap<SocketAddr, DiscoveredLanServer>>>,
}

#[derive(Debug)]
struct DiscoveredLanServer {
    info: LanServerInfo,
    last_seen: Instant,
}

impl LanListener {
    fn spawn(
        runtime: &ClientNetworkRuntime,
        port: u16,
        events: UnboundedSender<DiscoveryEvent>,
    ) -> Result<Self, DiscoveryError> {
        let socket = bind_lan_socket(port)?;
        let known = Arc::new(Mutex::new(HashMap::new()));

        let known_for_recv = known.clone();
        let events_for_recv = events.clone();
        let handle = runtime.spawn(async move {
            let mut buf = [0u8; LAN_BUFFER_SIZE];
            let mut prune_tick = tokio::time::interval(LAN_PRUNE_INTERVAL);
            loop {
                tokio::select! {
                    result = socket.recv_from(&mut buf) => {
                        match result {
                            Ok((len, source)) => {
                                if let Err(err) = handle_datagram(&buf[..len], source, &known_for_recv, &events_for_recv) {
                                    warn!(target = "network::discovery", "failed to handle discovery packet: {err:?}");
                                }
                            }
                            Err(err) => {
                                warn!(target = "network::discovery", "LAN discovery recv error: {err}");
                                tokio::time::sleep(Duration::from_millis(200)).await;
                            }
                        }
                    }
                    _ = prune_tick.tick() => {
                        prune_expired(&known_for_recv, &events_for_recv);
                    }
                }
            }
        });

        Ok(Self { handle, known })
    }

    fn snapshot(&self) -> Vec<LanServerInfo> {
        let map = self.known.lock().unwrap();
        map.values().map(|entry| entry.info.clone()).collect()
    }

    fn abort(self) {
        self.handle.abort();
    }
}

impl Drop for LanListener {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

fn bind_lan_socket(port: u16) -> Result<UdpSocket, DiscoveryError> {
    let std_socket = std::net::UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port))?;
    std_socket.set_nonblocking(true)?;
    std_socket.set_broadcast(true)?;
    UdpSocket::from_std(std_socket).map_err(DiscoveryError::from)
}

fn handle_datagram(
    bytes: &[u8],
    source: SocketAddr,
    known: &Arc<Mutex<HashMap<SocketAddr, DiscoveredLanServer>>>,
    events: &UnboundedSender<DiscoveryEvent>,
) -> Result<(), DiscoveryError> {
    let announcement = match decode_lan_announcement(bytes) {
        Ok(announcement) => announcement,
        Err(LanPacketDecodeError::InvalidMagic) => return Ok(()),
        Err(other) => return Err(other.into()),
    };

    let endpoint = SocketAddr::new(source.ip(), announcement.port);
    let now = Instant::now();
    let event = {
        use std::collections::hash_map::Entry;
        let mut map = known.lock().unwrap();
        match map.entry(endpoint) {
            Entry::Occupied(mut slot) => {
                slot.get_mut().last_seen = now;
                slot.get_mut().info.announcement = announcement.clone();
                DiscoveryEvent::LanServerUpdated(slot.get().info.clone())
            }
            Entry::Vacant(entry) => {
                let info = LanServerInfo {
                    endpoint,
                    announcement: announcement.clone(),
                };
                entry.insert(DiscoveredLanServer {
                    info: info.clone(),
                    last_seen: now,
                });
                DiscoveryEvent::LanServerDiscovered(info)
            }
        }
    };

    debug!(target = "network::discovery", "LAN server update: {:?}", event);
    let _ = events.send(event);
    Ok(())
}

fn prune_expired(
    known: &Arc<Mutex<HashMap<SocketAddr, DiscoveredLanServer>>>,
    events: &UnboundedSender<DiscoveryEvent>,
) {
    let now = Instant::now();
    let mut expired = Vec::new();
    {
        let mut map = known.lock().unwrap();
        map.retain(|addr, entry| {
            if now.duration_since(entry.last_seen) > LAN_ENTRY_TTL {
                expired.push(*addr);
                false
            } else {
                true
            }
        });
    }

    for addr in expired {
        debug!(target = "network::discovery", "LAN server expired: {addr}");
        let _ = events.send(DiscoveryEvent::LanServerExpired(addr));
    }
}

/// Placeholder-Handle für Steam-basierte Discovery. Solange keine Steamworks-Integration
/// vorliegt, protokolliert er lediglich Moduswechsel und stellt den gewünschten Status bereit.
#[derive(Debug)]
struct SteamDiscoveryHandle {
    mode: SteamDiscoveryMode,
}

impl SteamDiscoveryHandle {
    fn new(mode: SteamDiscoveryMode) -> Self {
        let handle = Self { mode };
        handle.log_state("init");
        handle
    }

    fn set_mode(&mut self, mode: SteamDiscoveryMode) {
        if self.mode != mode {
            self.mode = mode;
            self.log_state("update");
        }
    }

    fn mode(&self) -> &SteamDiscoveryMode {
        &self.mode
    }

    fn log_state(&self, stage: &str) {
        match self.mode {
            SteamDiscoveryMode::Disabled => debug!(
                target = "network::discovery",
                "steam discovery {stage}: disabled"
            ),
            SteamDiscoveryMode::LocalOnly => debug!(
                target = "network::discovery",
                "steam discovery {stage}: local-only placeholder"
            ),
        }
    }
}
