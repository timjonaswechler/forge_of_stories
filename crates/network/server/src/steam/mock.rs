#![cfg(feature = "steamworks-mock")]

use network_shared::discovery::{
    SteamLobbyId, SteamLobbyInfo, SteamRelayTicket, SteamServerEvent,
};

use crate::runtime::NetworkRuntime;

use super::{SteamBackendHandle, SteamIntegration};

/// Simulierte Steam-Integration für Tests/Entwicklung ohne SDK.
pub struct MockSteamIntegration {
    runtime: NetworkRuntime,
    auto_lobby: Option<LobbyConfig>,
    sender: Option<SteamBackendHandle>,
}

#[derive(Debug, Clone)]
pub struct LobbyConfig {
    pub lobby_id: SteamLobbyId,
    pub name: String,
    pub players: (u16, u16),
}

impl MockSteamIntegration {
    pub fn new(runtime: NetworkRuntime) -> Self {
        Self {
            runtime,
            auto_lobby: None,
            sender: None,
        }
    }

    pub fn with_lobby(mut self, config: LobbyConfig) -> Self {
        self.auto_lobby = Some(config);
        self
    }

    fn spawn_lobby(&self, handle: SteamBackendHandle, config: LobbyConfig) {
        let runtime = self.runtime.clone();
        runtime.spawn(async move {
            let lobby = SteamLobbyInfo {
                lobby_id: config.lobby_id,
                host_steam_id: 42,
                name: config.name,
                player_count: config.players.0,
                max_players: config.players.1,
                requires_password: false,
                relay_enabled: true,
                wan_visible: false,
            };
            let _ = handle.send(SteamServerEvent::Activated);
            let _ = handle.send(SteamServerEvent::LobbyDiscovered(lobby.clone()));
            let _ = handle.send(SteamServerEvent::LobbyUpdated(lobby.clone()));
            let ticket = SteamRelayTicket {
                lobby_id: config.lobby_id,
                app_id: 0,
                token: vec![1, 2, 3, 4],
                expires_at: None,
            };
            let _ = handle.send(SteamServerEvent::TicketIssued(ticket));
        });
    }
}

impl SteamIntegration for MockSteamIntegration {
    fn start(&mut self, handle: SteamBackendHandle) -> Result<(), super::SteamIntegrationError> {
        self.sender = Some(handle.clone());
        if let Some(config) = self.auto_lobby.clone() {
            self.spawn_lobby(handle, config);
        } else {
            let _ = handle.send(SteamServerEvent::Activated);
        }
        Ok(())
    }

    fn stop(&mut self) {
        if let Some(handle) = &self.sender {
            let _ = handle.send(SteamServerEvent::Deactivated);
            if let Some(config) = &self.auto_lobby {
                let _ = handle.send(SteamServerEvent::TicketRevoked(config.lobby_id));
                let _ = handle.send(SteamServerEvent::LobbyRemoved(config.lobby_id));
            }
        }
        self.sender = None;
    }
}
