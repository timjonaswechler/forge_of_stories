use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use bytes::Bytes;
use shared::{
    ClientId, DisconnectReason, OutgoingMessage, TransportCapabilities, TransportError,
    TransportEvent,
    channels::{ChannelId, ChannelKind},
    steam::{STEAM_CHANNEL_CONTROL, SteamAppId, SteamAuthTicket, SteamControlMessage},
};
use steamworks::{
    CallbackHandle, Server, ServerManager, SteamError, SteamId,
    networking::{Networking, P2PSessionConnectFail, P2PSessionRequest, SendType},
};
use tokio::{
    sync::{mpsc::UnboundedSender, oneshot},
    task::JoinHandle,
    time::sleep,
};
use tracing::{debug, info, warn};

use super::auth::SteamAuthValidator;
use crate::transport::ServerTransport;

#[derive(thiserror::Error, Debug)]
pub enum SteamServerTransportError {
    #[error("steamworks initialization failed: {0}")]
    Init(#[from] SteamError),
    #[error("steam server transport already started")]
    AlreadyStarted,
    #[error("unknown client id {0}")]
    UnknownClient(ClientId),
    #[error("steam auth validation error: {0}")]
    AuthValidation(String),
}

#[derive(Debug)]
pub struct SteamServerTransport {
    app_id: SteamAppId,
    server: Arc<Server<ServerManager>>,
    networking: Arc<Networking<ServerManager>>,
    callbacks: Vec<steamworks::CallbackHandle<ServerManager>>,
    callback_runtime: Option<tokio::task::JoinHandle<()>>,
    shutdown: Option<oneshot::Sender<()>>,
    capabilities: TransportCapabilities,
    unreliable_channel: Option<ChannelId>,
    events: Option<UnboundedSender<TransportEvent>>,
    connections: Arc<Mutex<HashMap<ClientId, SteamId>>>,
    steam_to_client: Arc<Mutex<HashMap<SteamId, ClientId>>>,
    next_client_id: Arc<Mutex<ClientId>>,
    poll_task: Option<tokio::task::JoinHandle<()>>,
    auth: Arc<SteamAuthValidator>,
}

impl SteamServerTransport {
    pub fn new_default(channels: &[ChannelKind]) -> Result<Self, SteamServerTransportError> {
        Self::new(SteamAppId::development(), channels)
    }

