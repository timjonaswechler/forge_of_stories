#![cfg(feature = "steamworks")]

use std::{sync::Arc, time::Duration};

use network_shared::discovery::{SteamLobbyId, SteamLobbyInfo, SteamRelayTicket, SteamServerEvent};
use steamworks::{AuthTicket, Client, ClientManager, Lobby, LobbyId, LobbyType, SingleClient};
use tokio::{sync::Mutex, task::JoinHandle};
use tracing::{debug, warn};

use crate::runtime::NetworkRuntime;

use super::{SteamBackendHandle, SteamIntegration, SteamIntegrationError};

/// Konfiguration für die Steamworks-Integration.
#[derive(Debug, Clone)]
pub struct SteamworksIntegrationConfig {
    pub app_id: u32,
    pub callback_interval: Duration,
    pub max_players: u32,
}

impl Default for SteamworksIntegrationConfig {
    fn default() -> Self {
        Self {
            app_id: 0,
            callback_interval: Duration::from_millis(50),
            max_players: 16,
        }
    }
}

/// Steamworks-basierte Implementierung der [`SteamIntegration`].
pub struct SteamworksIntegration {
    runtime: NetworkRuntime,
    config: SteamworksIntegrationConfig,
    backend_handle: Option<SteamBackendHandle>,
    client: Option<Client<ClientManager>>,
    single: Option<Arc<Mutex<SingleClient>>>,
    callbacks_task: Option<JoinHandle<()>>,
    lobby: Option<Lobby<ClientManager>>,
    lobby_id: Option<LobbyId>,
    auth_ticket: Option<AuthTicket>,
}

impl SteamworksIntegration {
    pub fn new(runtime: NetworkRuntime, config: SteamworksIntegrationConfig) -> Self {
        Self {
            runtime,
            config,
            backend_handle: None,
            client: None,
            single: None,
            callbacks_task: None,
            lobby: None,
            lobby_id: None,
            auth_ticket: None,
        }
    }

    fn spawn_callback_task(
        runtime: &NetworkRuntime,
        single: Arc<Mutex<SingleClient>>,
        handle: SteamBackendHandle,
        interval: Duration,
    ) -> JoinHandle<()> {
        runtime.spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                let result = {
                    let mut guard = single.lock().await;
                    guard.run_callbacks()
                };
                if let Err(err) = result {
                    warn!(target = "network::discovery", "steam callbacks failed: {err}");
                    let _ = handle.send(SteamServerEvent::Error {
                        message: format!("steam callbacks failed: {err}"),
                    });
                    break;
                }
            }
        })
    }

    fn initialise_lobby(
        &mut self,
        handle: &SteamBackendHandle,
    ) -> Result<(), SteamIntegrationError> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| SteamIntegrationError::runtime("steam client missing"))?;

        let matchmaking = client.matchmaking();
        let max_players = self.config.max_players.max(2) as i32;
        let lobby = matchmaking
            .create_lobby(LobbyType::Friends, max_players)
            .map_err(|err| SteamIntegrationError::runtime(err))?;

        let lobby_id = lobby.id();
        let server_name = self.config.server_name.clone();
        let _ = matchmaking.set_lobby_data(&lobby, "name", &server_name);

        let user = client.user();
        let steam_id = user.steam_id().raw();

        let lobby_info = SteamLobbyInfo {
            lobby_id: SteamLobbyId::new(lobby_id.raw()),
            host_steam_id: steam_id,
            name: server_name,
            player_count: 0,
            max_players: max_players as u16,
            requires_password: false,
            relay_enabled: true,
            wan_visible: false,
        };

        let _ = handle.send(SteamServerEvent::LobbyDiscovered(lobby_info.clone()));
        let _ = handle.send(SteamServerEvent::LobbyUpdated(lobby_info));

        if let Ok((ticket, data)) = user.authentication_session_ticket() {
            let relay_ticket = SteamRelayTicket {
                lobby_id: SteamLobbyId::new(lobby_id.raw()),
                app_id: self.config.app_id,
                token: data,
                expires_at: None,
            };
            let _ = handle.send(SteamServerEvent::TicketIssued(relay_ticket));
            self.auth_ticket = Some(ticket);
        }

        self.lobby = Some(lobby);
        self.lobby_id = Some(lobby_id);
        Ok(())
    }
}

impl SteamIntegration for SteamworksIntegration {
    fn start(&mut self, handle: SteamBackendHandle) -> Result<(), SteamIntegrationError> {
        self.stop();

        let app_id = self.config.app_id;
        debug!(target = "network::discovery", "initialising steamworks app_id={app_id}");
        let (client, single) = Client::init_app(app_id)
            .map_err(|err| SteamIntegrationError::start(err))?;

        let single = Arc::new(Mutex::new(single));
        let callbacks = Self::spawn_callback_task(
            &self.runtime,
            single.clone(),
            handle.clone(),
            self.config.callback_interval,
        );

        self.backend_handle = Some(handle.clone());
        self.client = Some(client);
        self.single = Some(single);
        self.callbacks_task = Some(callbacks);

        let _ = handle.send(SteamServerEvent::Activated);
        if let Err(err) = self.initialise_lobby(&handle) {
            warn!(target = "network::discovery", "failed to initialise lobby: {err}");
            let _ = handle.send(SteamServerEvent::Error {
                message: format!("lobby init failed: {err}"),
            });
        }
        Ok(())
    }

    fn stop(&mut self) {
        if let Some(task) = self.callbacks_task.take() {
            task.abort();
        }
        if let Some(client) = &self.client {
            if let Some(lobby) = self.lobby.take() {
                let lobby_id = lobby.id();
                client.matchmaking().leave_lobby(lobby_id);
                if let Some(handle) = &self.backend_handle {
                    let _ = handle.send(SteamServerEvent::LobbyRemoved(SteamLobbyId::new(lobby_id.raw())));
                }
            }
            if let Some(ticket) = self.auth_ticket.take() {
                client.user().cancel_authentication_ticket(ticket);
                if let Some(handle) = &self.backend_handle {
                    if let Some(lobby_id) = self.lobby_id {
                        let _ = handle.send(SteamServerEvent::TicketRevoked(SteamLobbyId::new(lobby_id.raw())));
                    }
                }
            }
        }
        self.single = None;
        self.client = None;
        if let Some(handle) = &self.backend_handle {
            let _ = handle.send(SteamServerEvent::Deactivated);
        }
        self.backend_handle = None;
        self.lobby_id = None;
    }
}

impl Drop for SteamworksIntegration {
    fn drop(&mut self) {
        self.stop();
    }
}
