use std::{
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use bevy::log::{error, warn};
use bytes::Bytes;
use network_shared::{
    ClientEvent, DisconnectReason, OutgoingMessage, TransportCapabilities, TransportError,
    channels::{ChannelAsyncMessage, ChannelKind, ChannelsConfiguration},
    error::AsyncChannelError,
};
use tokio::{
    runtime::Runtime,
    sync::{mpsc::UnboundedSender, oneshot},
    task::JoinHandle,
    time::sleep,
};

use super::{ClientTransport, ConnectTarget};
use crate::{
    ClientAsyncMessage, QuinnetClient,
    certificate::{CertificateVerificationMode, KnownHosts, TrustOnFirstUseConfig},
    connection::{ClientSideConnection, ConnectionLocalId, InternalConnectionState},
    error::ClientSendError,
};

#[derive(thiserror::Error, Debug)]
pub enum QuicClientTransportError {
    #[error("quic client transport already connected")]
    AlreadyConnected,
    #[error("quic client transport not connected")]
    NotConnected,
    #[error("invalid target address")]
    InvalidTarget,
    #[error("open connection failed: {0}")]
    OpenConnection(#[from] AsyncChannelError),
    #[error("send failed: {0}")]
    Send(#[from] ClientSendError),
    #[error("disconnect failed: {0}")]
    Disconnect(String),
    #[error("datagrams are not supported by the quic client transport yet")]
    DatagramsUnsupported,
}

pub struct QuicClientTransport {
    runtime: Runtime,
    client: Arc<Mutex<QuinnetClient>>,
    local_bind: SocketAddr,
    cert_mode: CertificateVerificationMode,
    channels: ChannelsConfiguration,
    capabilities: TransportCapabilities,
    connection_id: Option<ConnectionLocalId>,
    event_sender: Option<UnboundedSender<ClientEvent>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    event_task: Option<JoinHandle<()>>,
}

impl std::fmt::Debug for QuicClientTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuicClientTransport")
            .field("local_bind", &self.local_bind)
            .field("capabilities", &self.capabilities)
            .finish()
    }
}

impl QuicClientTransport {
    pub fn new(channels: ChannelsConfiguration, capabilities: TransportCapabilities) -> Self {
        // Install default crypto provider for rustls if not already set
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime for quic client transport");
        let client = QuinnetClient::new(runtime.handle().clone());

        let cert_mode = default_certificate_mode();

        Self {
            runtime,
            client: Arc::new(Mutex::new(client)),
            local_bind: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
            cert_mode,
            channels,
            capabilities,
            connection_id: None,
            event_sender: None,
            shutdown_tx: None,
            event_task: None,
        }
    }

    fn ensure_event_task(&mut self, events: UnboundedSender<ClientEvent>) {
        if self.event_task.is_some() {
            return;
        }

        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
        let client = Arc::clone(&self.client);

        let handle = self.runtime.spawn(async move {
            loop {
                {
                    let mut guard = match client.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    };

                    for (connection_id, connection) in &mut guard.connections {
                        drain_client_messages(*connection_id, connection, &events);
                    }
                }

                tokio::select! {
                    _ = sleep(Duration::from_millis(16)) => {},
                    _ = &mut shutdown_rx => break,
                }
            }
        });

        self.event_task = Some(handle);
        self.shutdown_tx = Some(shutdown_tx);
    }

    fn stop_event_task(&mut self) {
        if let Some(shutdown) = self.shutdown_tx.take() {
            let _ = shutdown.send(());
        }
        if let Some(handle) = self.event_task.take() {
            handle.abort();
        }
    }

    fn resolve_target(
        &self,
        host: &str,
        port: u16,
    ) -> Result<SocketAddr, QuicClientTransportError> {
        let addr_str = format!("{host}:{port}");
        addr_str
            .parse()
            .map_err(|_| QuicClientTransportError::InvalidTarget)
    }
}

impl ClientTransport for QuicClientTransport {
    type Error = QuicClientTransportError;

    fn connect(
        &mut self,
        target: ConnectTarget,
        events: UnboundedSender<ClientEvent>,
    ) -> Result<(), Self::Error> {
        if self.connection_id.is_some() {
            return Err(QuicClientTransportError::AlreadyConnected);
        }

        let socket = match target {
            ConnectTarget::Quic { host, port } => self.resolve_target(&host, port)?,
            ConnectTarget::Loopback => {
                return Err(QuicClientTransportError::InvalidTarget);
            }
            ConnectTarget::SteamLobby { .. } => {
                return Err(QuicClientTransportError::InvalidTarget);
            }
        };

        let mut guard = self.client.lock().expect("quic client mutex poisoned");

        let endpoint_config = crate::connection::ClientEndpointConfiguration::from_addrs_with_name(
            socket,
            socket.ip().to_string(),
            self.local_bind,
        );

        let connection_id = guard.open_connection(
            endpoint_config,
            self.cert_mode.clone(),
            self.channels.clone(),
        )?;
        guard.set_default_connection(connection_id);

        drop(guard);

        self.event_sender = Some(events.clone());
        self.ensure_event_task(events);
        self.connection_id = Some(connection_id);
        Ok(())
    }

    fn disconnect(&mut self, _reason: DisconnectReason) -> Result<(), Self::Error> {
        let conn_id = self
            .connection_id
            .ok_or(QuicClientTransportError::NotConnected)?;

        let mut guard = self.client.lock().expect("quic client mutex poisoned");
        if let Some(connection) = guard.get_connection_mut_by_id(conn_id) {
            connection
                .disconnect()
                .map_err(|e| QuicClientTransportError::Disconnect(e.to_string()))?;
        }
        drop(guard);
        self.connection_id = None;

        if let Some(sender) = self.event_sender.as_ref() {
            let _ = sender.send(ClientEvent::Disconnected {
                reason: DisconnectReason::Graceful,
            });
        }

        self.stop_event_task();
        self.event_sender = None;
        Ok(())
    }

