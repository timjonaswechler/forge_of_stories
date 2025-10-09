use std::{
    collections::BTreeMap,
    ffi::{CStr, CString},
    sync::{Arc, Mutex},
};

use shared::steam::{SteamAuthTicket, SteamDiscoveryEvent, SteamLobbyId, SteamLobbySummary};
use steamworks::{
    Client, ClientManager,
    matchmaking::LobbyId,
    user::{AuthSessionTicketResponse, AuthTicket},
};
use tokio::sync::mpsc::UnboundedSender;

use super::SteamTransportError;
use shared::ClientEvent;

pub struct SteamLobbyBrowser {
    client: Arc<Client<ClientManager>>,
}

impl SteamLobbyBrowser {
    pub fn new(client: Arc<Client<ClientManager>>) -> Self {
        Self { client }
    }

    pub fn request_lobby_list(
        &self,
        events: UnboundedSender<ClientEvent>,
    ) -> Result<(), SteamTransportError> {
        let client = Arc::clone(&self.client);
        self.client
            .matchmaking()
            .request_lobby_list(move |result| match result {
                Ok(lobbies) => {
                    for lobby in lobbies {
                        let summary = collect_lobby_summary(&client, lobby);
                        let _ = events.send(ClientEvent::Discovery(
                            SteamDiscoveryEvent::LobbyFound(summary),
                        ));
                    }
                    let _ = events.send(ClientEvent::Discovery(
                        SteamDiscoveryEvent::LobbyListFinished,
                    ));
                }
                Err(err) => {
                    let _ = events.send(ClientEvent::Discovery(
                        SteamDiscoveryEvent::LobbyListFailed(err.to_string()),
                    ));
                }
            });
        Ok(())
    }
}

pub struct SteamAuthManager {
    client: Arc<Client<ClientManager>>,
    ticket_state: Arc<Mutex<Option<AuthTicketEntry>>>,
    callback_handle: Mutex<Option<steamworks::CallbackHandle<ClientManager>>>,
    submit_hook: Mutex<Option<Arc<dyn Fn(SteamAuthTicket) + Send + Sync>>>,
}

struct AuthTicketEntry {
    handle: AuthTicket,
    data: Vec<u8>,
    steam_id: u64,
    active: bool,
}

impl SteamAuthManager {
    pub fn new(client: Arc<Client<ClientManager>>) -> Self {
        Self {
            client,
            ticket_state: Arc::new(Mutex::new(None)),
            callback_handle: Mutex::new(None),
            submit_hook: Mutex::new(None),
        }
    }

    pub fn register_callbacks(
        &self,
        events: UnboundedSender<ClientEvent>,
        submit_hook: Option<Arc<dyn Fn(SteamAuthTicket) + Send + Sync>>,
    ) {
        self.drop_callback();
        let ticket_state = Arc::clone(&self.ticket_state);
        let client = Arc::clone(&self.client);
        *self.submit_hook.lock().unwrap() = submit_hook.clone();
        let submit_hook_ref = Arc::clone(&self.submit_hook);
        let handle = self
            .client
            .register_callback::<AuthSessionTicketResponse, _>(move |response| {
                let mut guard = ticket_state.lock().unwrap();
                if let Some(entry) = guard.as_mut() {
                    match response.result {
                        Ok(()) => {
                            entry.active = true;
                            let payload = std::mem::take(&mut entry.data);
                            let ticket = SteamAuthTicket::new(entry.steam_id, payload);
                            let _ = events.send(ClientEvent::Discovery(
                                SteamDiscoveryEvent::AuthTicketReceived(ticket.clone()),
                            ));
                            if let Some(hook) = submit_hook_ref.lock().unwrap().clone() {
                                hook(ticket);
                            }
                        }
                        Err(err) => {
                            if let Some(entry) = guard.take() {
                                client.user().cancel_authentication_ticket(entry.handle);
                            }
                            let _ = events.send(ClientEvent::Discovery(
                                SteamDiscoveryEvent::AuthTicketRejected(err.to_string()),
                            ));
                        }
                    }
                }
            });
        *self.callback_handle.lock().unwrap() = Some(handle);
    }

