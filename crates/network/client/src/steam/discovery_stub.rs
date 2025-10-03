use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;

use network_shared::ClientEvent;

use super::SteamTransportError;

pub struct SteamLobbyBrowser;

impl SteamLobbyBrowser {
    pub fn new<T>(_client: Arc<T>) -> Self {
        Self
    }

    pub fn request_lobby_list(
        &self,
        _events: UnboundedSender<ClientEvent>,
    ) -> Result<(), SteamTransportError> {
        Err(SteamTransportError::Disabled)
    }
}

pub struct SteamAuthManager;

impl SteamAuthManager {
    pub fn new<T>(_client: Arc<T>) -> Self {
        Self
    }

    pub fn register_callbacks(
        &self,
        _events: UnboundedSender<ClientEvent>,
        _submit_hook: Option<Arc<dyn Fn(network_shared::steam::SteamAuthTicket) + Send + Sync>>,
    ) {
    }

    pub fn request_ticket(&self) -> Result<(), SteamTransportError> {
        Err(SteamTransportError::Disabled)
    }

    pub fn cancel_ticket(&self) {}

    pub fn has_active_ticket(&self) -> bool {
        false
    }

    pub fn drop_callback(&self) {}
}
