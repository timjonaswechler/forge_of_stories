use std::collections::BTreeMap;

use network_shared::steam::{SteamLobbyId, SteamLobbySummary, SteamLobbyVisibility};

#[derive(thiserror::Error, Debug)]
pub enum SteamLobbyError {
    #[error("steamworks feature is disabled")]
    Disabled,
}

#[derive(Debug, Clone)]
pub struct SteamLobbyConfig {
    pub max_players: u32,
    pub visibility: SteamLobbyVisibility,
    pub joinable: bool,
    pub metadata: BTreeMap<String, String>,
}

impl Default for SteamLobbyConfig {
    fn default() -> Self {
        Self {
            max_players: 8,
            visibility: SteamLobbyVisibility::FriendsOnly,
            joinable: true,
            metadata: BTreeMap::new(),
        }
    }
}

pub struct SteamLobbyHost;

impl SteamLobbyHost {
    pub fn new<T>(_server: T) -> Self {
        Self
    }

    pub fn lobby_id(&self) -> Option<SteamLobbyId> {
        None
    }

    pub fn joinable(&self) -> bool {
        false
    }

    pub async fn open(
        &mut self,
        _config: SteamLobbyConfig,
    ) -> Result<SteamLobbySummary, SteamLobbyError> {
        Err(SteamLobbyError::Disabled)
    }

    pub fn close(&mut self) {}

    pub fn set_joinable(&mut self, _joinable: bool) -> Result<(), SteamLobbyError> {
        Err(SteamLobbyError::Disabled)
    }

    pub fn update_metadata(
        &mut self,
        _key: impl Into<String>,
        _value: impl Into<String>,
    ) -> Result<(), SteamLobbyError> {
        Err(SteamLobbyError::Disabled)
    }

    pub fn clear_metadata(&mut self, _key: &str) -> Result<(), SteamLobbyError> {
        Err(SteamLobbyError::Disabled)
    }

    pub fn summary(&self) -> Option<SteamLobbySummary> {
        None
    }
}
