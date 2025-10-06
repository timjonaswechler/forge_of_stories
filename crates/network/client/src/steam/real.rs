use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::Bytes;
use network_shared::{
    ClientEvent, DisconnectReason, OutgoingMessage, TransportCapabilities, TransportError,
    channels::{ChannelId, ChannelKind},
    steam::{
        MAX_STEAM_PACKET_SIZE, STEAM_CHANNEL_CONTROL, SteamAppId, SteamAuthTicket,
        SteamControlMessage,
    },
};
use steamworks::{
    Client, ClientManager, SingleClient, SteamError, SteamId,
    matchmaking::LobbyId,
    networking::{Networking, P2PSessionConnectFail, P2PSessionRequest, SendType},
};
use tokio::{
    sync::{mpsc::UnboundedSender, oneshot},
    task::JoinHandle,
    time::sleep,
};
use tracing::warn;

use super::discovery::{SteamAuthManager, SteamLobbyBrowser};
use crate::transport::{ClientTransport, ConnectTarget};

#[derive(thiserror::Error, Debug)]
pub enum SteamTransportError {
    #[error("steamworks initialization failed: {0}")]
    Init(#[from] SteamError),
    #[error("steam client transport already connected")]
    AlreadyConnected,
    #[error("steam client transport not connected")]
    NotConnected,
    #[error("unsupported transport target: {0:?}")]
    UnsupportedTarget(ConnectTarget),
    #[error("unreliable channel not configured for steam transport")]
    NoUnreliableChannel,
    #[error("steam discovery failed: {0}")]
    DiscoveryFailed(String),
    #[error("steam auth ticket failed: {0}")]
    AuthTicketFailed(String),
    #[error("failed to encode steam control message: {0}")]
    ControlEncode(String),
}

#[derive(Debug)]
pub struct SteamClientTransport {
    app_id: SteamAppId,
    client: Arc<Client<ClientManager>>,
    single: Arc<SingleClient<ClientManager>>,
    networking: Arc<Networking<ClientManager>>,
    callbacks: Vec<steamworks::CallbackHandle<ClientManager>>,
    callback_task: Option<JoinHandle<()>>,
    shutdown: Option<oneshot::Sender<()>>,
    capabilities: TransportCapabilities,
    events: Option<UnboundedSender<ClientEvent>>,
    lobby: Option<LobbyId>,
    connection: Arc<Mutex<Option<SteamId>>>,
    unreliable_channel: Option<ChannelId>,
    poll_task: Option<JoinHandle<()>>,
    lobby_browser: SteamLobbyBrowser,
    auth_manager: SteamAuthManager,
}

impl SteamClientTransport {
    pub fn new_default(channels: &[ChannelKind]) -> Result<Self, SteamTransportError> {
        Self::new(SteamAppId::development(), channels)
    }

    pub fn new(app_id: SteamAppId, channels: &[ChannelKind]) -> Result<Self, SteamTransportError> {
        let (client, single) = Client::init_app(app_id.0)?;
        let client = Arc::new(client);
        let single = Arc::new(single);
        let networking = Arc::new(client.networking());

        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
        let callbacks_client = Arc::clone(&client);
        let callbacks_single = Arc::clone(&single);
        let callback_task = tokio::spawn(async move {
            loop {
                callbacks_single.run_callbacks(&callbacks_client);
                if shutdown_rx.try_recv().is_ok() {
                    break;
                }
                sleep(Duration::from_millis(16)).await;
            }
        });

        let unreliable_channel = channels.iter().enumerate().find_map(|(idx, kind)| {
            if matches!(kind, ChannelKind::Unreliable) {
                Some(idx as ChannelId)
            } else {
                None
            }
        });

        let capabilities = if unreliable_channel.is_some() {
            TransportCapabilities::default()
        } else {
            TransportCapabilities::new(true, false, false, u8::MAX as u16)
        };

        let lobby_browser = SteamLobbyBrowser::new(Arc::clone(&client));
        let auth_manager = SteamAuthManager::new(Arc::clone(&client));

        Ok(Self {
            app_id,
            client,
            single,
            networking,
            callbacks: Vec::new(),
            callback_task: Some(callback_task),
            shutdown: Some(shutdown_tx),
            capabilities,
            events: None,
            lobby: None,
            connection: Arc::new(Mutex::new(None)),
            unreliable_channel,
            poll_task: None,
            lobby_browser,
            auth_manager,
        })
    }
}

impl ClientTransport for SteamClientTransport {
    type Error = SteamTransportError;

