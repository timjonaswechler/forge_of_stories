//! Discovery-Logik für verfügbare Server.
//!
//! Der Fokus liegt aktuell auf LAN-Sichtbarkeit. Ein Listener empfängt
//! Broadcast-Pakete der Server, führt ein Timeout-Tracking und meldet
//! Änderungen über eine dedizierte Event-Queue. Die Struktur lässt sich
//! später um Steam Relay oder WAN-Suchen erweitern.

use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use network_shared::{
    config::{DiscoveryConfig, SteamDiscoveryMode},
    discovery::{
        LanPacketDecodeError, LanServerAnnouncement, SteamLobbyId, SteamLobbyInfo,
        SteamRelayTicket, SteamServerEvent, decode_lan_announcement,
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

#[cfg(feature = "steamworks")]
use steamworks::{Client, ClientManager, LobbyId, LobbyMatchList, LobbyType, SteamError};

const LAN_BUFFER_SIZE: usize = 512;
const LAN_ENTRY_TTL: Duration = Duration::from_secs(6);
const LAN_PRUNE_INTERVAL: Duration = Duration::from_secs(2);

/// Ereignisse, die der Discovery-Listener emittiert.
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    LanServerDiscovered(LanServerInfo),
    LanServerUpdated(LanServerInfo),
    LanServerExpired(SocketAddr),
    SteamStateChanged { mode: SteamDiscoveryMode },
    SteamLobbyDiscovered(SteamLobbyInfo),
    SteamLobbyUpdated(SteamLobbyInfo),
    SteamLobbyRemoved(SteamLobbyId),
    SteamTicketOffered {
        lobby: SteamLobbyId,
        ticket: SteamRelayTicket,
    },
    SteamTicketRevoked(SteamLobbyId),
    SteamAuthApproved(u64),
    SteamAuthRejected(String),
    SteamError(String),
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
    #[error("steam error: {0}")]
    Steam(String),
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
    #[cfg(feature = "steamworks")]
    steamworks: Option<SteamworksBrowser>,
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
            #[cfg(feature = "steamworks")]
            steamworks: None,
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

    /// Gibt die aktuell bekannten Steam-Lobbys zurück.
    pub fn steam_lobbies(&self) -> Vec<SteamLobbyInfo> {
        self.steam.lobbies()
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
        self.steam
            .reconfigure(self.config.steam.mode.clone(), &events);
        if self.config.lan_broadcast {
            self.start_lan_listener(events.clone())?;
        } else {
            self.stop_lan_listener();
        }
        #[cfg(feature = "steamworks")]
        self.configure_steamworks(events)?;
        Ok(())
    }

    /// Verarbeitet ein vom Server geliefertes Steam-Event.
    pub fn handle_steam_server_event(
        &mut self,
        event: SteamServerEvent,
        events: &UnboundedSender<DiscoveryEvent>,
    ) {
        self.steam.handle_event(event, events);
    }

    /// Beendet alle Listener und bleibt im Hidden-Modus.
    pub fn stop(&mut self) {
        self.stop_lan_listener();
        #[cfg(feature = "steamworks")]
        self.stop_steam_listener();
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

    #[cfg(feature = "steamworks")]
    fn configure_steamworks(
        &mut self,
        events: UnboundedSender<DiscoveryEvent>,
    ) -> Result<(), DiscoveryError> {
        if matches!(self.config.steam.mode, SteamDiscoveryMode::Disabled) {
            self.stop_steam_listener();
            return Ok(());
        }

        let app_id = 480; // default Steam test AppID

        if self.steamworks.is_none() {
            match SteamworksBrowser::start(self.runtime.clone(), events, app_id) {
                Ok(browser) => {
                    self.steamworks = Some(browser);
                }
                Err(err) => {
                    warn!(target = "network::discovery", "steamworks init failed: {err}");
                    return Err(DiscoveryError::Steam(err));
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "steamworks")]
    fn stop_steam_listener(&mut self) {
        if let Some(browser) = self.steamworks.take() {
            browser.stop();
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

#[cfg(feature = "steamworks")]
struct SteamworksBrowser {
    client: Client<ClientManager>,
    single: Arc<Mutex<SingleClient>>,
    callback_task: JoinHandle<()>,
    poll_task: JoinHandle<()>,
}

#[cfg(feature = "steamworks")]
impl SteamworksBrowser {
    fn start(
        runtime: ClientNetworkRuntime,
        events: UnboundedSender<DiscoveryEvent>,
        app_id: u32,
    ) -> Result<Self, String> {
        let (client, single) = Client::init_app(app_id)
            .map_err(|err| format!("steam init failed: {err}"))?;

        let single = Arc::new(Mutex::new(single));
        let callback_task = runtime.spawn({
            let single = Arc::clone(&single);
            let events = events.clone();
            async move {
                let mut ticker = tokio::time::interval(Duration::from_millis(50));
                loop {
                    ticker.tick().await;
                    if let Err(err) = single.lock().await.run_callbacks() {
                        warn!(target = "network::discovery", "steam callbacks failed: {err}");
                        let _ = events.send(DiscoveryEvent::SteamError(format!(
                            "steam callbacks failed: {err}"
                        )));
                        break;
                    }
                }
            }
        });

        let poll_task = runtime.spawn(Self::poll_lobbies(client.clone(), events.clone()));

        Ok(Self {
            client,
            single,
            callback_task,
            poll_task,
        })
    }

    async fn poll_lobbies(
        client: Client<ClientManager>,
        events: UnboundedSender<DiscoveryEvent>,
    ) {
        let matchmaking = client.matchmaking();
        let mut known: HashMap<SteamLobbyId, SteamLobbyInfo> = HashMap::new();
        let mut ticker = tokio::time::interval(Duration::from_secs(5));

        loop {
            ticker.tick().await;
            let (sender, receiver) = std::sync::mpsc::channel();
            matchmaking.request_lobby_list(move |result: Result<LobbyMatchList, SteamError>| {
                let converted = result.map(|list| list.into_iter().collect::<Vec<LobbyId>>());
                let _ = sender.send(converted);
            });

            let result = receiver.recv_timeout(Duration::from_secs(5));
            let list = match result {
                Ok(Ok(list)) => list,
                Ok(Err(err)) => {
                    let _ = events.send(DiscoveryEvent::SteamError(format!(
                        "failed to fetch lobby list: {err}"
                    )));
                    continue;
                }
                Err(_) => {
                    let _ = events.send(DiscoveryEvent::SteamError(
                        "timed out fetching lobby list".into(),
                    ));
                    continue;
                }
            };

            let mut current: HashSet<SteamLobbyId> = HashSet::new();

            for lobby_id in list {
                let name = matchmaking
                    .get_lobby_data(lobby_id, "name")
                    .unwrap_or_else(|| "Unknown Lobby".into());
                let owner = matchmaking
                    .get_lobby_owner(lobby_id)
                    .map(|id| id.raw())
                    .unwrap_or(0);
                let members = matchmaking.get_num_lobby_members(lobby_id) as u16;
                let max_members = matchmaking.get_lobby_member_limit(lobby_id) as u16;

                let info = SteamLobbyInfo {
                    lobby_id: SteamLobbyId::new(lobby_id.raw()),
                    host_steam_id: owner,
                    name,
                    player_count: members,
                    max_players: max_members,
                    requires_password: false,
                    relay_enabled: true,
                    wan_visible: false,
                };

                current.insert(info.lobby_id);

                if known
                    .insert(info.lobby_id, info.clone())
                    .is_none()
                {
                    let _ = events.send(DiscoveryEvent::SteamLobbyDiscovered(info.clone()));
                }
                let _ = events.send(DiscoveryEvent::SteamLobbyUpdated(info));
            }

            let removed: Vec<SteamLobbyId> = known
                .keys()
                .copied()
                .filter(|id| !current.contains(id))
                .collect();
            for lobby_id in removed {
                known.remove(&lobby_id);
                let _ = events.send(DiscoveryEvent::SteamLobbyRemoved(lobby_id));
            }
        }
    }

    fn stop(self) {
        self.callback_task.abort();
        self.poll_task.abort();
    }
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
    lobbies: HashMap<SteamLobbyId, SteamLobbyInfo>,
    tickets: HashMap<SteamLobbyId, SteamRelayTicket>,
}

impl SteamDiscoveryHandle {
    fn new(mode: SteamDiscoveryMode) -> Self {
        let handle = Self {
            mode,
            lobbies: HashMap::new(),
            tickets: HashMap::new(),
        };
        handle.log_state("init");
        handle
    }

    fn reconfigure(
        &mut self,
        mode: SteamDiscoveryMode,
        events: &UnboundedSender<DiscoveryEvent>,
    ) {
        if self.mode != mode {
            self.mode = mode;
            self.log_state("update");
        } else {
            self.log_state("noop");
        }
        self.clear(events);
        self.emit_state(events);
    }

    fn handle_event(
        &mut self,
        event: SteamServerEvent,
        events: &UnboundedSender<DiscoveryEvent>,
    ) {
        match event {
            SteamServerEvent::Activated => {
                self.log_state("activated");
                self.emit_state(events);
            }
            SteamServerEvent::Deactivated => {
                self.log_state("deactivated");
                self.clear(events);
                self.emit_state(events);
            }
            SteamServerEvent::LobbyDiscovered(info) => {
                self.log_state("lobby_discovered");
                self.lobbies.insert(info.lobby_id, info.clone());
                let _ = events.send(DiscoveryEvent::SteamLobbyDiscovered(info));
            }
            SteamServerEvent::LobbyUpdated(info) => {
                self.log_state("lobby_updated");
                self.lobbies.insert(info.lobby_id, info.clone());
                let _ = events.send(DiscoveryEvent::SteamLobbyUpdated(info));
            }
            SteamServerEvent::LobbyRemoved(lobby) => {
                self.log_state("lobby_removed");
                self.lobbies.remove(&lobby);
                self.tickets.remove(&lobby);
                let _ = events.send(DiscoveryEvent::SteamLobbyRemoved(lobby));
            }
            SteamServerEvent::TicketIssued(ticket) => {
                self.log_state("ticket_issued");
                self.tickets.insert(ticket.lobby_id, ticket.clone());
                let _ = events.send(DiscoveryEvent::SteamTicketOffered {
                    lobby: ticket.lobby_id,
                    ticket,
                });
            }
            SteamServerEvent::TicketRevoked(lobby) => {
                self.log_state("ticket_revoked");
                self.tickets.remove(&lobby);
                let _ = events.send(DiscoveryEvent::SteamTicketRevoked(lobby));
            }
            SteamServerEvent::AuthApproved { steam_id } => {
                let _ = events.send(DiscoveryEvent::SteamAuthApproved(steam_id));
            }
            SteamServerEvent::AuthRejected { reason } => {
                let _ = events.send(DiscoveryEvent::SteamAuthRejected(reason));
            }
            SteamServerEvent::Error { message } => {
                self.log_state("error");
                let _ = events.send(DiscoveryEvent::SteamError(message));
            }
        }
    }

    fn emit_state(&self, events: &UnboundedSender<DiscoveryEvent>) {
        let event = DiscoveryEvent::SteamStateChanged {
            mode: self.mode.clone(),
        };
        let _ = events.send(event);
    }

    fn clear(&mut self, events: &UnboundedSender<DiscoveryEvent>) {
        for lobby in self.lobbies.keys().copied().collect::<Vec<_>>() {
            let _ = events.send(DiscoveryEvent::SteamLobbyRemoved(lobby));
        }
        self.lobbies.clear();
        self.tickets.clear();
    }

    fn mode(&self) -> &SteamDiscoveryMode {
        &self.mode
    }

    fn lobbies(&self) -> Vec<SteamLobbyInfo> {
        self.lobbies.values().cloned().collect()
    }

    fn log_state(&self, stage: &str) {
        match self.mode {
            SteamDiscoveryMode::Disabled => debug!(
                target = "network::discovery",
                "steam discovery {stage}: disabled"
            ),
            SteamDiscoveryMode::LocalOnly => debug!(
                target = "network::discovery",
                "steam discovery {stage}: local-only active"
            ),
        }
    }
}
