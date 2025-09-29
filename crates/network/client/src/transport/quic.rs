//! QUIC-Transport-Backend für den Client.

use std::{fs::File, io::BufReader, net::SocketAddr, sync::Arc, time::Duration};

use bytes::Bytes;
use network_shared::{
    config::{ClientNetworkingConfig, ClientTlsTrust},
    events::{ClientEvent, DisconnectReason, TransportCapabilities, TransportError},
    messages::OutgoingMessage,
    serialization::{BincodeSerializer, MessageSerializer, SerializationError},
};
use quinn::crypto::rustls::QuicClientConfig;
use quinn::rustls::RootCertStore;
use quinn::rustls::SignatureScheme;
use quinn::rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified};
use quinn::rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use quinn::{self, Connection, ConnectionError, Endpoint, VarInt};
use rustls_native_certs as native_certs;
use rustls_pemfile as pemfile;
use tokio::sync::mpsc::UnboundedSender;

use super::{ClientTransport, ConnectTarget};
use crate::runtime::ClientNetworkRuntime;

const MAX_STREAM_MESSAGE_SIZE: usize = 128 * 1024;

/// QUIC-Clienttransport, der Verbindungen über Quinn aufbaut und Ereignisse in die generischen
/// `ClientEvent`s einspeist.
#[derive(Debug)]
pub struct QuicClientTransport<S = BincodeSerializer>
where
    S: MessageSerializer + std::fmt::Debug,
{
    runtime: ClientNetworkRuntime,
    config: ClientNetworkingConfig,
    serializer: Arc<S>,
    endpoint: Option<Endpoint>,
    connection: Option<Connection>,
}