    fn connect(
        &mut self,
        target: ConnectTarget,
        events: UnboundedSender<ClientEvent>,
    ) -> Result<(), Self::Error> {
        if self.events.is_some() {
            return Err(SteamTransportError::AlreadyConnected);
        }

        match target {
            ConnectTarget::SteamLobby { lobby_id } => {
                let lobby = LobbyId::from_raw(lobby_id);
                self.events = Some(events.clone());
                self.lobby = Some(lobby);
                {
                    let mut guard = self.connection.lock().unwrap();
                    *guard = None;
                }
                self.register_callbacks();
                let networking = Arc::clone(&self.networking);
                let connection_state = Arc::clone(&self.connection);
                let submit_hook: Arc<dyn Fn(SteamAuthTicket) + Send + Sync> = Arc::new(
                    move |ticket| {
                        let payload = match bincode::serialize(&SteamControlMessage::AuthRequest(
                            ticket.clone(),
                        )) {
                            Ok(data) => data,
                            Err(err) => {
                                warn!("failed to serialize steam auth control message: {}", err);
                                return;
                            }
                        };

                        if let Some(remote) = *connection_state.lock().unwrap() {
                            let mut buffer = Vec::with_capacity(1 + payload.len());
                            buffer.push(STEAM_CHANNEL_CONTROL);
                            buffer.extend_from_slice(&payload);

                            if !networking.send_p2p_packet(remote, SendType::Reliable, &buffer) {
                                warn!(
                                    "failed to send steam auth control packet to {:?} via Networking API",
                                    remote
                                );
                            }
                        } else {
                            warn!(
                                "no steam remote available when attempting to submit auth ticket"
                            );
                        }
                    },
                );
                self.auth_manager
                    .register_callbacks(events.clone(), Some(submit_hook));

                let matchmaking = self.client.matchmaking();
                let client = Arc::clone(&self.client);
                let networking = Arc::clone(&self.networking);
                let events_for_join = events.clone();
                let connection_state = Arc::clone(&self.connection);

                matchmaking.join_lobby(lobby, move |result| match result {
                    Ok(joined) => {
                        let owner = client.matchmaking().lobby_owner(joined);
                        let already_connected = {
                            let mut guard = connection_state.lock().unwrap();
                            let already = guard.is_some();
                            *guard = Some(owner);
                            already
                        };

                        if !networking.send_p2p_packet(
                            owner,
                            SendType::Reliable,
                            &[STEAM_CHANNEL_CONTROL],
                        ) {
                            warn!(
                                "failed to send initial Steam handshake packet to {:?}",
                                owner
                            );
                        }

                        if !already_connected {
                            if let Err(err) =
                                events_for_join.send(ClientEvent::Connected { client_id: None })
                            {
                                warn!("failed to send connected event: {}", err);
                            }
                        }
                    }
                    Err(_) => {
                        {
                            let mut guard = connection_state.lock().unwrap();
                            *guard = None;
                        }

                        if let Err(err) = events_for_join.send(ClientEvent::Error {
                            error: TransportError::Other("failed to join Steam lobby".to_string()),
                        }) {
                            warn!("failed to send lobby error event: {}", err);
                        }
                    }
                });

                let poll_networking = Arc::clone(&self.networking);
                let poll_events = events;
                let poll_connection = Arc::clone(&self.connection);
                let unreliable_channel = self.unreliable_channel;
                let poll_task = tokio::spawn(run_client_loop(
                    poll_networking,
                    poll_events,
                    poll_connection,
                    unreliable_channel,
                ));
                self.poll_task = Some(poll_task);

                if let Err(err) = self.request_auth_ticket() {
                    warn!("steam auth ticket request failed: {}", err);
                }

                Ok(())
            }
            other => Err(SteamTransportError::UnsupportedTarget(other)),
        }
    }

    fn disconnect(&mut self, reason: DisconnectReason) -> Result<(), Self::Error> {
        if let Some(task) = self.poll_task.take() {
            task.abort();
        }

        for handle in self.callbacks.drain(..) {
            drop(handle);
        }

        if let Some(lobby) = self.lobby.take() {
            self.client.matchmaking().leave_lobby(lobby);
        }

        if let Some(remote) = self.connection.lock().unwrap().take() {
            self.networking.close_p2p_session(remote);
        }

        self.auth_manager.cancel_ticket();
        self.auth_manager.drop_callback();

        if let Some(events) = self.events.take() {
            let _ = events.send(ClientEvent::Disconnected { reason });
        }

        Ok(())
    }

    fn send(&self, message: OutgoingMessage) -> Result<(), Self::Error> {
        let remote = self
            .connection
            .lock()
            .unwrap()
            .copied()
            .ok_or(SteamTransportError::NotConnected)?;

        let mut payload = Vec::with_capacity(1 + message.payload.len());
        payload.push(message.channel);
        payload.extend_from_slice(&message.payload);

        let send_type = if Some(message.channel) == self.unreliable_channel {
            SendType::Unreliable
        } else {
            SendType::Reliable
        };

        if !self.networking.send_p2p_packet(remote, send_type, &payload) {
            warn!("failed to send packet to Steam peer {:?}", remote);
        }

        Ok(())
    }

