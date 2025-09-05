pub mod extensions;
pub mod messaging;
pub mod runtime;
pub mod session;

use crate::runtime::ServerRuntime;
use anyhow::Result;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

/// Handle to the running server instance.
#[derive(Debug)]
pub struct ServerHandle {
    shutdown_tx: Option<oneshot::Sender<()>>,
    join: JoinHandle<Result<()>>,
}

impl ServerHandle {
    pub async fn shutdown(mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        // Wait for background task to complete and propagate any error.
        match self.join.await {
            Ok(res) => res,
            Err(join_err) => Err(join_err.into()),
        }
    }
}

/// Minimal runtime config for quick-start. The full settings-based config
/// will replace this once the runtime is wired.
#[derive(Debug, Clone)]
pub struct ServerConfigMinimal {
    pub bind_address: String,
    pub port: u16,
}

impl Default for ServerConfigMinimal {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".into(),
            port: 27015,
        }
    }
}

/// Start the Forge of Stories QUIC server (skeleton).
///
/// This is a temporary stub that returns a handle and prints a startup message.
/// The transport and control-plane are implemented in the `runtime` and `extensions` modules.
pub async fn start_server(config: Option<ServerConfigMinimal>) -> Result<ServerHandle> {
    let cfg = config.unwrap_or_default();
    println!(
        "🚀 Starting FOS Server on {}:{} ...",
        cfg.bind_address, cfg.port
    );

    // Spawn background runtime that manages the control-plane (UDS) and shuts down on signal.
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let join = tokio::spawn(async move {
        let mut rt = ServerRuntime::new(None);
        rt.start().await?;
        // Wait for shutdown signal; ignore if sender is dropped without sending.
        let _ = shutdown_rx.await;
        rt.shutdown().await?;
        Ok(())
    });

    Ok(ServerHandle {
        shutdown_tx: Some(shutdown_tx),
        join,
    })
}