impl<S> QuicClientTransport<S>
where
    S: MessageSerializer + std::fmt::Debug,
{
    pub fn new(
        runtime: ClientNetworkRuntime,
        config: ClientNetworkingConfig,
        serializer: S,
    ) -> Self {
        Self {
            runtime,
            config,
            serializer: Arc::new(serializer),
            endpoint: None,
            connection: None,
        }
    }

    fn capabilities(&self) -> TransportCapabilities {
        self.config.capabilities
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QuicClientError {
    #[error("quic client transport ist bereits verbunden")]
    AlreadyConnected,
    #[error("ziel wird vom quic transport nicht unterstützt")]
    UnsupportedTarget,
    #[error("adresse ungültig: {0}")]
    Address(std::io::Error),
    #[error("connect error: {0}")]
    Connect(#[from] quinn::ConnectError),
    #[error("connection error: {0}")]
    Connection(#[from] quinn::ConnectionError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] SerializationError),
    #[error("send datagram error: {0}")]
    SendDatagram(#[from] quinn::SendDatagramError),
    #[error("write error: {0}")]
    Write(#[from] quinn::WriteError),
    #[error("tls config error: {0}")]
    TlsConfig(String),
    #[error("keine aktive verbindung")]
    NotConnected,
}

impl<S> ClientTransport for QuicClientTransport<S>
where
    S: MessageSerializer + Send + Sync + 'static + std::fmt::Debug,
{
    type Error = QuicClientError;

    fn connect(
        &mut self,
        target: ConnectTarget,
        events: UnboundedSender<ClientEvent>,
    ) -> Result<(), Self::Error> {
        if self.connection.is_some() {
            return Err(QuicClientError::AlreadyConnected);
        }

        let (host, port) = match target {
            ConnectTarget::Quic { host, port } => (host, port),
            ConnectTarget::Steam { .. } => return Err(QuicClientError::UnsupportedTarget),
        };

        let addr: SocketAddr = format!("{host}:{port}").parse().map_err(|e| {
            QuicClientError::Address(std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
        })?;

        let client_config = build_client_config(&self.config)?;

        let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())?;
        endpoint.set_default_client_config(client_config);

        let connecting = endpoint.connect(addr, host.as_str())?;
        let runtime_handle = self.runtime.handle();
        let connection = runtime_handle.block_on(async { connecting.await })?;

        let events_clone = events.clone();
        let serializer = self.serializer.clone();
        let runtime = self.runtime.clone();
        runtime.spawn(read_datagrams(
            connection.clone(),
            serializer.clone(),
            events_clone,
        ));

        let events_clone = events.clone();
        runtime.spawn(read_uni_streams(
            connection.clone(),
            serializer.clone(),
            events_clone,
        ));

        let events_clone = events.clone();
        let connection_for_close = connection.clone();
        runtime.spawn(async move {
            let reason = connection_for_close.closed().await;
            let disconnect = map_close_reason(&reason);
            let _ = events_clone.send(ClientEvent::Disconnected { reason: disconnect });
        });

        let _ = events.send(ClientEvent::Connected);

        self.endpoint = Some(endpoint);
        self.connection = Some(connection);

        Ok(())
    }

    fn disconnect(&mut self, reason: DisconnectReason) -> Result<(), Self::Error> {
        if let Some(connection) = self.connection.take() {
            let code = disconnect_code(reason);
            connection.close(code, format!("{reason:?}").as_bytes());
        }
        if let Some(endpoint) = self.endpoint.take() {
            endpoint.close(0u32.into(), b"shutdown");
        }
        Ok(())
    }

    fn send(&self, message: OutgoingMessage) -> Result<(), Self::Error> {
        let connection = self
            .connection
            .as_ref()
            .ok_or(QuicClientError::NotConnected)?;
        let bytes = self.serializer.serialize(&message)?;
        let is_datagram = matches!(
            message.channel,
            network_shared::channels::ChannelKind::UnreliableSequenced
        );

        let runtime_handle = self.runtime.handle();
        let conn = connection.clone();
        runtime_handle.block_on(async move {
            if is_datagram {
                conn.send_datagram(Bytes::from(bytes))?;
            } else {
                let mut stream = conn.open_uni().await?;
                let len = bytes.len() as u32;
                stream.write_all(&len.to_be_bytes()).await?;
                stream.write_all(&bytes).await?;
                let _ = stream.finish();
            }
            Ok::<(), QuicClientError>(())
        })?;

        Ok(())
    }

    fn capabilities(&self) -> TransportCapabilities {
        self.capabilities()
    }
}

async fn read_datagrams<S>(
    connection: Connection,
    serializer: Arc<S>,
    events: UnboundedSender<ClientEvent>,
) where
    S: MessageSerializer + Send + Sync + 'static + std::fmt::Debug,
{
    loop {
        match connection.read_datagram().await {
            Ok(bytes) => match serializer.deserialize::<OutgoingMessage>(&bytes) {
                Ok(msg) => {
                    let _ = events.send(ClientEvent::Message {
                        channel: msg.channel,
                        payload: msg.message,
                    });
                }
                Err(err) => {
                    let _ = events.send(ClientEvent::Error {
                        error: TransportError::Serialization(err),
                    });
                }
            },
            Err(err) => {
                if !is_connection_closed(&err) {
                    let _ = events.send(ClientEvent::Error {
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
    events: UnboundedSender<ClientEvent>,
) where
    S: MessageSerializer + Send + Sync + 'static + std::fmt::Debug,
{
    loop {
        match connection.accept_uni().await {
            Ok(mut recv) => {
                let mut len_buf = [0u8; 4];
                if let Err(err) = recv.read_exact(&mut len_buf).await {
                    let _ = events.send(ClientEvent::Error {
                        error: TransportError::Other(err.to_string()),
                    });
                    continue;
                }
                let len = u32::from_be_bytes(len_buf) as usize;
                if len > MAX_STREAM_MESSAGE_SIZE {
                    let _ = events.send(ClientEvent::Error {
                        error: TransportError::Other("stream message too large".into()),
                    });
                    continue;
                }
                let mut buf = vec![0u8; len];
                if let Err(err) = recv.read_exact(&mut buf).await {
                    let _ = events.send(ClientEvent::Error {
                        error: TransportError::Other(err.to_string()),
                    });
                    continue;
                }
                match serializer.deserialize::<OutgoingMessage>(&buf) {
                    Ok(msg) => {
                        let _ = events.send(ClientEvent::Message {
                            channel: msg.channel,
                            payload: msg.message,
                        });
                    }
                    Err(err) => {
                        let _ = events.send(ClientEvent::Error {
                            error: TransportError::Serialization(err),
                        });
                    }
                }
            }
            Err(err) => {
                if !is_connection_closed(&err) {
                    let _ = events.send(ClientEvent::Error {
                        error: TransportError::Other(err.to_string()),
                    });
                }
                break;
            }
        }
    }
}

fn build_client_config(
    config: &ClientNetworkingConfig,
) -> Result<quinn::ClientConfig, QuicClientError> {
    let quic_crypto = match &config.tls.trust {
        ClientTlsTrust::InsecureSkipVerification => {
            let crypto = quinn::rustls::ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(SkipServerVerification))
                .with_no_client_auth();
            QuicClientConfig::try_from(crypto)
                .map_err(|e| QuicClientError::TlsConfig(e.to_string()))?
        }
        trust => {
            let root_store = build_root_store(trust)?;
            let crypto = quinn::rustls::ClientConfig::builder().with_root_certificates(root_store);
            let crypto = crypto.with_no_client_auth();
            QuicClientConfig::try_from(crypto)
                .map_err(|e| QuicClientError::TlsConfig(e.to_string()))?
        }
    };

    let mut client_config = quinn::ClientConfig::new(Arc::new(quic_crypto));
    client_config.transport_config(Arc::new(build_transport_config(config)));
    Ok(client_config)
}

fn build_root_store(trust: &ClientTlsTrust) -> Result<RootCertStore, QuicClientError> {
    let mut store = RootCertStore::empty();
    match trust {
        ClientTlsTrust::System => {
            let certs = native_certs::load_native_certs()
                .map_err(|err| QuicClientError::TlsConfig(format!("load native certs: {err}")))?;
            for cert in certs {
                store
                    .add(cert)
                    .map_err(|err| QuicClientError::TlsConfig(format!("add native cert: {err}")))?;
            }
        }
        ClientTlsTrust::CertificateFile { ca_certificate } => {
            let file = File::open(ca_certificate).map_err(|err| {
                QuicClientError::TlsConfig(format!(
                    "open ca certificate {}: {err}",
                    ca_certificate.display()
                ))
            })?;
            let mut reader = BufReader::new(file);
            let certs = pemfile::certs(&mut reader)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|err| QuicClientError::TlsConfig(format!("parse ca certificate: {err}")))?;
            if certs.is_empty() {
                return Err(QuicClientError::TlsConfig(
                    "no certificates found in trust store".into(),
                ));
            }
            for cert in certs {
                store
                    .add(cert)
                    .map_err(|err| QuicClientError::TlsConfig(format!("add ca cert: {err}")))?;
            }
        }
        ClientTlsTrust::InsecureSkipVerification => {
            return Err(QuicClientError::TlsConfig(
                "insecure skip verification should be handled separately".into(),
            ));
        }
    }
    Ok(store)
}

fn build_transport_config(config: &ClientNetworkingConfig) -> quinn::TransportConfig {
    let mut transport = quinn::TransportConfig::default();
    transport
        .max_concurrent_bidi_streams(quinn::VarInt::from_u32(
            config.capabilities.max_ordered_streams as u32,
        ))
        .max_concurrent_uni_streams(quinn::VarInt::from_u32(
            config.capabilities.max_unordered_streams as u32,
        ))
        .datagram_receive_buffer_size(Some((config.transport.max_datagram_size as usize) * 32))
        .keep_alive_interval(Some(Duration::from_secs(
            config.transport.idle_timeout_secs / 2,
        )));
    if let Ok(timeout) =
        quinn::IdleTimeout::try_from(Duration::from_secs(config.transport.idle_timeout_secs))
    {
        transport.max_idle_timeout(Some(timeout));
    }
    transport
}

#[derive(Debug)]
struct SkipServerVerification;

impl quinn::rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, quinn::rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _signature: &quinn::rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, quinn::rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _signature: &quinn::rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, quinn::rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ED25519,
            SignatureScheme::RSA_PSS_SHA256,
        ]
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
