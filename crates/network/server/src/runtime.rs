//! Verwaltung des serverseitigen Tokio-Runtimes und zugehöriger Tasks.

use std::{future::Future, sync::Arc};

use thiserror::Error;
use tokio::runtime::{Builder, Handle, Runtime};

/// Shared handle to the networking runtime.
#[derive(Debug, Clone)]
pub struct NetworkRuntime {
    runtime: Arc<Runtime>,
}

impl NetworkRuntime {
    /// Builds a multi-thread runtime (single worker) intended for server transports.
    pub fn current_thread() -> Result<Self, RuntimeError> {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .map_err(RuntimeError::Build)?;
        Ok(Self {
            runtime: Arc::new(runtime),
        })
    }

    /// Spawns a future onto the networking runtime.
    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }

    /// Returns a clone of the internal [`Handle`].
    pub fn handle(&self) -> Handle {
        self.runtime.handle().clone()
    }
}

/// Errors that can be raised when constructing or interacting with the runtime.
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("failed to build tokio runtime: {0}")]
    Build(std::io::Error),
}
