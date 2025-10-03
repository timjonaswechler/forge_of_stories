//! Shared types and helpers for Steam-based networking transports.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Timeout to wait for Steam relay connections before giving up.
pub const RELAY_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Size of the temporary buffer used when reading Steam networking messages.
pub const MAX_STEAM_PACKET_SIZE: usize = 1_200;

/// Identifier used to tag messages transmitted via Steam transport.
pub const STEAM_CHANNEL_CONTROL: u8 = 0;

/// Wrapper around Steamworks App ID to make intent explicit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SteamAppId(pub u32);

impl SteamAppId {
    pub const fn development() -> Self {
        Self(super::STEAM_APP_ID)
    }
}

/// Steam lobby identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SteamLobbyId(pub u64);

impl SteamLobbyId {
    pub const fn raw(self) -> u64 {
        self.0
    }

    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

/// Visibility for Steam lobbies that can be advertised or hidden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SteamLobbyVisibility {
    Private,
    FriendsOnly,
    Public,
    Invisible,
}

/// Summary information that discovery surfaces for an advertised lobby.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamLobbySummary {
    pub lobby_id: SteamLobbyId,
    pub owner_steam_id: u64,
    pub player_count: u32,
    pub max_players: Option<u32>,
    pub metadata: BTreeMap<String, String>,
}

impl SteamLobbySummary {
    pub fn new(
        lobby_id: SteamLobbyId,
        owner_steam_id: u64,
        player_count: u32,
        max_players: Option<u32>,
        metadata: BTreeMap<String, String>,
    ) -> Self {
        Self {
            lobby_id,
            owner_steam_id,
            player_count,
            max_players,
            metadata,
        }
    }
}

/// Auth ticket payload emitted via discovery/auth flows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SteamAuthTicket {
    pub steam_id: u64,
    pub ticket: Vec<u8>,
}

impl SteamAuthTicket {
    pub fn new(steam_id: u64, ticket: Vec<u8>) -> Self {
        Self { steam_id, ticket }
    }
}

/// Discovery-specific events surfaced on the client transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SteamDiscoveryEvent {
    LobbyFound(SteamLobbySummary),
    LobbyRemoved(SteamLobbyId),
    LobbyListFinished,
    LobbyListFailed(String),
    AuthTicketReceived(SteamAuthTicket),
    AuthTicketRejected(String),
}

/// Messages exchanged on the Steam control channel (channel id 0).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SteamControlMessage {
    AuthRequest(SteamAuthTicket),
}