    fn send_datagram(&self, payload: Bytes) -> Result<(), Self::Error> {
        let remote = self
            .connection
            .lock()
            .unwrap()
            .copied()
            .ok_or(SteamTransportError::NotConnected)?;

        let channel = self
            .unreliable_channel
            .ok_or(SteamTransportError::NoUnreliableChannel)?;

        let mut buffer = Vec::with_capacity(1 + payload.len());
        buffer.push(channel);
        buffer.extend_from_slice(&payload);

        if !self
            .networking
            .send_p2p_packet(remote, SendType::Unreliable, &buffer)
        {
            warn!("failed to send datagram to Steam peer {:?}", remote);
        }

        Ok(())
    }

    fn capabilities(&self) -> TransportCapabilities {
        self.capabilities
    }
}

impl Drop for SteamClientTransport {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(handle) = self.callback_task.take() {
            handle.abort();
        }
        if let Some(task) = self.poll_task.take() {
            task.abort();
        }
        for handle in self.callbacks.drain(..) {
            drop(handle);
        }

        if let Some(lobby) = self.lobby.take() {
            self.client.matchmaking().leave_lobby(lobby);
        }

        if let Some(remote) = self.connection.lock().unwrap().take() {
            self.networking.close_p2p_session(remote);
        }

        self.auth_manager.cancel_ticket();
        self.auth_manager.drop_callback();

        self.events = None;
    }
}

fn register_callbacks(&mut self) {
    let events = self.events.as_ref().unwrap().clone();
    let networking = Arc::clone(&self.networking);
    let connection_state = Arc::clone(&self.connection);

    let request_events = events.clone();
    let request_networking = Arc::clone(&self.networking);
    let request_state = Arc::clone(&connection_state);
    let request_handle = self
        .client
        .register_callback::<P2PSessionRequest, _>(move |req| {
            request_networking.accept_p2p_session(req.remote);
            let mut guard = request_state.lock().unwrap();
            let already_connected = guard.is_some();
            *guard = Some(req.remote);
            drop(guard);

            if !already_connected {
                if let Err(err) = request_events.send(ClientEvent::Connected { client_id: None }) {
                    warn!("failed to send connected event: {}", err);
                }
            }
        });

    let fail_events = events;
    let fail_networking = networking;
    let fail_state = connection_state;
    let fail_handle = self
        .client
        .register_callback::<P2PSessionConnectFail, _>(move |fail| {
            fail_networking.close_p2p_session(fail.remote);
            let mut guard = fail_state.lock().unwrap();
            *guard = None;
            drop(guard);

            if let Err(err) = fail_events.send(ClientEvent::Disconnected {
                reason: DisconnectReason::TransportError,
            }) {
                warn!("failed to send disconnect event: {}", err);
            }
        });

    self.callbacks.push(request_handle);
    self.callbacks.push(fail_handle);
}

impl SteamClientTransport {
    pub fn request_lobby_list(&self) -> Result<(), SteamTransportError> {
        let events = self
            .events
            .as_ref()
            .ok_or(SteamTransportError::NotConnected)?
            .clone();
        self.lobby_browser.request_lobby_list(events)
    }

    pub fn request_auth_ticket(&self) -> Result<(), SteamTransportError> {
        self.auth_manager.request_ticket()
    }

    pub fn cancel_auth_ticket(&self) {
        self.auth_manager.cancel_ticket();
    }

    pub fn has_active_auth_ticket(&self) -> bool {
        self.auth_manager.has_active_ticket()
    }

    pub fn submit_auth_ticket(&self, ticket: SteamAuthTicket) -> Result<(), SteamTransportError> {
        let payload = bincode::serialize(&SteamControlMessage::AuthRequest(ticket))
            .map_err(|err| SteamTransportError::ControlEncode(err.to_string()))?;
        self.send(OutgoingMessage::new(STEAM_CHANNEL_CONTROL, payload))
    }
}

async fn run_client_loop(
    networking: Arc<Networking<ClientManager>>,
    events: UnboundedSender<ClientEvent>,
    connection_state: Arc<Mutex<Option<SteamId>>>,
    unreliable_channel: Option<ChannelId>,
) {
    let mut buffer = vec![0u8; MAX_STEAM_PACKET_SIZE];
    let mut ticker = tokio::time::interval(Duration::from_millis(16));
    loop {
        while let Some(size) = networking.is_p2p_packet_available() {
            if size > buffer.len() {
                buffer.resize(size, 0);
            }

            if let Some((remote, read)) = networking.read_p2p_packet(&mut buffer[..size]) {
                if read == 0 {
                    continue;
                }
                {
                    let mut guard = connection_state.lock().unwrap();
                    *guard = Some(remote);
                }

                let channel = buffer[0];
                if channel == STEAM_CHANNEL_CONTROL {
                    continue;
                }

                let payload = Bytes::copy_from_slice(&buffer[1..read]);

                let send_result = if Some(channel) == unreliable_channel {
                    events.send(ClientEvent::Datagram { payload })
                } else {
                    events.send(ClientEvent::Message { channel, payload })
                };

                if send_result.is_err() {
                    return;
                }
            }
        }

        if events.is_closed() {
            break;
        }

        ticker.tick().await;
    }
}
