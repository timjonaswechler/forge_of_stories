use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use shared::{ClientId, TransportEvent, steam::SteamAuthTicket};
use steamworks::{
    Server, ServerManager, SteamId, server::AuthSessionError, server::ValidateAuthTicketResponse,
};
use tokio::sync::mpsc::UnboundedSender;

use super::SteamServerTransportError;

#[derive(Debug)]
pub struct SteamAuthValidator {
    server: Arc<Server<ServerManager>>,
    events: Mutex<Option<UnboundedSender<TransportEvent>>>,
    pending: Mutex<HashMap<u64, ClientId>>,
    active: Mutex<HashSet<u64>>,
    callback_handle: steamworks::CallbackHandle<ServerManager>,
}

impl SteamAuthValidator {
    pub fn new(server: Arc<Server<ServerManager>>) -> Self {
        let events = Mutex::new(None);
        let pending = Mutex::new(HashMap::new());
        let active = Mutex::new(HashSet::new());
        let events_ref = events.clone();
        let pending_ref = pending.clone();
        let active_ref = active.clone();
        let callback_handle =
            server.register_callback::<ValidateAuthTicketResponse, _>(move |response| {
                let steam_id_raw = response.steam_id.raw();
                let client_id = pending_ref.lock().unwrap().remove(&steam_id_raw);

                if response.response.is_ok() {
                    active_ref.lock().unwrap().insert(steam_id_raw);
                }

                if let Some(sender) = events_ref.lock().unwrap().as_ref() {
                    let _ = sender.send(TransportEvent::AuthResult {
                        client: client_id,
                        steam_id: steam_id_raw,
                        owner_steam_id: response.owner_steam_id.raw(),
                        result: response.response.map(|_| ()).map_err(|err| err.to_string()),
                    });
                }
            });

        Self {
            server,
            events,
            pending,
            active,
            callback_handle,
        }
    }

    pub fn attach(&self, sender: UnboundedSender<TransportEvent>) {
        *self.events.lock().unwrap() = Some(sender);
    }

    pub fn detach(&self) {
        *self.events.lock().unwrap() = None;
    }

    pub fn validate_ticket(
        &self,
        client: ClientId,
        steam_id: SteamId,
        ticket: SteamAuthTicket,
    ) -> Result<(), SteamServerTransportError> {
        if steam_id.raw() != ticket.steam_id {
            return Err(SteamServerTransportError::AuthValidation(
                "steam id mismatch between connection and ticket".into(),
            ));
        }

        let mut pending = self.pending.lock().unwrap();
        if pending.contains_key(&ticket.steam_id) {
            return Err(SteamServerTransportError::AuthValidation(
                "validation already pending for steam id".into(),
            ));
        }

        match self
            .server
            .begin_authentication_session(steam_id, &ticket.ticket)
        {
            Ok(()) => {
                pending.insert(ticket.steam_id, client);
                Ok(())
            }
            Err(AuthSessionError::DuplicateRequest) => {
                // Treat as success; session already active.
                self.active.lock().unwrap().insert(ticket.steam_id);
                if let Some(sender) = self.events.lock().unwrap().as_ref() {
                    let _ = sender.send(TransportEvent::AuthResult {
                        client: Some(client),
                        steam_id: ticket.steam_id,
                        owner_steam_id: steam_id.raw(),
                        result: Ok(()),
                    });
                }
                Ok(())
            }
            Err(err) => {
                if let Some(sender) = self.events.lock().unwrap().as_ref() {
                    let _ = sender.send(TransportEvent::AuthResult {
                        client: Some(client),
                        steam_id: ticket.steam_id,
                        owner_steam_id: steam_id.raw(),
                        result: Err(err.to_string()),
                    });
                }
                Err(SteamServerTransportError::AuthValidation(err.to_string()))
            }
        }
    }

    pub fn end_session(&self, steam_id: SteamId) {
        let raw = steam_id.raw();
        self.pending.lock().unwrap().remove(&raw);
        if self.active.lock().unwrap().remove(&raw) {
            self.server.end_authentication_session(steam_id);
        }
    }
}

impl Drop for SteamAuthValidator {
    fn drop(&mut self) {
        self.detach();
        for steam_id in self.active.lock().unwrap().drain() {
            self.server
                .end_authentication_session(SteamId::from_raw(steam_id));
        }
        let mut pending = self.pending.lock().unwrap();
        for steam_id in pending.keys() {
            self.server
                .end_authentication_session(SteamId::from_raw(*steam_id));
        }
        pending.clear();
        drop(self.callback_handle);
    }
}
