use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};

#[cfg(unix)]
use {
    std::fs,
    std::os::unix::fs::PermissionsExt,
    tokio::{net::UnixListener, sync::oneshot, task::JoinHandle},
};

/// Build the control-plane router.
///
/// Routes:
/// - GET /v1/health -> 200 OK "ok"
fn build_router() -> Router {
    Router::new().route("/v1/health", get(health))
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

#[cfg(unix)]
pub struct UdsAxumHandle {
    join: JoinHandle<()>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    socket_path: PathBuf,
}

#[cfg(unix)]
impl UdsAxumHandle {
    /// Gracefully stop the UDS axum server and remove the socket file.
    pub async fn shutdown(mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            // Best-effort signal; ignore if receiver already dropped.
            let _ = tx.send(());
        }
        // Wait for the server task to exit.
        let _ = self.join.await;

        // Best-effort cleanup of the socket path.
        let _ = std::fs::remove_file(&self.socket_path);

        Ok(())
    }

    /// Returns the socket path this server is bound to.
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

/// Start an axum server bound to a Unix domain socket at the given `socket_path`.
///
/// This sets the UDS file mode to 0o660 (rw for owner and group).
#[cfg(unix)]
pub async fn start_uds_axum<P: AsRef<Path>>(socket_path: P) -> Result<UdsAxumHandle> {
    let socket_path = socket_path.as_ref().to_path_buf();

    // Ensure parent directory exists.
    if let Some(parent) = socket_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("creating parent directories for {:?}", socket_path))?;
    }

    // Remove stale socket file if present.
    if socket_path.exists() {
        fs::remove_file(&socket_path)
            .with_context(|| format!("removing stale socket at {:?}", socket_path))?;
    }

    // Bind UDS.
    let uds = UnixListener::bind(&socket_path)
        .with_context(|| format!("binding unix socket at {:?}", socket_path))?;

    // Set permissions to 0660.
    let mut perms = fs::metadata(&socket_path)
        .with_context(|| format!("reading metadata for {:?}", socket_path))?
        .permissions();
    perms.set_mode(0o660);
    fs::set_permissions(&socket_path, perms)
        .with_context(|| format!("setting permissions 0o660 for {:?}", socket_path))?;

    let app = build_router();

    // Shutdown coordination.
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // Spawn server in background.
    let join = tokio::spawn(async move {
        // axum::serve is generic over listeners (including UnixListener on unix).
        // We attach a graceful shutdown signal that resolves when `shutdown_rx` fires.
        let server = axum::serve(uds, app).with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
        });

        if let Err(err) = server.await {
            // Log the error to stderr; in a real logger setup, use tracing/log.
            eprintln!("UDS axum server error: {err:?}");
        }
    });

    Ok(UdsAxumHandle {
        join,
        shutdown_tx: Some(shutdown_tx),
        socket_path,
    })
}

#[cfg(not(unix))]
compile_error!(
    "UDS control-plane is only supported on unix platforms. Build on a unix target to enable this module."
);
