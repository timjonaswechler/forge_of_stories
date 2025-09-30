//! Abstraktion für Steam-Integrationen (Steamworks/Aeronet).
//!
//! Externe Integrationen erhalten den `SteamBackendHandle`, um `SteamServerEvent`s
//! in den Discovery-Controller einzuspeisen. Dadurch bleiben die transportnahen
//! Komponenten unabhängig von konkreten SDK-Bindings.

use std::fmt;

use thiserror::Error;

use super::SteamBackendHandle;

/// Fehler, die beim Starten/Stoppen des Integrationslayers auftreten können.
#[derive(Debug, Error)]
pub enum SteamIntegrationError {
    #[error("failed to start steam integration: {0}")]
    Start(String),
    #[error("steam integration runtime error: {0}")]
    Runtime(String),
}

impl SteamIntegrationError {
    pub fn start<E: fmt::Display>(err: E) -> Self {
        Self::Start(err.to_string())
    }

    pub fn runtime<E: fmt::Display>(err: E) -> Self {
        Self::Runtime(err.to_string())
    }
}

/// Trait, das konkrete Steamworks/Aeronet-Adapter implementieren müssen.
pub trait SteamIntegration: Send {
    /// Initialisiert die Integration. Der übergebene Handle darf Eventqueues
    /// beschreiben und ist clonable.
    fn start(&mut self, handle: SteamBackendHandle) -> Result<(), SteamIntegrationError>;

    /// Stoppt laufende Tasks und räumt Ressourcen auf.
    fn stop(&mut self);
}
