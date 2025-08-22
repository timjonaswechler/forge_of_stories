use crate::messages::{AetherStatsSnapshot, AetherToWizard, WizardToAether};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::task::JoinHandle;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

/// Minimal settings snapshot required to start Aether.
/// Extend this as your game/server needs evolve (ports, storage paths, tickrate, etc.).
#[derive(Clone, Debug)]
pub struct AetherSettingsSnapshot {
    pub tick_hz: u32,
    pub quic_bind_addr: String,
}

impl Default for AetherSettingsSnapshot {
    fn default() -> Self {
        Self {
            tick_hz: 60,
            quic_bind_addr: "0.0.0.0:7777".into(),
        }
    }
}

/// Runtime state of the supervised Aether server.
pub enum AetherState {
    Stopped,
    Starting {
        handle: JoinHandle<()>,
        cancel: CancellationToken,
        control_tx: UnboundedSender<WizardToAether>,
        since: Instant,
        settings: AetherSettingsSnapshot,
    },
    Running {
        handle: JoinHandle<()>,
        cancel: CancellationToken,
        control_tx: UnboundedSender<WizardToAether>,
        since: Instant,
        settings: AetherSettingsSnapshot,
    },
    Stopping,
}

impl AetherState {
    pub fn is_running(&self) -> bool {
        matches!(self, AetherState::Running { .. })
    }

    pub fn can_start(&self) -> bool {
        matches!(self, AetherState::Stopped)
    }
}

/// Supervisor that owns and controls Aether's lifecycle.
/// It exposes:
/// - event_rx: stream of AetherToWizard events (Started, Stopped, Stats, Error)
/// - start/stop/restart methods
/// - send_control for runtime tuning commands
pub struct AetherSupervisor {
    state: AetherState,
    event_tx: UnboundedSender<AetherToWizard>,
    event_rx: Option<UnboundedReceiver<AetherToWizard>>,
}

impl Default for AetherSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl AetherSupervisor {
    /// Create a new Supervisor in Stopped state.
    pub fn new() -> Self {
        let (event_tx, event_rx) = unbounded_channel();
        Self {
            state: AetherState::Stopped,
            event_tx,
            event_rx: Some(event_rx),
        }
    }

    /// Returns whether the server is currently considered running (or starting).
    pub fn is_running(&self) -> bool {
        self.state.is_running()
    }

    /// Returns whether we can start a new server process (i.e., currently stopped).
    pub fn can_start(&self) -> bool {
        self.state.can_start()
    }

    /// Take ownership of the event receiver for server events (Started/Stopped/Stats/Error).
    /// Can only be taken once; subsequent calls return None until you recreate the supervisor.
    pub fn take_event_receiver(&mut self) -> Option<UnboundedReceiver<AetherToWizard>> {
        self.event_rx.take()
    }

    /// Start the server with the provided settings, if possible.
    /// No-op if not in Stopped state.
    pub fn start(&mut self, settings: AetherSettingsSnapshot) {
        if !self.can_start() {
            warn!("AetherSupervisor.start() ignored: not in Stopped state");
            return;
        }

        let cancel = CancellationToken::new();
        let (control_tx, control_rx) = unbounded_channel::<WizardToAether>();
        let event_tx = self.event_tx.clone();
        let since = Instant::now();
        let settings_clone = settings.clone();
        let cancel_child = cancel.clone();

        let handle = tokio::spawn(async move {
            if let Err(e) =
                spawn_aether_task(settings_clone, control_rx, event_tx.clone(), cancel_child).await
            {
                let _ = event_tx.send(AetherToWizard::Error(format!("aether task error: {e}")));
            }
        });

        // -> Hier Starting statt Running
        self.state = AetherState::Starting {
            handle,
            cancel,
            control_tx,
            since,
            settings,
        };
    }

