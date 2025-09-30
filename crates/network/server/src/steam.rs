//! Steam-spezifische Discovery-Steuerung für den Server.
//!
//! Diese Datei definiert die Controller-Logik, die später durch eine
//! tatsächliche Steamworks-/Aeronet-Implementierung hinterlegt werden kann.

use std::fmt;

use network_shared::config::{ServerDeployment, SteamDiscoveryMode};
use thiserror::Error;
use tracing::{debug, warn};

/// Status des Steam-Discovery-Subsystems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteamDiscoveryStatus {
    Inactive,
    Active,
    BlockedDedicated,
}

impl SteamDiscoveryStatus {
    pub const fn is_active(self) -> bool {
        matches!(self, SteamDiscoveryStatus::Active)
    }
}

/// Fehler, die beim Starten/Stoppen der Steam-Discovery auftreten können.
#[derive(Debug, Error)]
pub enum SteamDiscoveryError {
    #[error("backend error: {0}")]
    Backend(String),
}

/// Abstraktes Backend für Steamworks/Aeronet.
pub trait SteamDiscoveryBackend: Send {
    fn activate(&mut self) -> Result<(), SteamDiscoveryError>;
    fn deactivate(&mut self);
}

/// Platzhalter-Backend ohne echte Steam-Integration.
#[derive(Debug, Default)]
pub struct NoopSteamDiscovery;

impl SteamDiscoveryBackend for NoopSteamDiscovery {
    fn activate(&mut self) -> Result<(), SteamDiscoveryError> {
        debug!(target = "network::discovery", "steam discovery noop activate");
        Ok(())
    }

    fn deactivate(&mut self) {
        debug!(target = "network::discovery", "steam discovery noop deactivate");
    }
}

/// Controller, der Deployment/Modus berücksichtigt und das Backend steuert.
pub struct SteamDiscoveryController {
    deployment: ServerDeployment,
    mode: SteamDiscoveryMode,
    status: SteamDiscoveryStatus,
    backend: Box<dyn SteamDiscoveryBackend>,
}

impl SteamDiscoveryController {
    pub fn new(deployment: ServerDeployment, mode: SteamDiscoveryMode) -> Self {
        let mut controller = Self {
            deployment,
            mode,
            status: SteamDiscoveryStatus::Inactive,
            backend: Box::<NoopSteamDiscovery>::default(),
        };
        controller.apply_mode();
        controller
    }

    pub fn set_mode(&mut self, deployment: ServerDeployment, mode: SteamDiscoveryMode) {
        self.deployment = deployment;
        self.mode = mode;
        self.apply_mode();
    }

    pub fn status(&self) -> SteamDiscoveryStatus {
        self.status
    }

    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    fn apply_mode(&mut self) {
        match (&self.mode, &self.deployment) {
            (SteamDiscoveryMode::Disabled, _) => {
                if self.status.is_active() {
                    self.backend.deactivate();
                }
                self.status = SteamDiscoveryStatus::Inactive;
                debug!(target = "network::discovery", "steam discovery disabled");
            }
            (SteamDiscoveryMode::LocalOnly, ServerDeployment::LocalHost) => {
                if let Err(err) = self.backend.activate() {
                    warn!(
                        target = "network::discovery",
                        "steam discovery activation failed: {err}"
                    );
                    self.status = SteamDiscoveryStatus::Inactive;
                } else {
                    self.status = SteamDiscoveryStatus::Active;
                    debug!(
                        target = "network::discovery",
                        "steam discovery active in local-host mode"
                    );
                }
            }
            (SteamDiscoveryMode::LocalOnly, ServerDeployment::Dedicated) => {
                if self.status.is_active() {
                    self.backend.deactivate();
                }
                self.status = SteamDiscoveryStatus::BlockedDedicated;
                warn!(
                    target = "network::discovery",
                    "steam discovery blocked: deployment is dedicated"
                );
            }
        }
    }
}

impl fmt::Debug for SteamDiscoveryController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SteamDiscoveryController")
            .field("deployment", &self.deployment)
            .field("mode", &self.mode)
            .field("status", &self.status)
            .finish()
    }
}
