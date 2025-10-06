use std::{collections::BTreeMap, ffi::CString, sync::Arc};

use network_shared::steam::{SteamLobbyId, SteamLobbySummary, SteamLobbyVisibility};
use steamworks::{
    Server, ServerManager,
    matchmaking::{LobbyId, LobbyType},
};
use tokio::sync::oneshot;

#[derive(thiserror::Error, Debug)]
pub enum SteamLobbyError {
    #[error("steam lobby already open")]
    AlreadyOpen,
    #[error("steam lobby is not open")]
    LobbyNotOpen,
    #[error("failed to create Steam lobby: {0}")]
    CreateFailed(String),
    #[error("Steam lobby creation callback dropped before completing")]
    CallbackDropped,
    #[error("failed to flag lobby joinable")]
    SetJoinableFailed,
    #[error("metadata key contains null byte")]
    InvalidMetadataKey,
    #[error("metadata value contains null byte")]
    InvalidMetadataValue,
    #[error("Steam matchmaking rejected metadata for key {0}")]
    MetadataRejected(String),
    #[error("Steam matchmaking interface unavailable")]
    MatchmakingUnavailable,
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

/// Helper responsible for opening/closing the hosting lobby that LAN/Steam clients discover.
pub struct SteamLobbyHost {
    server: Arc<Server<ServerManager>>,
    lobby_id: Option<LobbyId>,
    metadata: BTreeMap<String, String>,
    max_players: Option<u32>,
    joinable: bool,
}

impl SteamLobbyHost {
    pub fn new(server: Arc<Server<ServerManager>>) -> Self {
        Self {
            server,
            lobby_id: None,
            metadata: BTreeMap::new(),
            max_players: None,
            joinable: true,
        }
    }

    pub fn lobby_id(&self) -> Option<SteamLobbyId> {
        self.lobby_id.map(|id| SteamLobbyId::from_raw(id.raw()))
    }

    pub fn joinable(&self) -> bool {
        self.joinable
    }

    pub async fn open(
        &mut self,
        config: SteamLobbyConfig,
    ) -> Result<SteamLobbySummary, SteamLobbyError> {
        if self.lobby_id.is_some() {
            return Err(SteamLobbyError::AlreadyOpen);
        }

        let lobby_type = lobby_visibility_to_type(config.visibility);
        let max_players = config.max_players;
        let joinable = config.joinable;
        let metadata = config.metadata.clone();
        let server = Arc::clone(&self.server);
        let (sender, receiver) = oneshot::channel();

        server
            .matchmaking()
            .create_lobby(lobby_type, max_players, move |result| {
                let outcome = result
                    .map_err(|err| SteamLobbyError::CreateFailed(err.to_string()))
                    .and_then(|lobby_id| {
                        let matchmaking = server.matchmaking();
                        if !matchmaking.set_lobby_joinable(lobby_id, joinable) {
                            return Err(SteamLobbyError::SetJoinableFailed);
                        }
                        apply_metadata(lobby_id, &metadata)?;

                        let owner = matchmaking.lobby_owner(lobby_id).raw();
                        let player_count = matchmaking.lobby_member_count(lobby_id) as u32;
                        let summary = SteamLobbySummary::new(
                            SteamLobbyId::from_raw(lobby_id.raw()),
                            owner,
                            player_count,
                            Some(max_players),
                            metadata.clone(),
                        );
                        Ok((lobby_id, summary))
                    });

                let _ = sender.send(outcome);
            });

        let (lobby_id, summary) = receiver
            .await
            .map_err(|_| SteamLobbyError::CallbackDropped)??;

        self.lobby_id = Some(lobby_id);
        self.metadata = summary.metadata.clone();
        self.max_players = summary.max_players;
        self.joinable = joinable;

        Ok(summary)
    }

    pub fn close(&mut self) {
        if let Some(lobby) = self.lobby_id.take() {
            self.server.matchmaking().leave_lobby(lobby);
        }
        self.metadata.clear();
        self.max_players = None;
        self.joinable = true;
    }