    pub fn stop(&mut self) {
        let prev = std::mem::replace(&mut self.state, AetherState::Stopping);
        let event_tx = self.event_tx.clone();
        if let AetherState::Running {
            handle,
            cancel,
            control_tx,
            ..
        } = prev
        {
            let _ = control_tx.send(WizardToAether::Shutdown);
            cancel.cancel();

            // Join im Hintergrund, UI blockiert nicht

            tokio::spawn(async move {
                if let Err(join_err) = handle.await {
                    error!("Aether task join error: {join_err}");
                    let _ = event_tx.send(AetherToWizard::ServerStopped);
                }
                // Das Aether-Task sendet normalerweise ServerStopped selbst beim Exit
            });

            self.state = AetherState::Stopped;
        } else {
            self.state = AetherState::Stopped;
        }
    }

    pub fn restart(&mut self, new_settings: AetherSettingsSnapshot) {
        self.stop();
        self.start(new_settings);
    }

    pub fn mark_started(&mut self) {
        let prev = std::mem::replace(&mut self.state, AetherState::Stopped);
        match prev {
            AetherState::Starting {
                handle,
                cancel,
                control_tx,
                since,
                settings,
            } => {
                self.state = AetherState::Running {
                    handle,
                    cancel,
                    control_tx,
                    since,
                    settings,
                };
            }
            other => {
                self.state = other;
            }
        }
    }

    /// Send a control message to the running server (e.g., runtime tuning).
    /// Returns Err if not running or if the channel is closed.
    pub fn send_control(&self, msg: WizardToAether) -> Result<(), SendError> {
        match &self.state {
            AetherState::Running { control_tx, .. } => control_tx
                .send(msg)
                .map_err(|e| SendError::ChannelClosed(e.to_string())),
            _ => Err(SendError::NotRunning),
        }
    }

    /// Access current settings if running.
    pub fn current_settings(&self) -> Option<&AetherSettingsSnapshot> {
        match &self.state {
            AetherState::Running { settings, .. } => Some(settings),
            _ => None,
        }
    }
}

/// Error type for control message sending.
#[derive(Debug)]
pub enum SendError {
    NotRunning,
    ChannelClosed(String),
}

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SendError::NotRunning => write!(f, "server is not running"),
            SendError::ChannelClosed(e) => write!(f, "control channel closed: {e}"),
        }
    }
}

impl std::error::Error for SendError {}

/// Placeholder async task that represents the running Aether server.
/// Replace this with the actual `aether` crate integration when ready.
///
/// Responsibilities:
/// - Notify Started
/// - Handle control_rx (Shutdown, ApplyRuntimeSetting, etc.)
/// - Emit periodic Stats
/// - Exit on cancellation or Shutdown, then notify Stopped
async fn spawn_aether_task(
    settings: AetherSettingsSnapshot,
    mut control_rx: UnboundedReceiver<WizardToAether>,
    event_tx: UnboundedSender<AetherToWizard>,
    cancel: CancellationToken,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "Aether starting on {} (tick {} Hz)",
        settings.quic_bind_addr, settings.tick_hz
    );
    let _ = event_tx.send(AetherToWizard::ServerStarted);

    let start = Instant::now();
    let mut stats_tick = interval(Duration::from_millis(1000));

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                debug!("Aether cancellation received");
                break;
            }
            Some(ctrl) = control_rx.recv() => {
                match ctrl {
                    WizardToAether::Shutdown => {
                        debug!("Aether graceful shutdown requested");
                        break;
                    }
                    WizardToAether::ApplyRuntimeSetting { key, value } => {
                        info!("Aether runtime setting: {key} = {value}");
                        // TODO: apply in ECS/resources once integrated
                    }
                    WizardToAether::StartServer | WizardToAether::StopServer => {
                        // These are high-level controls; supervisor should translate them.
                        debug!("Ignoring high-level control in child loop: {:?}", ctrl);
                    }
                }
            }
            _ = stats_tick.tick() => {
                let uptime = start.elapsed().as_secs();
                // TODO: players from ECS/network once integrated
                let _ = event_tx.send(AetherToWizard::Stats(AetherStatsSnapshot {
                    uptime_secs: uptime,
                    players: 0,
                }));
            }
        }
    }

    info!("Aether stopped");
    let _ = event_tx.send(AetherToWizard::ServerStopped);
    Ok(())
}
