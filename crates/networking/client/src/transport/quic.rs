use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};

use bevy::log::warn;
use shared::transport::{ClientTransport, TransportPayload, TransportResult};
use shared::{
    ClientEvent, DisconnectReason, TransportCapabilities, TransportError,
    channels::{ChannelAsyncMessage, ChannelKind, ChannelsConfiguration},
    error::AsyncChannelError,
};
use tokio::runtime::Runtime;

use super::ConnectTarget;
use crate::{
    ClientAsyncMessage, QuinnetClient,
    certificate::{CertificateVerificationMode, KnownHosts, TrustOnFirstUseConfig},
    connection::{ClientSideConnection, ConnectionLocalId, InternalConnectionState},
    error::ClientSendError,
};

pub struct QuicClientTransport {
    runtime: Runtime,
    client: Arc<Mutex<QuinnetClient>>,
    local_bind: SocketAddr,
    cert_mode: CertificateVerificationMode,
    channels: ChannelsConfiguration,
    capabilities: TransportCapabilities,
    connection_id: Option<ConnectionLocalId>,
    pending_events: Vec<ClientEvent>,
}

impl std::fmt::Debug for QuicClientTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuicClientTransport")
            .field("local_bind", &self.local_bind)
            .field("capabilities", &self.capabilities)
            .field("connected", &self.connection_id.is_some())
            .finish()
    }
}

impl QuicClientTransport {
    pub fn new(channels: ChannelsConfiguration, capabilities: TransportCapabilities) -> Self {
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
            pending_events: Vec::new(),
        }
    }

    fn resolve_target(&self, host: &str, port: u16) -> TransportResult<SocketAddr> {
        let addr_str = format!("{host}:{port}");
        addr_str
            .parse()
            .map_err(|_| TransportError::InvalidConfig("invalid quic target"))
    }

    fn collect_events(&mut self, output: &mut Vec<ClientEvent>) -> TransportResult<()> {
        let mut guard = self
            .client
            .lock()
            .map_err(|_| TransportError::Other("quic client mutex poisoned".into()))?;

        for (connection_id, connection) in &mut guard.connections {
            drain_client_messages(*connection_id, connection, output);
        }

        Ok(())
    }

    fn with_connection<R>(
        &self,
        f: impl FnOnce(&mut ClientSideConnection) -> TransportResult<R>,
    ) -> TransportResult<R> {
        let conn_id = self.connection_id.ok_or(TransportError::NotReady)?;
        let mut guard = self
            .client
            .lock()
            .map_err(|_| TransportError::Other("quic client mutex poisoned".into()))?;
        let connection = guard
            .get_connection_mut_by_id(conn_id)
            .ok_or(TransportError::NotReady)?;
        f(connection)
    }

    fn map_send_error(err: ClientSendError) -> TransportError {
        TransportError::Other(format!("failed to send quic payload: {err}"))
    }

    fn map_open_error(err: AsyncChannelError) -> TransportError {
        TransportError::Other(format!("failed to open quic connection: {err}"))
    }

    /// Returns the advertised capabilities for this transport instance.
    pub fn capabilities(&self) -> TransportCapabilities {
        self.capabilities
    }

    /// Receive a gameplay message using the underlying connection API.
    pub fn receive_message<T: serde::de::DeserializeOwned>(
        &mut self,
    ) -> Result<Option<(shared::channels::ChannelId, T)>, crate::error::ClientMessageReceiveError>
    {
        let conn_id = self.connection_id.ok_or(crate::error::ConnectionClosed)?;
        let mut guard = self.client.lock().expect("quic client mutex poisoned");
        let connection = guard
            .get_connection_mut_by_id(conn_id)
            .ok_or(crate::error::ConnectionClosed)?;
        connection.receive_message()
    }

    /// Send a gameplay message using the underlying connection API.
    pub fn send_message_on<T: serde::Serialize>(
        &mut self,
        channel: shared::channels::ChannelId,
        message: T,
    ) -> Result<(), crate::error::ClientMessageSendError> {
        let conn_id = self
            .connection_id
            .ok_or(crate::error::ClientSendError::ConnectionClosed)?;
        let mut guard = self.client.lock().expect("quic client mutex poisoned");
        let connection = guard
            .get_connection_mut_by_id(conn_id)
            .ok_or(crate::error::ClientSendError::ConnectionClosed)?;
        connection.send_message_on(channel, message)?;
        Ok(())
    }
}

impl ClientTransport for QuicClientTransport {
    type ConnectTarget = ConnectTarget;

    fn poll_events(&mut self, output: &mut Vec<ClientEvent>) {
        if !self.pending_events.is_empty() {
            output.extend(self.pending_events.drain(..));
        }

        if let Err(err) = self.collect_events(output) {
            output.push(ClientEvent::Error { error: err });
        }
    }

