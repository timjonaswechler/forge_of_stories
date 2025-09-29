//! QUIC-Transport-Backend für den Server.

use std::{collections::HashMap, fs::File, io::BufReader, net::SocketAddr, sync::Arc};

use bytes::Bytes;
use network_shared::{
    config::{ServerNetworkingConfig, ServerTlsConfig, ServerTlsMode},
    events::{DisconnectReason, TransportCapabilities, TransportError, TransportEvent},
    ids::{ClientId, IdGenerator, SessionId},
    messages::OutgoingMessage,
    serialization::{BincodeSerializer, MessageSerializer, SerializationError},
};
use quinn::rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use quinn::{self, Connection, ConnectionError, Endpoint, Incoming, VarInt};
use rustls_pemfile as pemfile;

use tokio::sync::{RwLock, mpsc::UnboundedSender};

use super::ServerTransport;
use crate::runtime::NetworkRuntime;

const MAX_STREAM_MESSAGE_SIZE: usize = 128 * 1024;
const DEFAULT_SESSION: SessionId = SessionId::new(0);

/// Quinn-basierter Servertransport. Baut Quinn-Endpunkte auf und leitet eingehende Ereignisse in
/// die generischen `TransportEvent`s des Netzwerk-Stacks weiter.
#[derive(Debug)]
pub struct QuicServerTransport<S = BincodeSerializer>
where
    S: MessageSerializer + std::fmt::Debug,
{
    runtime: NetworkRuntime,
    config: ServerNetworkingConfig,
    serializer: Arc<S>,
    endpoint: Option<Endpoint>,
    connections: Arc<RwLock<HashMap<ClientId, Connection>>>,
    next_client_id: IdGenerator,
}

