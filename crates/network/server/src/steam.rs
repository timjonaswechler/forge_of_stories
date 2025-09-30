//! Steam-spezifische Discovery-Steuerung für den Server.
//!
//! Diese Datei definiert die Controller-Logik, die später durch eine
//! tatsächliche Steamworks-/Aeronet-Implementierung hinterlegt werden kann.

use std::fmt;

use network_shared::{
    config::{ServerDeployment, SteamDiscoveryMode},
    discovery::{SteamLobbyId, SteamLobbyInfo, SteamRelayTicket, SteamServerEvent},
};
use thiserror::Error;
use tracing::{debug, warn};
use tokio::sync::mpsc::UnboundedSender;

mod channel_backend;
pub mod integration;

pub use channel_backend::{ChannelSteamDiscoveryBackend, SteamBackendHandle};
pub use integration::{SteamIntegration, SteamIntegrationError};

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
pub type SteamServerEventSender = UnboundedSender<SteamServerEvent>;

pub trait SteamDiscoveryBackend: Send {
    fn set_event_sink(&mut self, sender: SteamServerEventSender);
    fn activate(&mut self) -> Result<(), SteamDiscoveryError>;
    fn deactivate(&mut self);
}

/// Platzhalter-Backend ohne echte Steam-Integration.
#[derive(Debug, Default)]
pub struct NoopSteamDiscovery;

impl SteamDiscoveryBackend for NoopSteamDiscovery {
    fn set_event_sink(&mut self, _sender: SteamServerEventSender) {}

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
    event_tx: Option<SteamServerEventSender>,
}

impl SteamDiscoveryController {
    pub fn new(deployment: ServerDeployment, mode: SteamDiscoveryMode) -> Self {
        let mut controller = Self {
            deployment,
            mode,
            status: SteamDiscoveryStatus::Inactive,
            backend: Box::<NoopSteamDiscovery>::default(),
            event_tx: None,
        };
        controller.apply_mode();
        controller
    }

    pub fn set_event_sender(&mut self, sender: SteamServerEventSender) {
        self.event_tx = Some(sender.clone());
        self.backend.set_event_sink(sender);
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
                self.publish(SteamServerEvent::Deactivated);
            }
            (SteamDiscoveryMode::LocalOnly, ServerDeployment::LocalHost) => {
                if let Err(err) = self.backend.activate() {
                    warn!(
                        target = "network::discovery",
                        "steam discovery activation failed: {err}"
                    );
                    self.status = SteamDiscoveryStatus::Inactive;
                    self.publish(SteamServerEvent::Error {
                        message: format!("activation failed: {err}"),
                    });
                } else {
                    self.status = SteamDiscoveryStatus::Active;
                    debug!(
                        target = "network::discovery",
                        "steam discovery active in local-host mode"
                    );
                    self.publish(SteamServerEvent::Activated);
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
                self.publish(SteamServerEvent::Error {
                    message: "steam discovery blocked for dedicated deployment".into(),
                });
            }
        }
    }

    pub fn replace_backend<B>(&mut self, mut backend: B)
    where
        B: SteamDiscoveryBackend + 'static,
    {
        if self.status.is_active() {
            self.backend.deactivate();
        }
        if let Some(sender) = &self.event_tx {
            backend.set_event_sink(sender.clone());
        }
        self.backend = Box::new(backend);
        self.status = SteamDiscoveryStatus::Inactive;
        self.apply_mode();
    }

    pub fn publish_lobby_update(&self, info: SteamLobbyInfo) {
        self.publish(SteamServerEvent::LobbyUpdated(info));
    }

    pub fn publish_lobby_discovered(&self, info: SteamLobbyInfo) {
        self.publish(SteamServerEvent::LobbyDiscovered(info));
    }

    pub fn publish_lobby_removed(&self, lobby: SteamLobbyId) {
        self.publish(SteamServerEvent::LobbyRemoved(lobby));
    }

    pub fn publish_ticket(&self, ticket: SteamRelayTicket) {
        self.publish(SteamServerEvent::TicketIssued(ticket));
    }

    pub fn publish_ticket_revoked(&self, lobby: SteamLobbyId) {
        self.publish(SteamServerEvent::TicketRevoked(lobby));
    }

    fn publish(&self, event: SteamServerEvent) {
        if let Some(sender) = &self.event_tx {
            let _ = sender.send(event);
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