    fn connect(&mut self, target: Self::ConnectTarget) -> TransportResult<()> {
        if self.connection_id.is_some() {
            return Err(TransportError::Other(
                "quic client already connected".into(),
            ));
        }

        let socket = match target {
            ConnectTarget::Quic { host, port } => self.resolve_target(&host, port)?,
            ConnectTarget::Loopback => {
                return Err(TransportError::InvalidConfig(
                    "loopback target not supported by quic client",
                ));
            }
            ConnectTarget::SteamLobby { .. } => {
                return Err(TransportError::InvalidConfig(
                    "steam lobby target not supported by quic client",
                ));
            }
        };

        let mut guard = self
            .client
            .lock()
            .map_err(|_| TransportError::Other("quic client mutex poisoned".into()))?;

        let endpoint_config = crate::connection::ClientEndpointConfiguration::from_addrs_with_name(
            socket,
            socket.ip().to_string(),
            self.local_bind,
        );

        let connection_id = guard
            .open_connection(
                endpoint_config,
                self.cert_mode.clone(),
                self.channels.clone(),
            )
            .map_err(Self::map_open_error)?;
        guard.set_default_connection(connection_id);

        drop(guard);
        self.connection_id = Some(connection_id);
        Ok(())
    }

    fn disconnect(&mut self) -> TransportResult<()> {
        if self.connection_id.is_none() {
            return Err(TransportError::NotReady);
        }

        let result = self.with_connection(|connection| {
            connection.disconnect().map_err(|err| {
                TransportError::Other(format!("failed to disconnect quic client: {err}"))
            })
        });

        self.connection_id = None;
        self.pending_events.push(ClientEvent::Disconnected {
            reason: DisconnectReason::Graceful,
        });
        result
    }

    fn send(&mut self, payload: TransportPayload) -> TransportResult<()> {
        self.with_connection(|connection| match payload {
            TransportPayload::Message { channel, payload } => connection
                .send_payload_on(channel, payload)
                .map_err(Self::map_send_error),
            TransportPayload::Datagram { payload } => {
                if let Some(channel) = connection.first_unreliable_channel() {
                    connection
                        .send_payload_on(channel, payload)
                        .map_err(Self::map_send_error)
                } else {
                    warn!("attempted to send datagram without unreliable channel configured");
                    Err(TransportError::InvalidConfig(
                        "quic client missing unreliable channel",
                    ))
                }
            }
        })
    }
}

fn default_certificate_mode() -> CertificateVerificationMode {
    let cert_dir = dirs::data_dir().expect("data directory not found");
    let hosts_file = cert_dir.join("known_hosts");
    CertificateVerificationMode::TrustOnFirstUse(TrustOnFirstUseConfig {
        known_hosts: KnownHosts::HostsFile(hosts_file.to_string_lossy().into_owned()),
        ..TrustOnFirstUseConfig::default()
    })
}

fn drain_client_messages(
    connection_id: ConnectionLocalId,
    connection: &mut ClientSideConnection,
    output: &mut Vec<ClientEvent>,
) {
    while let Ok(message) = connection.from_async_client_recv.try_recv() {
        forward_client_async_message(connection_id, connection, message, output);
    }

    while let Ok(message) = connection.from_channels_recv.try_recv() {
        match message {
            ChannelAsyncMessage::LostConnection => {
                if !matches!(connection.state, InternalConnectionState::Disconnected) {
                    connection.try_disconnect_closed_connection();
                    output.push(ClientEvent::Disconnected {
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
                    output.push(ClientEvent::Datagram { payload });
                } else {
                    output.push(ClientEvent::Message { channel, payload });
                }
            }
            Ok(None) => break,
            Err(_err) => {
                if !matches!(connection.state, InternalConnectionState::Disconnected) {
                    connection.try_disconnect_closed_connection();
                }
                output.push(ClientEvent::Disconnected {
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
    output: &mut Vec<ClientEvent>,
) {
    match message {
        ClientAsyncMessage::Connected(internal_connection, client_id) => {
            connection.state = InternalConnectionState::Connected(internal_connection, client_id);
            output.push(ClientEvent::Connected { client_id });
        }
        ClientAsyncMessage::ConnectionFailed(err) => {
            connection.state = InternalConnectionState::Disconnected;
            output.push(ClientEvent::Error {
                error: TransportError::Other(err.to_string()),
            });
            output.push(ClientEvent::Disconnected {
                reason: DisconnectReason::TransportError,
            });
        }
        ClientAsyncMessage::ConnectionClosed => {
            if !matches!(connection.state, InternalConnectionState::Disconnected) {
                connection.try_disconnect_closed_connection();
                output.push(ClientEvent::Disconnected {
                    reason: DisconnectReason::TransportError,
                });
            }
        }
        ClientAsyncMessage::CertificateInteractionRequest { .. } => {
            warn!("Ignoring certificate interaction request in minimal client transport");
            output.push(ClientEvent::Error {
                error: TransportError::Other(
                    "certificate interaction not supported in this build".into(),
                ),
            });
        }
        ClientAsyncMessage::CertificateTrustUpdate(_) => {
            warn!("Ignoring certificate trust update in minimal client transport");
        }
        ClientAsyncMessage::CertificateConnectionAbort { .. } => {
            connection.state = InternalConnectionState::Disconnected;
            output.push(ClientEvent::Error {
                error: TransportError::Other("certificate verification aborted by remote".into()),
            });
            output.push(ClientEvent::Disconnected {
                reason: DisconnectReason::AuthenticationFailed,
            });
        }
    }
}