impl<S> QuicServerTransport<S>
where
    S: MessageSerializer + std::fmt::Debug,
{
    pub fn new(runtime: NetworkRuntime, config: ServerNetworkingConfig, serializer: S) -> Self {
        Self {
            runtime,
            config,
            serializer: Arc::new(serializer),
            endpoint: None,
            connections: Arc::new(RwLock::new(HashMap::new())),
            next_client_id: IdGenerator::new(1),
        }
    }

    fn capabilities(&self) -> TransportCapabilities {
        self.config.capabilities
    }

    fn listen_addr(&self) -> SocketAddr {
        self.config.transport.listen_addr
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QuicServerError {
    #[error("quic server transport ist bereits gestartet")]
    AlreadyStarted,
    #[error("quic server transport ist nicht initialisiert")]
    NotStarted,
    #[error("quic server transport konnte nicht gestartet werden: {0}")]
    StartFailed(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] SerializationError),
    #[error("send datagram error: {0}")]
    SendDatagram(#[from] quinn::SendDatagramError),
    #[error("write error: {0}")]
    Write(#[from] quinn::WriteError),
    #[error("open stream error: {0}")]
    OpenStream(#[from] quinn::ConnectionError),
    #[error("unknown client")]
    UnknownClient,
    #[error("tls config error: {0}")]
    TlsConfig(String),
}

impl<S> ServerTransport for QuicServerTransport<S>
where
    S: MessageSerializer + Send + Sync + 'static + std::fmt::Debug,
{
    type Error = QuicServerError;

    fn start(&mut self, events: UnboundedSender<TransportEvent>) -> Result<(), Self::Error> {
        if self.endpoint.is_some() {
            return Err(QuicServerError::AlreadyStarted);
        }

        let server_config = build_server_config(&self.config)?;
        let listen_addr = self.listen_addr();
        let endpoint = quinn::Endpoint::server(server_config, listen_addr)?;

        let accept_endpoint = endpoint.clone();
        let runtime = self.runtime.clone();
        let serializer = self.serializer.clone();
        let connections = self.connections.clone();
        let id_gen = self.next_client_id.clone();
        let events_clone = events.clone();

        self.runtime.spawn(async move {
            run_accept_loop(
                accept_endpoint,
                runtime,
                serializer,
                connections,
                events_clone,
                id_gen,
            )
            .await;
        });

        self.endpoint = Some(endpoint);
        Ok(())
    }

    fn stop(&mut self) {
        if let Some(endpoint) = self.endpoint.take() {
            endpoint.close(0u32.into(), b"shutdown");
        }
    }

    fn send(&self, client: ClientId, message: OutgoingMessage) -> Result<(), Self::Error> {
        let bytes = self.serializer.serialize(&message)?;
        let is_datagram = matches!(
            message.channel,
            network_shared::channels::ChannelKind::UnreliableSequenced
        );

        let connections = self.connections.clone();
        let fut = async move {
            let guard = connections.read().await;
            let connection = guard.get(&client).ok_or(QuicServerError::UnknownClient)?;
            if is_datagram {
                connection.send_datagram(Bytes::from(bytes))?;
            } else {
                let mut stream = connection.open_uni().await?;
                let len = bytes.len() as u32;
                stream.write_all(&len.to_be_bytes()).await?;
                stream.write_all(&bytes).await?;
                let _ = stream.finish();
            }
            Ok::<(), QuicServerError>(())
        };

        self.runtime.handle().block_on(fut)?;

        Ok(())
    }

    fn disconnect(&self, client: ClientId, reason: DisconnectReason) -> Result<(), Self::Error> {
        let connections = self.connections.clone();
        let fut = async move {
            let mut guard = connections.write().await;
            if let Some(connection) = guard.remove(&client) {
                let code = disconnect_code(reason);
                connection.close(code, format!("{reason:?}").as_bytes());
            }
            Ok::<(), QuicServerError>(())
        };
        self.runtime.handle().block_on(fut)?;
        Ok(())
    }

    fn capabilities(&self) -> TransportCapabilities {
        self.capabilities()
    }
}

async fn run_accept_loop<S>(
    endpoint: Endpoint,
    runtime: NetworkRuntime,
    serializer: Arc<S>,
    connections: Arc<RwLock<HashMap<ClientId, Connection>>>,
    events: UnboundedSender<TransportEvent>,
    id_gen: IdGenerator,
) where
    S: MessageSerializer + Send + Sync + 'static + std::fmt::Debug,
{
    loop {
        match endpoint.accept().await {
            Some(incoming) => {
                handle_incoming(
                    incoming,
                    runtime.clone(),
                    serializer.clone(),
                    connections.clone(),
                    events.clone(),
                    id_gen.clone(),
                )
                .await
            }
            None => break,
        }
    }
    endpoint.wait_idle().await;
}

async fn handle_incoming<S>(
    incoming: Incoming,
    runtime: NetworkRuntime,
    serializer: Arc<S>,
    connections: Arc<RwLock<HashMap<ClientId, Connection>>>,
    events: UnboundedSender<TransportEvent>,
    id_gen: IdGenerator,
) where
    S: MessageSerializer + Send + Sync + 'static + std::fmt::Debug,
{
    match incoming.accept() {
        Ok(connecting) => {
            let client_id = ClientId::new(id_gen.next());
            let runtime_clone = runtime.clone();
            runtime.spawn(handle_connection(
                connecting,
                runtime_clone,
                serializer,
                connections,
                events,
                DEFAULT_SESSION,
                client_id,
            ));
        }
        Err(err) => {
            let _ = events.send(TransportEvent::Error {
                session: Some(DEFAULT_SESSION),
                client: None,
                error: TransportError::Other(err.to_string()),
            });
        }
    }
}

async fn handle_connection<S>(
    connecting: quinn::Connecting,
    runtime: NetworkRuntime,
    serializer: Arc<S>,
    connections: Arc<RwLock<HashMap<ClientId, Connection>>>,
    events: UnboundedSender<TransportEvent>,
    session: SessionId,
    client: ClientId,
) where
    S: MessageSerializer + Send + Sync + 'static + std::fmt::Debug,
{
    match connecting.await {
        Ok(connection) => {
            {
                let mut guard = connections.write().await;
                guard.insert(client, connection.clone());
            }
            let _ = events.send(TransportEvent::PeerConnected { session, client });

            let events_clone = events.clone();
            runtime.spawn(read_datagrams(
                connection.clone(),
                serializer.clone(),
                events_clone,
                session,
                client,
            ));

            let events_clone = events.clone();
            runtime.spawn(read_uni_streams(
                connection.clone(),
                serializer.clone(),
                events_clone,
                session,
                client,
            ));

            let connections_clone = connections.clone();
            runtime.spawn(async move {
                let reason = connection.closed().await;
                {
                    let mut guard = connections_clone.write().await;
                    guard.remove(&client);
                }
                let disconnect_reason = map_close_reason(&reason);
                let _ = events.send(TransportEvent::PeerDisconnected {
                    session,
                    client,
                    reason: disconnect_reason,
                });
            });
        }
        Err(err) => {
            let _ = events.send(TransportEvent::Error {
                session: Some(session),
                client: Some(client),
                error: TransportError::Other(err.to_string()),
            });
        }
    }
}

async fn read_datagrams<S>(
    connection: Connection,
    serializer: Arc<S>,
    events: UnboundedSender<TransportEvent>,
    session: SessionId,
    client: ClientId,
) where
    S: MessageSerializer + Send + Sync + 'static + std::fmt::Debug,
{
    loop {
        match connection.read_datagram().await {
            Ok(bytes) => match serializer.deserialize::<OutgoingMessage>(&bytes) {
                Ok(msg) => {
                    let _ = events.send(TransportEvent::Message {
                        session,
                        client,
                        channel: msg.channel,
                        payload: msg.message,
                    });
                }
                Err(err) => {
                    let _ = events.send(TransportEvent::Error {
                        session: Some(session),
                        client: Some(client),
                        error: TransportError::Serialization(err),
                    });
                }
            },
            Err(err) => {
                if !is_connection_closed(&err) {
                    let _ = events.send(TransportEvent::Error {
                        session: Some(session),
                        client: Some(client),
                        error: TransportError::Other(err.to_string()),
                    });
                }
                break;
            }
        }
    }
}

async fn read_uni_streams<S>(
    connection: Connection,
    serializer: Arc<S>,
    events: UnboundedSender<TransportEvent>,
    session: SessionId,
    client: ClientId,
) where
    S: MessageSerializer + Send + Sync + 'static + std::fmt::Debug,
{
    loop {
        match connection.accept_uni().await {
            Ok(mut recv) => {
                let mut len_buf = [0u8; 4];
                if let Err(err) = recv.read_exact(&mut len_buf).await {
                    let _ = events.send(TransportEvent::Error {
                        session: Some(session),
                        client: Some(client),
                        error: TransportError::Other(err.to_string()),
                    });
                    continue;
                }
                let len = u32::from_be_bytes(len_buf) as usize;
                if len > MAX_STREAM_MESSAGE_SIZE {
                    let _ = events.send(TransportEvent::Error {
                        session: Some(session),
                        client: Some(client),
                        error: TransportError::Other("stream message too large".into()),
                    });
                    continue;
                }
                let mut buf = vec![0u8; len];
                if let Err(err) = recv.read_exact(&mut buf).await {
                    let _ = events.send(TransportEvent::Error {
                        session: Some(session),
                        client: Some(client),
                        error: TransportError::Other(err.to_string()),
                    });
                    continue;
                }
                match serializer.deserialize::<OutgoingMessage>(&buf) {
                    Ok(msg) => {
                        let _ = events.send(TransportEvent::Message {
                            session,
                            client,
                            channel: msg.channel,
                            payload: msg.message,
                        });
                    }
                    Err(err) => {
                        let _ = events.send(TransportEvent::Error {
                            session: Some(session),
                            client: Some(client),
                            error: TransportError::Serialization(err),
                        });
                    }
                }
            }
            Err(err) => {
                if !is_connection_closed(&err) {
                    let _ = events.send(TransportEvent::Error {
                        session: Some(session),
                        client: Some(client),
                        error: TransportError::Other(err.to_string()),
                    });
                }
                break;
            }
        }
    }
}

fn build_server_config(
    config: &ServerNetworkingConfig,
) -> Result<quinn::ServerConfig, QuicServerError> {
    let (certs, private_key) = load_server_identity(&config.tls)?;
    let mut server_config = quinn::ServerConfig::with_single_cert(certs, private_key)
        .map_err(|err| QuicServerError::TlsConfig(err.to_string()))?;

    let mut transport = quinn::TransportConfig::default();
    transport
        .max_concurrent_bidi_streams(quinn::VarInt::from_u32(
            config.capabilities.max_ordered_streams as u32,
        ))
        .max_concurrent_uni_streams(quinn::VarInt::from_u32(
            config.capabilities.max_unordered_streams as u32,
        ))
        .datagram_receive_buffer_size(Some((config.transport.max_datagram_size as usize) * 64))
        .keep_alive_interval(Some(std::time::Duration::from_secs(
            config.transport.idle_timeout_secs / 2,
        )))
        .max_idle_timeout(Some(
            quinn::IdleTimeout::try_from(std::time::Duration::from_secs(
                config.transport.idle_timeout_secs,
            ))
            .map_err(|err| QuicServerError::TlsConfig(err.to_string()))?,
        ));
    server_config.transport = Arc::new(transport);
    Ok(server_config)
}

fn load_server_identity(
    tls: &ServerTlsConfig,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), QuicServerError> {
    match &tls.mode {
        ServerTlsMode::SelfSigned { subject } => {
            let certified_key = rcgen::generate_simple_self_signed([subject.clone()])
                .map_err(|err| QuicServerError::TlsConfig(err.to_string()))?;
            let rcgen::CertifiedKey { cert, signing_key } = certified_key;
            let cert_der = CertificateDer::from(cert);
            let priv_key = PrivatePkcs8KeyDer::from(signing_key.serialize_der());
            Ok((vec![cert_der], priv_key.into()))
        }
        ServerTlsMode::CertificateFiles {
            certificate,
            private_key,
        } => {
            let cert_file = File::open(certificate).map_err(|err| {
                QuicServerError::TlsConfig(format!(
                    "certificate file {}: {err}",
                    certificate.display()
                ))
            })?;
            let mut reader = BufReader::new(cert_file);
            let certs = pemfile::certs(&mut reader)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|err| QuicServerError::TlsConfig(format!("parse certificate: {err}")))?;
            if certs.is_empty() {
                return Err(QuicServerError::TlsConfig("no certificates found".into()));
            }

            let key_file = File::open(private_key).map_err(|err| {
                QuicServerError::TlsConfig(format!(
                    "private key file {}: {err}",
                    private_key.display()
                ))
            })?;
            let mut reader = BufReader::new(key_file);
            let key = pemfile::private_key(&mut reader)
                .map_err(|err| QuicServerError::TlsConfig(format!("parse private key: {err}")))?;
            let key = key
                .ok_or_else(|| QuicServerError::TlsConfig("no private key found in file".into()))?;

            Ok((certs, key))
        }
    }
}

fn disconnect_code(reason: DisconnectReason) -> VarInt {
    let code = match reason {
        DisconnectReason::Graceful => 0,
        DisconnectReason::Timeout => 1,
        DisconnectReason::Kicked => 2,
        DisconnectReason::AuthenticationFailed => 3,
        DisconnectReason::ProtocolMismatch => 4,
        DisconnectReason::TransportError => 5,
    };
    VarInt::from_u32(code)
}

fn map_close_reason(error: &ConnectionError) -> DisconnectReason {
    match error {
        ConnectionError::ApplicationClosed { .. } => DisconnectReason::Graceful,
        ConnectionError::LocallyClosed => DisconnectReason::Graceful,
        ConnectionError::TimedOut => DisconnectReason::Timeout,
        ConnectionError::ConnectionClosed(_) => DisconnectReason::TransportError,
        ConnectionError::TransportError(_) => DisconnectReason::TransportError,
        _ => DisconnectReason::TransportError,
    }
}

fn is_connection_closed(error: &ConnectionError) -> bool {
    matches!(
        error,
        ConnectionError::LocallyClosed
            | ConnectionError::ApplicationClosed { .. }
            | ConnectionError::TimedOut
            | ConnectionError::ConnectionClosed(_)
    )
}