    pub fn new(
        app_id: SteamAppId,
        channels: &[ChannelKind],
    ) -> Result<Self, SteamServerTransportError> {
        let ip = std::net::Ipv4Addr::UNSPECIFIED;
        let port = 0;
        let game_port = 0;
        let query_port = 0;
        let (server, single) =
            Server::init_app(app_id.0, ip, port, game_port, query_port, &[], None)?;
        let server = Arc::new(server);
        let networking = Arc::new(server.networking());

        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
        let server_clone = Arc::clone(&server);
        let single_clone = Arc::clone(&single);
        let callback_runtime = tokio::spawn(async move {
            loop {
                single_clone.run_callbacks(&server_clone);
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

        let auth = Arc::new(SteamAuthValidator::new(Arc::clone(&server)));

        Ok(Self {
            app_id,
            server,
            networking,
            callbacks: Vec::new(),
            callback_runtime: Some(callback_runtime),
            shutdown: Some(shutdown_tx),
            capabilities,
            unreliable_channel,
            events: None,
            connections: Arc::new(Mutex::new(HashMap::new())),
            steam_to_client: Arc::new(Mutex::new(HashMap::new())),
            next_client_id: Arc::new(Mutex::new(1)),
            poll_task: None,
            auth,
        })
    }

    pub fn validate_auth_ticket(
        &self,
        client: ClientId,
        ticket: SteamAuthTicket,
    ) -> Result<(), SteamServerTransportError> {
        let steam_id = self
            .connections
            .lock()
            .unwrap()
            .get(&client)
            .copied()
            .ok_or(SteamServerTransportError::UnknownClient(client))?;
        self.auth.validate_ticket(client, steam_id, ticket)
    }
}

impl ServerTransport for SteamServerTransport {
    type Error = SteamServerTransportError;

    fn start(&mut self, events: UnboundedSender<TransportEvent>) -> Result<(), Self::Error> {
        if self.events.is_some() {
            return Err(SteamServerTransportError::AlreadyStarted);
        }

        let networking = Arc::clone(&self.networking);
        let steam_to_client = Arc::clone(&self.steam_to_client);
        let connections = Arc::clone(&self.connections);
        let next_id = Arc::clone(&self.next_client_id);
        let events_clone = events.clone();
        let request_handle = self
            .server
            .register_callback::<P2PSessionRequest, _>(move |req| {
                networking.accept_p2p_session(req.remote);
                let mut reverse = steam_to_client.lock().unwrap();
                if let Some(&client_id) = reverse.get(&req.remote) {
                    debug!(
                        "Steam client {:?} re-requested session (client_id {})",
                        req.remote, client_id
                    );
                    return;
                }

                let mut id_guard = next_id.lock().unwrap();
                let client_id = *id_guard;
                *id_guard += 1;
                reverse.insert(req.remote, client_id);
                connections.lock().unwrap().insert(client_id, req.remote);

                let _ = events_clone.send(TransportEvent::PeerConnected { client: client_id });
                info!(
                    "Steam client {:?} accepted with id {}",
                    req.remote, client_id
                );
            });

        let networking = Arc::clone(&self.networking);
        let steam_to_client = Arc::clone(&self.steam_to_client);
        let connections = Arc::clone(&self.connections);
        let events_clone = events.clone();
        let auth_fail = Arc::clone(&self.auth);
        let fail_handle = self
            .server
            .register_callback::<P2PSessionConnectFail, _>(move |fail| {
                if let Some(client_id) = steam_to_client.lock().unwrap().remove(&fail.remote) {
                    connections.lock().unwrap().remove(&client_id);
                    let reason = DisconnectReason::TransportError;
                    let _ = events_clone.send(TransportEvent::PeerDisconnected {
                        client: client_id,
                        reason,
                    });
                    warn!(
                        "Steam client {:?} disconnected (client_id {}, error {})",
                        fail.remote, client_id, fail.error
                    );
                    networking.close_p2p_session(fail.remote);
                    auth_fail.end_session(fail.remote);
                }
            });

        self.callbacks.push(request_handle);
        self.callbacks.push(fail_handle);

        let poll_connections = Arc::clone(&self.connections);
        let poll_reverse = Arc::clone(&self.steam_to_client);
        let poll_networking = Arc::clone(&self.networking);
        let unreliable_channel = self.unreliable_channel;
        let poll_events = events.clone();
        let auth = Arc::clone(&self.auth);
        let poll_task = tokio::spawn(async move {
            run_server_loop(
                poll_networking,
                poll_connections,
                poll_reverse,
                poll_events,
                unreliable_channel,
                auth,
            )
            .await;
        });

        self.poll_task = Some(poll_task);
        self.auth.attach(events.clone());
        self.events = Some(events);

        Ok(())
    }

    fn stop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(handle) = self.callback_runtime.take() {
            handle.abort();
        }
        if let Some(task) = self.poll_task.take() {
            task.abort();
        }
        for handle in self.callbacks.drain(..) {
            drop(handle);
        }

        if let Some(events) = self.events.take() {
            drop(events);
        }

        let mut connections_guard = self.connections.lock().unwrap();
        for steam_id in connections_guard.values() {
            self.networking.close_p2p_session(*steam_id);
            self.auth.end_session(*steam_id);
        }
        connections_guard.clear();
        self.steam_to_client.lock().unwrap().clear();
        self.auth.detach();
    }

    fn send(&self, client: ClientId, message: OutgoingMessage) -> Result<(), Self::Error> {
        let steam_id = self
            .connections
            .lock()
            .unwrap()
            .get(&client)
            .copied()
            .ok_or(SteamServerTransportError::UnknownClient(client))?;

        let mut payload = Vec::with_capacity(1 + message.payload.len());
        payload.push(message.channel);
        payload.extend_from_slice(&message.payload);

        let send_type = if Some(message.channel) == self.unreliable_channel {
            SendType::Unreliable
        } else {
            SendType::Reliable
        };

        if !self
            .networking
            .send_p2p_packet(steam_id, send_type, &payload)
        {
            warn!("failed to send packet to Steam client {:?}", steam_id);
        }

        Ok(())
    }

    fn send_datagram(&self, client: ClientId, payload: Bytes) -> Result<(), Self::Error> {
        let steam_id = self
            .connections
            .lock()
            .unwrap()
            .get(&client)
            .copied()
            .ok_or(SteamServerTransportError::UnknownClient(client))?;

        if !self
            .networking
            .send_p2p_packet(steam_id, SendType::Unreliable, &payload)
        {
            warn!("failed to send datagram to Steam client {:?}", steam_id);
        }

        Ok(())
    }

    fn disconnect(&self, client: ClientId, _reason: DisconnectReason) -> Result<(), Self::Error> {
        let steam_id = self
            .connections
            .lock()
            .unwrap()
            .get(&client)
            .copied()
            .ok_or(SteamServerTransportError::UnknownClient(client))?;

        self.networking.close_p2p_session(steam_id);
        self.auth.end_session(steam_id);
        Ok(())
    }

    fn capabilities(&self) -> TransportCapabilities {
        self.capabilities
    }
}

impl Drop for SteamServerTransport {
    fn drop(&mut self) {
        self.stop();
    }
}

async fn run_server_loop(
    networking: Arc<Networking<ServerManager>>,
    connections: Arc<Mutex<HashMap<ClientId, SteamId>>>,
    reverse: Arc<Mutex<HashMap<SteamId, ClientId>>>,
    events: UnboundedSender<TransportEvent>,
    unreliable_channel: Option<ChannelId>,
    auth: Arc<SteamAuthValidator>,
) {
    let mut buffer = vec![0u8; shared::steam::MAX_STEAM_PACKET_SIZE];
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

                if let Some(client_id) = reverse.lock().unwrap().get(&remote).copied() {
                    let channel = buffer[0];
                    if channel == STEAM_CHANNEL_CONTROL {
                        handle_control_message(&auth, &events, client_id, remote, &buffer[1..read]);
                        continue;
                    }
                    let payload = Bytes::copy_from_slice(&buffer[1..read]);

                    let event = if Some(channel) == unreliable_channel {
                        TransportEvent::Datagram {
                            client: client_id,
                            payload,
                        }
                    } else {
                        TransportEvent::Message {
                            client: client_id,
                            channel,
                            payload,
                        }
                    };

                    let _ = events.send(event);
                } else {
                    debug!("received packet from unknown Steam client {:?}", remote);
                }
            }
        }

        if events.is_closed() {
            break;
        }

        ticker.tick().await;
    }
}

fn handle_control_message(
    auth: &Arc<SteamAuthValidator>,
    events: &UnboundedSender<TransportEvent>,
    client_id: ClientId,
    steam_id: SteamId,
    payload: &[u8],
) {
    match bincode::deserialize::<SteamControlMessage>(payload) {
        Ok(SteamControlMessage::AuthRequest(ticket)) => {
            if let Err(err) = auth.validate_ticket(client_id, steam_id, ticket) {
                let _ = events.send(TransportEvent::Error {
                    client: Some(client_id),
                    error: TransportError::Other(err.to_string()),
                });
            }
        }
        Err(err) => {
            warn!("failed to decode steam control message: {}", err);
            let _ = events.send(TransportEvent::Error {
                client: Some(client_id),
                error: TransportError::Other(err.to_string()),
            });
        }
    }
}