    pub fn request_ticket(&self) -> Result<(), SteamTransportError> {
        let mut guard = self.ticket_state.lock().unwrap();
        if guard.is_some() {
            return Err(SteamTransportError::AuthTicketFailed(
                "ticket request already pending".into(),
            ));
        }
        let user = self.client.user();
        let steam_id = user.steam_id().raw();
        let (handle, data) = user.authentication_session_ticket();
        *guard = Some(AuthTicketEntry {
            handle,
            data,
            steam_id,
            active: false,
        });
        Ok(())
    }

    pub fn cancel_ticket(&self) {
        let mut guard = self.ticket_state.lock().unwrap();
        if let Some(entry) = guard.take() {
            self.client
                .user()
                .cancel_authentication_ticket(entry.handle);
        }
    }

    pub fn drop_callback(&self) {
        if let Some(handle) = self.callback_handle.lock().unwrap().take() {
            drop(handle);
        }
        *self.submit_hook.lock().unwrap() = None;
    }

    pub fn has_active_ticket(&self) -> bool {
        let guard = self.ticket_state.lock().unwrap();
        guard.as_ref().map(|entry| entry.active).unwrap_or(false)
    }
}

fn collect_lobby_summary(client: &Client<ClientManager>, lobby: LobbyId) -> SteamLobbySummary {
    let matchmaking = client.matchmaking();
    let owner = matchmaking.lobby_owner(lobby).raw();
    let player_count = matchmaking.lobby_member_count(lobby) as u32;
    let max_players = lobby_member_limit(lobby);
    let metadata = lobby_metadata(lobby).unwrap_or_default();
    SteamLobbySummary::new(
        SteamLobbyId::from_raw(lobby.raw()),
        owner,
        player_count,
        max_players,
        metadata,
    )
}

fn lobby_member_limit(lobby: LobbyId) -> Option<u32> {
    let interface = unsafe { steamworks::sys::SteamAPI_SteamMatchmaking_v009() };
    if interface.is_null() {
        return None;
    }
    let limit = unsafe {
        steamworks::sys::SteamAPI_ISteamMatchmaking_GetLobbyMemberLimit(interface, lobby.0)
    };
    if limit <= 0 { None } else { Some(limit as u32) }
}

fn lobby_metadata(lobby: LobbyId) -> Result<BTreeMap<String, String>, SteamTransportError> {
    let interface = unsafe { steamworks::sys::SteamAPI_SteamMatchmaking_v009() };
    if interface.is_null() {
        return Err(SteamTransportError::DiscoveryFailed(
            "Steam matchmaking interface unavailable".into(),
        ));
    }

    let count = unsafe {
        steamworks::sys::SteamAPI_ISteamMatchmaking_GetLobbyDataCount(interface, lobby.0)
    };
    if count <= 0 {
        return Ok(BTreeMap::new());
    }

    let mut metadata = BTreeMap::new();
    for index in 0..count {
        let mut key_buffer = vec![0i8; 256];
        let mut value_buffer = vec![0i8; 1024];
        let ok = unsafe {
            steamworks::sys::SteamAPI_ISteamMatchmaking_GetLobbyDataByIndex(
                interface,
                lobby.0,
                index,
                key_buffer.as_mut_ptr(),
                key_buffer.len() as i32,
                value_buffer.as_mut_ptr(),
                value_buffer.len() as i32,
            )
        };
        if !ok {
            continue;
        }
        let key = unsafe { CStr::from_ptr(key_buffer.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        let value = unsafe { CStr::from_ptr(value_buffer.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        metadata.insert(key, value);
    }

    Ok(metadata)
}
