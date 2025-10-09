use std::{
    collections::HashSet,
    net::IpAddr,
    sync::{Arc, Mutex},
};

use bevy::log::{error, trace};
use shared::transport::{ServerTransport, TransportPayload, TransportResult};
use shared::{
    ClientId, DisconnectReason, TransportCapabilities, TransportError, TransportEvent,
    channels::{ChannelAsyncMessage, ChannelId, ChannelKind, ChannelsConfiguration},
};
use tokio::runtime::Runtime;

use crate::{
    QuinnetServer, ServerAsyncMessage, ServerEndpointConfiguration,
    certificate::CertificateRetrievalMode,
    error::{EndpointAlreadyClosed, EndpointStartError, ServerReceiveError, ServerSendError},
};

/// Quinn-based server transport that wraps the Quinnet server implementation
/// behind the shared transport trait.
pub struct QuicServerTransport {
    _runtime: Runtime,
    server: Arc<Mutex<QuinnetServer>>,
    endpoint_config: ServerEndpointConfiguration,
    cert_mode: CertificateRetrievalMode,
    channels: ChannelsConfiguration,
    capabilities: TransportCapabilities,
    datagram_channel: Option<ChannelId>,
    started: bool,
}

impl std::fmt::Debug for QuicServerTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuicServerTransport")
            .field("endpoint_config", &self.endpoint_config)
            .field("capabilities", &self.capabilities)
            .field("started", &self.started)
            .finish()
    }
}

impl QuicServerTransport {
    pub fn new(
        endpoint_config: ServerEndpointConfiguration,
        channels: ChannelsConfiguration,
        capabilities: TransportCapabilities,
    ) -> TransportResult<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|err| {
                TransportError::Other(format!(
                    "failed to build tokio runtime for quic server: {err}"
                ))
            })?;
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

        Ok(Self {
            _runtime: runtime,
            server: Arc::new(Mutex::new(server)),
            endpoint_config,
            cert_mode,
            channels,
            capabilities,
            datagram_channel,
            started: false,
        })
    }

    fn ensure_started(&mut self) -> TransportResult<()> {
        if self.started {
            return Ok(());
        }

        {
            let mut guard = self
                .server
                .lock()
                .map_err(|_| TransportError::Other("quic server mutex poisoned".into()))?;
            guard
                .start_endpoint(
                    self.endpoint_config.clone(),
                    self.cert_mode.clone(),
                    self.channels.clone(),
                )
                .map_err(map_endpoint_error)?;
        }

        self.started = true;
        Ok(())
    }

    fn with_endpoint<F, R>(&mut self, f: F) -> TransportResult<R>
    where
        F: FnOnce(&mut crate::Endpoint) -> TransportResult<R>,
    {
        let mut guard = self
            .server
            .lock()
            .map_err(|_| TransportError::Other("quic server mutex poisoned".into()))?;
        let endpoint = guard.get_endpoint_mut().ok_or(TransportError::NotReady)?;
        f(endpoint)
    }

    fn collect_events(&mut self, output: &mut Vec<TransportEvent>) -> TransportResult<()> {
        self.ensure_started()?;
        self.with_endpoint(|endpoint| {
            while let Some(message) = endpoint.try_recv_async_message() {
                handle_async_message(endpoint, message, output);
            }

            let mut lost_clients = HashSet::new();
            endpoint.poll_channel_messages(|client_id, message| {
                if matches!(message, ChannelAsyncMessage::LostConnection) {
                    lost_clients.insert(client_id);
                }
            });
            for client_id in lost_clients {
                endpoint.try_disconnect_client(client_id);
                output.push(TransportEvent::PeerDisconnected {
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
                                output.push(TransportEvent::Datagram {
                                    client: client_id,
                                    payload,
                                });
                            } else {
                                output.push(TransportEvent::Message {
                                    client: client_id,
                                    channel: channel_id,
                                    payload,
                                });
                            }
                        }
                        Ok(None) => break,
                        Err(ServerReceiveError::ConnectionClosed) => {
                            endpoint.try_disconnect_closed_client(client_id);
                            output.push(TransportEvent::PeerDisconnected {
                                client: client_id,
                                reason: DisconnectReason::TransportError,
                            });
                            break;
                        }
                        Err(ServerReceiveError::UnknownClient(_)) => break,
                    }
                }
            }
            Ok(())
        })
    }

    fn send_payload(
        endpoint: &mut crate::Endpoint,
        client: ClientId,
        payload: TransportPayload,
        datagram_channel: Option<ChannelId>,
    ) -> TransportResult<()> {
        match payload {
            TransportPayload::Message { channel, payload } => endpoint
                .send_payload_on(client, channel, payload)
                .map_err(map_send_error),
            TransportPayload::Datagram { payload } => {
                let channel = datagram_channel.ok_or(TransportError::InvalidConfig(
                    "quic transport missing unreliable channel",
                ))?;
                endpoint
                    .send_payload_on(client, channel, payload)
                    .map_err(map_send_error)
            }
        }
    }

    /// Returns the advertised capabilities for this transport instance.
    pub fn capabilities(&self) -> TransportCapabilities {
        self.capabilities
    }

    /// Stops the underlying Quinn endpoint. Primarily used for tests.
    pub fn shutdown(&mut self) {
        if let Ok(mut guard) = self.server.lock() {
            if let Err(err) = guard.stop_endpoint() {
                if matches!(err, EndpointAlreadyClosed) {
                    trace!("quic endpoint already closed");
                }
            }
        }
        self.started = false;
    }
}

