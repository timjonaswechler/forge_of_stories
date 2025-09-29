//! Verwaltung des clientseitigen Tokio-Runtimes.

use std::{future::Future, sync::Arc};

use thiserror::Error;
use tokio::runtime::{Builder, Handle, Runtime};

/// Gemeinsamer Zugriffspunkt auf den Client-Netzwerkruntime.
#[derive(Debug, Clone)]
pub struct ClientNetworkRuntime {
    runtime: Arc<Runtime>,
}

impl ClientNetworkRuntime {
    /// Baut einen Multi-Thread-Runtime mit zwei Worker-Threads für clientseitige Aufgaben.
    pub fn multi_thread() -> Result<Self, RuntimeError> {
        let runtime = Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(RuntimeError::Build)?;
        Ok(Self {
            runtime: Arc::new(runtime),
        })
    }

    /// Spawnt ein Future auf dem Runtime.
    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }

    /// Liefert einen Klon des Handles.
    pub fn handle(&self) -> Handle {
        self.runtime.handle().clone()
    }
}

/// Fehler, die beim Aufbau des Runtimes auftreten können.
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("failed to build tokio runtime: {0}")]
    Build(std::io::Error),
}