    fn send(&self, message: OutgoingMessage) -> Result<(), Self::Error> {
        let conn_id = self
            .connection_id
            .ok_or(QuicClientTransportError::NotConnected)?;
        let mut guard = self.client.lock().expect("quic client mutex poisoned");
        let connection = guard
            .get_connection_mut_by_id(conn_id)
            .ok_or(QuicClientTransportError::NotConnected)?;

        connection.send_payload_on(message.channel, message.payload.clone())?;
        Ok(())
    }

    fn send_datagram(&self, payload: Bytes) -> Result<(), Self::Error> {
        let conn_id = self
            .connection_id
            .ok_or(QuicClientTransportError::NotConnected)?;
        let mut guard = self.client.lock().expect("quic client mutex poisoned");
        let connection = guard
            .get_connection_mut_by_id(conn_id)
            .ok_or(QuicClientTransportError::NotConnected)?;

        if let Some(channel) = connection.first_unreliable_channel() {
            connection.send_payload_on(channel, payload)?;
            Ok(())
        } else {
            warn!("attempted to send datagram but no unreliable channel configured");
            Err(QuicClientTransportError::DatagramsUnsupported)
        }
    }

    fn capabilities(&self) -> TransportCapabilities {
        self.capabilities
    }
}

fn default_certificate_mode() -> CertificateVerificationMode {
    let cert_dir = default_cert_directory();
    if let Err(err) = fs::create_dir_all(&cert_dir) {
        warn!(
            "failed to create certificate directory {}: {}",
            cert_dir.display(),
            err
        );
    }

    let hosts_file = cert_dir.join("known_hosts");
    CertificateVerificationMode::TrustOnFirstUse(TrustOnFirstUseConfig {
        known_hosts: KnownHosts::HostsFile(hosts_file.to_string_lossy().into_owned()),
        ..TrustOnFirstUseConfig::default()
    })
}

fn default_cert_directory() -> PathBuf {
    let base = dirs::config_dir()
        .or_else(|| dirs::data_dir())
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("Forge_of_Stories").join("network").join("certs")
}

fn drain_client_messages(
    connection_id: ConnectionLocalId,
    connection: &mut ClientSideConnection,
    events: &UnboundedSender<ClientEvent>,
) {
    while let Ok(message) = connection.from_async_client_recv.try_recv() {
        forward_client_async_message(connection_id, connection, message, events);
    }

    while let Ok(message) = connection.from_channels_recv.try_recv() {
        match message {
            ChannelAsyncMessage::LostConnection => {
                if !matches!(connection.state, InternalConnectionState::Disconnected) {
                    connection.try_disconnect_closed_connection();
                    let _ = events.send(ClientEvent::Disconnected {
                        reason: DisconnectReason::TransportError,
                    });
                }
            }
        }
    }

    loop {
        match connection.receive_payload() {
            Ok(Some((channel, payload))) => {
                if matches!(
                    connection.channel_kind(channel),
                    Some(ChannelKind::Unreliable)
                ) {
                    let _ = events.send(ClientEvent::Datagram { payload });
                } else {
                    let _ = events.send(ClientEvent::Message { channel, payload });
                }
            }
            Ok(None) => break,
            Err(_err) => {
                if !matches!(connection.state, InternalConnectionState::Disconnected) {
                    connection.try_disconnect_closed_connection();
                }
                let _ = events.send(ClientEvent::Disconnected {
                    reason: DisconnectReason::TransportError,
                });
                break;
            }
        }
    }
}

fn forward_client_async_message(
    _connection_id: ConnectionLocalId,
    connection: &mut ClientSideConnection,
    message: ClientAsyncMessage,
    events: &UnboundedSender<ClientEvent>,
) {
    match message {
        ClientAsyncMessage::Connected(internal_connection, client_id) => {
            connection.state = InternalConnectionState::Connected(internal_connection, client_id);
            let _ = events.send(ClientEvent::Connected { client_id });
        }
        ClientAsyncMessage::ConnectionFailed(err) => {
            connection.state = InternalConnectionState::Disconnected;
            let _ = events.send(ClientEvent::Error {
                error: TransportError::Other(err.to_string()),
            });
            let _ = events.send(ClientEvent::Disconnected {
                reason: DisconnectReason::TransportError,
            });
        }
        ClientAsyncMessage::ConnectionClosed => {
            if !matches!(connection.state, InternalConnectionState::Disconnected) {
                connection.try_disconnect_closed_connection();
                let _ = events.send(ClientEvent::Disconnected {
                    reason: DisconnectReason::TransportError,
                });
            }
        }
        ClientAsyncMessage::CertificateInteractionRequest { .. } => {
            warn!("certificate interaction request received but not handled yet");
        }
        ClientAsyncMessage::CertificateTrustUpdate(info) => {
            warn!("certificate trust update ignored: {:?}", info);
        }
        ClientAsyncMessage::CertificateConnectionAbort { status, cert_info } => {
            error!(
                "connection aborted during certificate verification: {:?} {:?}",
                status, cert_info
            );
            connection.try_disconnect_closed_connection();
            connection.state = InternalConnectionState::Disconnected;
            let _ = events.send(ClientEvent::Error {
                error: TransportError::Other(format!(
                    "certificate verification failed: {:?}",
                    status
                )),
            });
            let _ = events.send(ClientEvent::Disconnected {
                reason: DisconnectReason::TransportError,
            });
        }
    }
}