impl ServerTransport for QuicServerTransport {
    fn poll_events(&mut self, output: &mut Vec<TransportEvent>) {
        if let Err(err) = self.collect_events(output) {
            output.push(TransportEvent::Error {
                client: None,
                error: err,
            });
        }
    }

    fn send(&mut self, client: ClientId, payload: TransportPayload) -> TransportResult<()> {
        self.ensure_started()?;
        let datagram_channel = self.datagram_channel;
        self.with_endpoint(|endpoint| {
            Self::send_payload(endpoint, client, payload, datagram_channel)
        })
    }

    fn broadcast(&mut self, payload: TransportPayload) -> TransportResult<()> {
        self.ensure_started()?;
        let datagram_channel = self.datagram_channel;
        self.with_endpoint(|endpoint| {
            let clients = endpoint.clients();
            for client_id in clients {
                Self::send_payload(endpoint, client_id, payload.clone(), datagram_channel)?;
            }
            Ok(())
        })
    }

    fn broadcast_excluding(
        &mut self,
        exclude: &[ClientId],
        payload: TransportPayload,
    ) -> TransportResult<()> {
        self.ensure_started()?;
        let datagram_channel = self.datagram_channel;
        let exclude: HashSet<_> = exclude.iter().copied().collect();
        self.with_endpoint(|endpoint| {
            let clients = endpoint.clients();
            for client_id in clients {
                if exclude.contains(&client_id) {
                    continue;
                }
                Self::send_payload(endpoint, client_id, payload.clone(), datagram_channel)?;
            }
            Ok(())
        })
    }
}

impl Drop for QuicServerTransport {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn handle_async_message(
    endpoint: &mut crate::Endpoint,
    message: ServerAsyncMessage,
    output: &mut Vec<TransportEvent>,
) {
    match message {
        ServerAsyncMessage::ClientConnected(connection) => {
            match endpoint.handle_connection(connection) {
                Ok(client_id) => {
                    output.push(TransportEvent::PeerConnected { client: client_id });
                }
                Err(err) => {
                    error!("failed to register new connection: {}", err);
                }
            }
        }
        ServerAsyncMessage::ClientConnectionClosed(client_id) => {
            endpoint.try_disconnect_closed_client(client_id);
            output.push(TransportEvent::PeerDisconnected {
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

fn map_endpoint_error(err: EndpointStartError) -> TransportError {
    TransportError::Other(format!("failed to start quic endpoint: {err}"))
}

fn map_send_error(err: ServerSendError) -> TransportError {
    TransportError::Other(format!("failed to send quic payload: {err}"))
}
