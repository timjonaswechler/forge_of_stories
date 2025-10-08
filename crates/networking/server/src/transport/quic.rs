use std::{
    collections::HashSet,
    fs,
    net::IpAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use bevy::log::{error, trace, warn};
use bytes::Bytes;
use shared::{
    ClientId, DisconnectReason, OutgoingMessage, TransportCapabilities, TransportEvent,
    channels::{ChannelAsyncMessage, ChannelId, ChannelKind, ChannelsConfiguration},
};
use tokio::{
    runtime::Runtime,
    sync::{mpsc::UnboundedSender, oneshot},
    task::JoinHandle,
    time::sleep,
};

use super::ServerTransport;
use crate::{
    QuinnetServer, ServerAsyncMessage, ServerEndpointConfiguration,
    certificate::CertificateRetrievalMode,
    error::{
        EndpointAlreadyClosed, EndpointStartError, ServerDisconnectError, ServerReceiveError,
        ServerSendError,
    },
};

#[derive(thiserror::Error, Debug)]
pub enum QuicServerTransportError {
    #[error("quic server transport already started")]
    AlreadyStarted,
    #[error("quic server transport not started")]
    NotStarted,
    #[error("endpoint start error: {0}")]
    Start(#[from] EndpointStartError),
    #[error("send error: {0}")]
    Send(#[from] ServerSendError),
    #[error("disconnect error: {0}")]
    Disconnect(#[from] ServerDisconnectError),
    #[error("datagrams are not supported by the quic transport yet")]
    DatagramsUnsupported,
}

/// Quinn-based server transport that wraps the Quinnet server implementation
/// behind the shared transport trait.
pub struct QuicServerTransport {
    runtime: Runtime,
    server: Arc<Mutex<QuinnetServer>>,
    endpoint_config: ServerEndpointConfiguration,
    cert_mode: CertificateRetrievalMode,
    channels: ChannelsConfiguration,
    capabilities: TransportCapabilities,
    datagram_channel: Option<ChannelId>,
    event_sender: Option<UnboundedSender<TransportEvent>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    event_task: Option<JoinHandle<()>>,
}

impl std::fmt::Debug for QuicServerTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuicServerTransport")
            .field("endpoint_config", &self.endpoint_config)
            .field("capabilities", &self.capabilities)
            .finish()
    }
}

impl QuicServerTransport {
    pub fn new(
        endpoint_config: ServerEndpointConfiguration,
        channels: ChannelsConfiguration,
        capabilities: TransportCapabilities,
    ) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime for quic server transport");
        let server = QuinnetServer::new(runtime.handle().clone());

        let cert_mode = default_certificate_mode(&endpoint_config);

        let datagram_channel = channels
            .configs()
            .iter()
            .enumerate()
            .find_map(|(idx, kind)| {
                if matches!(kind, ChannelKind::Unreliable) {
                    Some(idx as ChannelId)
                } else {
                    None
                }
            });

        Self {
            runtime,
            server: Arc::new(Mutex::new(server)),
            endpoint_config,
            cert_mode,
            channels,
            capabilities,
            datagram_channel,
            event_sender: None,
            shutdown_tx: None,
            event_task: None,
        }
    }

    fn spawn_event_task(&mut self, events: UnboundedSender<TransportEvent>) -> oneshot::Sender<()> {
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
        let server = Arc::clone(&self.server);

        let handle = self.runtime.spawn(async move {
            loop {
                {
                    let mut guard = match server.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    if let Some(endpoint) = guard.get_endpoint_mut() {
                        while let Some(message) = endpoint.try_recv_async_message() {
                            forward_server_async_message(endpoint, message, &events);
                        }

                        let mut lost_clients = HashSet::new();
                        endpoint.poll_channel_messages(|client_id, message| {
                            if matches!(message, ChannelAsyncMessage::LostConnection) {
                                lost_clients.insert(client_id);
                            }
                        });
                        for client_id in lost_clients.into_iter() {
                            endpoint.try_disconnect_client(client_id);
                            let _ = events.send(TransportEvent::PeerDisconnected {
                                client: client_id,
                                reason: DisconnectReason::TransportError,
                            });
                        }

                        let client_ids = endpoint.clients();
                        for client_id in client_ids {
                            loop {
                                match endpoint.receive_payload_from(client_id) {
                                    Ok(Some((channel_id, payload))) => {
                                        if matches!(
                                            endpoint.channel_kind(channel_id),
                                            Some(ChannelKind::Unreliable)
                                        ) {
                                            let _ = events.send(TransportEvent::Datagram {
                                                client: client_id,
                                                payload,
                                            });
                                        } else {
                                            let _ = events.send(TransportEvent::Message {
                                                client: client_id,
                                                channel: channel_id,
                                                payload,
                                            });
                                        }
                                    }
                                    Ok(None) => break,
                                    Err(ServerReceiveError::ConnectionClosed) => {
                                        endpoint.try_disconnect_closed_client(client_id);
                                        let _ = events.send(TransportEvent::PeerDisconnected {
                                            client: client_id,
                                            reason: DisconnectReason::TransportError,
                                        });
                                        break;
                                    }
                                    Err(ServerReceiveError::UnknownClient(_)) => break,
                                }
                            }
                        }
                    }
                }

                tokio::select! {
                    _ = sleep(Duration::from_millis(16)) => {},
                    _ = &mut shutdown_rx => break,
                }
            }
        });

        self.event_task = Some(handle);
        shutdown_tx
    }
}

impl ServerTransport for QuicServerTransport {
    type Error = QuicServerTransportError;

    fn start(&mut self, events: UnboundedSender<TransportEvent>) -> Result<(), Self::Error> {
        if self.event_task.is_some() {
            return Err(QuicServerTransportError::AlreadyStarted);
        }

        {
            let mut guard = self.server.lock().expect("quic server mutex poisoned");
            guard.start_endpoint(
                self.endpoint_config.clone(),
                self.cert_mode.clone(),
                self.channels.clone(),
            )?;
        }

        self.event_sender = Some(events.clone());
        let shutdown_tx = self.spawn_event_task(events);
        self.shutdown_tx = Some(shutdown_tx);
        Ok(())
    }

    fn stop(&mut self) {
        if let Some(shutdown) = self.shutdown_tx.take() {
            let _ = shutdown.send(());
        }

        if let Some(handle) = self.event_task.take() {
            handle.abort();
        }

        self.event_sender = None;

        if let Ok(mut guard) = self.server.lock() {
            if let Err(err) = guard.stop_endpoint() {
                match err {
                    EndpointAlreadyClosed => trace!("quic endpoint already closed"),
                }
            }
        }
    }

    fn send(&self, client: ClientId, message: OutgoingMessage) -> Result<(), Self::Error> {
        let mut guard = self.server.lock().expect("quic server mutex poisoned");
        let endpoint = guard
            .get_endpoint_mut()
            .ok_or(QuicServerTransportError::NotStarted)?;

        endpoint.send_payload_on(client, message.channel, message.payload.clone())?;
        Ok(())
    }

    fn send_datagram(&self, client: ClientId, payload: Bytes) -> Result<(), Self::Error> {
        let mut guard = self.server.lock().expect("quic server mutex poisoned");
        let endpoint = guard
            .get_endpoint_mut()
            .ok_or(QuicServerTransportError::NotStarted)?;

        if let Some(channel) = self.datagram_channel {
            endpoint.send_payload_on(client, channel, payload)?;
            Ok(())
        } else {
            warn!(
                "attempted to send datagram to client {} without unreliable channel",
                client
            );
            Err(QuicServerTransportError::DatagramsUnsupported)
        }
    }

    fn disconnect(&self, client: ClientId, _reason: DisconnectReason) -> Result<(), Self::Error> {
        let mut guard = self.server.lock().expect("quic server mutex poisoned");
        let endpoint = guard
            .get_endpoint_mut()
            .ok_or(QuicServerTransportError::NotStarted)?;
        endpoint.disconnect_client(client)?;
        Ok(())
    }

    fn capabilities(&self) -> TransportCapabilities {
        self.capabilities
    }
}

fn forward_server_async_message(
    endpoint: &mut crate::Endpoint,
    message: ServerAsyncMessage,
    events: &UnboundedSender<TransportEvent>,
) {
    match message {
        ServerAsyncMessage::ClientConnected(connection) => {
            match endpoint.handle_connection(connection) {
                Ok(client_id) => {
                    let _ = events.send(TransportEvent::PeerConnected { client: client_id });
                }
                Err(err) => {
                    error!("failed to register new connection: {}", err);
                }
            }
        }
        ServerAsyncMessage::ClientConnectionClosed(client_id) => {
            endpoint.try_disconnect_closed_client(client_id);
            let _ = events.send(TransportEvent::PeerDisconnected {
                client: client_id,
                reason: DisconnectReason::TransportError,
            });
        }
    }
}

fn default_certificate_mode(config: &ServerEndpointConfiguration) -> CertificateRetrievalMode {
    // TODO: Implement certificate directory creation and default certificate mode
    let cert_dir = dirs::data_dir().expect("data directory not found");
    let cert_file = cert_dir.join("server.pem");
    let key_file = cert_dir.join("server.key");
    let hostname = match config.local_bind_addr().ip() {
        IpAddr::V4(ip) if !ip.is_unspecified() => ip.to_string(),
        IpAddr::V6(ip) if !ip.is_unspecified() => ip.to_string(),
        _ => "forge-of-stories.local".to_string(),
    };

    CertificateRetrievalMode::LoadFromFileOrGenerateSelfSigned {
        cert_file: cert_file.to_string_lossy().into_owned(),
        key_file: key_file.to_string_lossy().into_owned(),
        server_hostname: hostname,
        save_on_disk: true,
    }
}