    pub fn set_joinable(&mut self, joinable: bool) -> Result<(), SteamLobbyError> {
        let lobby = self.lobby_id.ok_or(SteamLobbyError::LobbyNotOpen)?;
        if !self
            .server
            .matchmaking()
            .set_lobby_joinable(lobby, joinable)
        {
            return Err(SteamLobbyError::SetJoinableFailed);
        }
        self.joinable = joinable;
        Ok(())
    }

    pub fn update_metadata(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), SteamLobbyError> {
        let lobby = self.lobby_id.ok_or(SteamLobbyError::LobbyNotOpen)?;
        let key = key.into();
        let value = value.into();
        set_lobby_data(lobby, &key, &value)?;
        self.metadata.insert(key, value);
        Ok(())
    }

    pub fn clear_metadata(&mut self, key: &str) -> Result<(), SteamLobbyError> {
        let lobby = self.lobby_id.ok_or(SteamLobbyError::LobbyNotOpen)?;
        delete_lobby_data(lobby, key)?;
        self.metadata.remove(key);
        Ok(())
    }

    pub fn summary(&self) -> Option<SteamLobbySummary> {
        let lobby_id = self.lobby_id?;
        let matchmaking = self.server.matchmaking();
        let owner = matchmaking.lobby_owner(lobby_id).raw();
        let player_count = matchmaking.lobby_member_count(lobby_id) as u32;
        Some(SteamLobbySummary::new(
            SteamLobbyId::from_raw(lobby_id.raw()),
            owner,
            player_count,
            self.max_players,
            self.metadata.clone(),
        ))
    }
}

fn lobby_visibility_to_type(visibility: SteamLobbyVisibility) -> LobbyType {
    match visibility {
        SteamLobbyVisibility::Private => LobbyType::Private,
        SteamLobbyVisibility::FriendsOnly => LobbyType::FriendsOnly,
        SteamLobbyVisibility::Public => LobbyType::Public,
        SteamLobbyVisibility::Invisible => LobbyType::Invisible,
    }
}

fn apply_metadata(
    lobby: LobbyId,
    metadata: &BTreeMap<String, String>,
) -> Result<(), SteamLobbyError> {
    for (key, value) in metadata {
        set_lobby_data(lobby, key, value)?;
    }
    Ok(())
}

fn set_lobby_data(lobby: LobbyId, key: &str, value: &str) -> Result<(), SteamLobbyError> {
    let interface = unsafe { steamworks::sys::SteamAPI_SteamMatchmaking_v009() };
    if interface.is_null() {
        return Err(SteamLobbyError::MatchmakingUnavailable);
    }
    let key_c = CString::new(key).map_err(|_| SteamLobbyError::InvalidMetadataKey)?;
    let value_c = CString::new(value).map_err(|_| SteamLobbyError::InvalidMetadataValue)?;
    let success = unsafe {
        steamworks::sys::SteamAPI_ISteamMatchmaking_SetLobbyData(
            interface,
            lobby.0,
            key_c.as_ptr(),
            value_c.as_ptr(),
        )
    };
    if success {
        Ok(())
    } else {
        Err(SteamLobbyError::MetadataRejected(key.to_owned()))
    }
}

fn delete_lobby_data(lobby: LobbyId, key: &str) -> Result<(), SteamLobbyError> {
    let interface = unsafe { steamworks::sys::SteamAPI_SteamMatchmaking_v009() };
    if interface.is_null() {
        return Err(SteamLobbyError::MatchmakingUnavailable);
    }
    let key_c = CString::new(key).map_err(|_| SteamLobbyError::InvalidMetadataKey)?;
    let success = unsafe {
        steamworks::sys::SteamAPI_ISteamMatchmaking_DeleteLobbyData(
            interface,
            lobby.0,
            key_c.as_ptr(),
        )
    };
    if success {
        Ok(())
    } else {
        Err(SteamLobbyError::MetadataRejected(key.to_owned()))
    }
}
