use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};

/// Server runtime that wires:
/// - Control-plane (UDS + axum)
/// - Data-plane (QUIC endpoint with self-signed TLS)
pub struct ServerRuntime {
    // Control-plane (UDS)
    socket_path: PathBuf,
    #[cfg(unix)]
    uds: Option<crate::extensions::uds::axum::UdsAxumHandle>,

    // Data-plane (QUIC)
    bind_addr: SocketAddr,
    quic_endpoint: Option<quinn::Endpoint>,
    accept_task: Option<tokio::task::JoinHandle<()>>,
}

impl ServerRuntime {
    /// Build the runtime with a provided socket path (or sensible default) and bind address
    /// from env or defaults.
    ///
    /// Env overrides:
    /// - FOS_AETHER_UDS: path to the UDS socket
    /// - FOS_AETHER_BIND: "IP:PORT" (e.g. "0.0.0.0:27015")
    pub fn new(socket_path: Option<PathBuf>) -> Self {
        let socket_path = socket_path.unwrap_or_else(default_socket_path);
        let bind_addr = default_bind_addr();

        Self {
            socket_path,
            #[cfg(unix)]
            uds: None,
            bind_addr,
            quic_endpoint: None,
            accept_task: None,
        }
    }

    /// Start subsystems:
    /// - Control-plane (UDS + axum)
    /// - Data-plane (QUIC + self-signed TLS + accept loop)
    #[cfg(unix)]
    pub async fn start(&mut self) -> Result<()> {
        use crate::extensions::uds::axum::start_uds_axum;

        // Control-plane (UDS)
        let handle = start_uds_axum(&self.socket_path)
            .await
            .with_context(|| format!("starting UDS control-plane at {:?}", self.socket_path))?;
        self.uds = Some(handle);

        // Data-plane (QUIC)
        self.start_quic().await?;

        Ok(())
    }

    /// No-op UDS on non-unix; still start QUIC.
    #[cfg(not(unix))]
    pub async fn start(&mut self) -> Result<()> {
        self.start_quic().await
    }

    async fn start_quic(&mut self) -> Result<()> {
        let server_config = build_quic_server_config().context("build QUIC server config")?;
        let endpoint =
            quinn::Endpoint::server(server_config, self.bind_addr).context("bind QUIC endpoint")?;

        println!("QUIC listening on {}", self.bind_addr);

        // Spawn accept loop
        let task = tokio::spawn(async move {
            loop {
                match endpoint.accept().await {
                    Some(connecting) => {
                        tokio::spawn(async move {
                            match connecting.await {
                                Ok(conn) => {
                                    let peer = conn.remote_address();
                                    println!("QUIC connection from {}", peer);
                                    if let Err(e) = handle_connection(conn).await {
                                        eprintln!("connection error from {}: {e:?}", peer);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("incoming connection failed: {e:?}");
                                }
                            }
                        });
                    }
                    None => break,
                }
            }
        });

        self.accept_task = Some(task);
        // endpoint is owned by the accept task and will be dropped on shutdown

        Ok(())
    }

    /// Run the runtime until a shutdown signal is received (Ctrl+C).
    ///
    /// This awaits Ctrl+C and then performs a graceful shutdown of the control-plane and QUIC.
    pub async fn run_until_shutdown(mut self) -> Result<()> {
        println!(
            "Control-plane (UDS) {}",
            if cfg!(unix) {
                format!("listening at {:?}", self.socket_path)
            } else {
                "not available on this platform".to_string()
            }
        );
        println!("Data-plane (QUIC) listening on {}", self.bind_addr);
        println!("Press Ctrl+C to stop.");

        tokio::signal::ctrl_c()
            .await
            .context("waiting for Ctrl+C failed")?;

        self.shutdown().await?;
        Ok(())
    }

    /// Gracefully stop subsystems and cleanup resources.
    pub async fn shutdown(&mut self) -> Result<()> {
        // Stop QUIC accept loop and endpoint first.
        if let Some(task) = self.accept_task.take() {
            task.abort();
            let _ = task.await;
        }
        if let Some(endpoint) = self.quic_endpoint.take() {
            // Best-effort wait for idle connections; not required at this stage.
            let _ = endpoint.wait_idle().await;
            drop(endpoint);
        }

        // Stop control-plane (UDS)
        #[cfg(unix)]
        {
            if let Some(handle) = self.uds.take() {
                handle
                    .shutdown()
                    .await
                    .with_context(|| format!("shutting down UDS at {:?}", self.socket_path))?;
            }
        }
        Ok(())
    }

    /// Get the configured socket path.
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

fn default_socket_path() -> PathBuf {
    // Honor explicit override first.
    if let Ok(p) = std::env::var("FOS_AETHER_UDS") {
        return PathBuf::from(p);
    }

    // Follow XDG_RUNTIME_DIR if present (common on Linux).
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(dir).join("aether.sock");
    }

    // Fallback to system temp dir (works on macOS/Linux).
    // std::env::temp_dir().join("aether.sock")
    PathBuf::from("/tmp/aether.sock")
}

fn default_bind_addr() -> SocketAddr {
    if let Ok(s) = std::env::var("FOS_AETHER_BIND") {
        if let Ok(addr) = s.parse::<SocketAddr>() {
            return addr;
        } else {
            eprintln!("Invalid FOS_AETHER_BIND '{}', falling back to default", s);
        }
    }
    // Default: 0.0.0.0:27015
    SocketAddr::from(([0, 0, 0, 0], 27015))
}

fn build_quic_server_config() -> Result<quinn::ServerConfig> {
    // Self-signed certificate for development
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()])
        .context("rcgen self-signed")?;
    let cert_der = cert.cert.der().to_vec();
    let key_der = cert.signing_key.serialize_der();

    let key = rustls::pki_types::PrivateKeyDer::from(rustls::pki_types::PrivatePkcs8KeyDer::from(
        key_der,
    ));
    let cert_chain = vec![rustls::pki_types::CertificateDer::from(cert_der)];

    let server_config =
        quinn::ServerConfig::with_single_cert(cert_chain, key).context("quinn with_single_cert")?;
    Ok(server_config)
}

async fn handle_connection(conn: quinn::Connection) -> Result<()> {
    use network_shared::messaging::stream::{QuinnRecvBincodeExt, QuinnSendBincodeExt};
    use network_shared::protocol::{ClientHello, PROTOCOL_VERSION, ServerHello};

    // Expect the client to open a bidi stream for handshake.
    let (mut send, mut recv) = conn.accept_bi().await.context("accept_bi")?;

    // Receive version hello
    let hello: ClientHello = recv.recv_bincode().await.context("recv ClientHello")?;
    match hello {
        ClientHello::Version { version } if version == PROTOCOL_VERSION => {
            let resp = ServerHello::VersionAccepted {
                version: PROTOCOL_VERSION,
            };
            send.send_bincode(&resp).await.context("send ServerHello")?;
        }
        ClientHello::Version { .. } => {
            let resp = ServerHello::VersionMismatch {
                server_version: PROTOCOL_VERSION,
            };
            send.send_bincode(&resp)
                .await
                .context("send VersionMismatch")?;
        }
    }

    Ok(())
}
