//! TaskManager — central background task orchestration for Wizard.
//!
//! Goals
//! - Single async loop that receives task commands over an MPSC channel.
//! - Spawn/cancel background jobs and report lifecycle via `Action::Logic(..)` back to the UI.
//! - Domain-specific helpers (config load/save, network checks) with a consistent API.
//! - Clear handoff point to add structured log piping into a future log view.
//!
//! Integration strategy
//! - App creates a `TaskManagerHandle` and keeps the cloneable sender.
//! - The manager sends `TaskStarted`, `TaskProgress`, `TaskCompleted` actions to the existing app channel.
//! - Cancellation is cooperative via `JoinHandle::abort()`; the manager also emits a `TaskCompleted` with `success=false` on cancel.
//!
//! NOTE: This module is currently standalone; wire it in `app.rs` by creating the handle and
//! replacing the inlined simulated executor in `handle_logic_action` with commands to this manager.

use std::collections::HashMap;

use tokio::{sync::mpsc, task::JoinHandle};
use tracing::{debug, error, info, warn};

use crate::action::{Action, LogicAction, TaskId, TaskKind, TaskProgress, TaskResult, TaskSpec};

/// Commands sent to the `TaskManager` control loop.
#[derive(Debug)]
pub enum TaskCommand {
    /// Spawn a simulated task that emits progress steps; useful for UX testing.
    SpawnSimulated {
        id: TaskId,
        spec: TaskSpec,
        /// Number of progress steps to simulate (default: 10).
        steps: u32,
        /// Delay between steps in milliseconds (default: 150ms).
        step_delay_ms: u64,
    },

    /// Spawn a "Load Config" domain task (currently a stub with progress).
    SpawnLoadConfig { id: TaskId },

    /// Spawn a "Save Config" domain task (currently a stub with progress).
    SpawnSaveConfig { id: TaskId },

    /// Spawn a simple network connectivity check to a given host:port.
    SpawnNetworkCheck {
        id: TaskId,
        address: String,
        /// Timeout in milliseconds for the connect attempt.
        timeout_ms: u64,
    },

    /// Attempt to cancel a task by its id.
    Cancel { id: TaskId },

    /// Ask the manager to stop and abort all running tasks (best-effort).
    Shutdown,
}

/// Cloneable handle to send commands to the `TaskManager`.
#[derive(Clone)]
pub struct TaskManagerHandle {
    tx: mpsc::UnboundedSender<TaskCommand>,
}

impl TaskManagerHandle {
    pub fn spawn_simulated(
        &self,
        id: TaskId,
        label: impl Into<String>,
        steps: u32,
        step_delay_ms: u64,
        payload_json: Option<String>,
    ) -> Result<(), mpsc::error::SendError<TaskCommand>> {
        let spec = TaskSpec {
            kind: TaskKind::Other,
            label: label.into(),
            payload_json,
        };
        self.tx.send(TaskCommand::SpawnSimulated {
            id,
            spec,
            steps,
            step_delay_ms,
        })
    }

    pub fn spawn_load_config(&self, id: TaskId) -> Result<(), mpsc::error::SendError<TaskCommand>> {
        self.tx.send(TaskCommand::SpawnLoadConfig { id })
    }

    pub fn spawn_save_config(&self, id: TaskId) -> Result<(), mpsc::error::SendError<TaskCommand>> {
        self.tx.send(TaskCommand::SpawnSaveConfig { id })
    }

    pub fn spawn_network_check(
        &self,
        id: TaskId,
        address: impl Into<String>,
        timeout_ms: u64,
    ) -> Result<(), mpsc::error::SendError<TaskCommand>> {
        self.tx.send(TaskCommand::SpawnNetworkCheck {
            id,
            address: address.into(),
            timeout_ms,
        })
    }

    pub fn cancel(&self, id: TaskId) -> Result<(), mpsc::error::SendError<TaskCommand>> {
        self.tx.send(TaskCommand::Cancel { id })
    }

    pub fn shutdown(&self) -> Result<(), mpsc::error::SendError<TaskCommand>> {
        self.tx.send(TaskCommand::Shutdown)
    }
}

/// Internal handle metadata for active tasks.
struct ActiveTask {
    handle: JoinHandle<()>,
    _kind: TaskKind,
    _label: String,
}

/// TaskManager state and control loop.
pub struct TaskManager {
    action_tx: mpsc::UnboundedSender<Action>,
    cmd_rx: mpsc::UnboundedReceiver<TaskCommand>,
    active: HashMap<TaskId, ActiveTask>,
}

impl TaskManager {
    /// Create and spawn the manager loop, returning a handle for issuing commands and
    /// a `JoinHandle` for the manager itself (optional to await at shutdown).
    pub fn new(action_tx: mpsc::UnboundedSender<Action>) -> (TaskManagerHandle, JoinHandle<()>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut mgr = TaskManager {
            action_tx,
            cmd_rx: rx,
            active: HashMap::new(),
        };
        let join = tokio::spawn(async move { mgr.run().await });
        (TaskManagerHandle { tx }, join)
    }

    async fn run(&mut self) {
        info!("TaskManager loop started");
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                TaskCommand::SpawnSimulated {
                    id,
                    spec,
                    steps,
                    step_delay_ms,
                } => {
                    self.spawn_simulated(id, spec, steps, step_delay_ms);
                }
                TaskCommand::SpawnLoadConfig { id } => {
                    self.spawn_load_config(id);
                }
                TaskCommand::SpawnSaveConfig { id } => {
                    self.spawn_save_config(id);
                }
                TaskCommand::SpawnNetworkCheck {
                    id,
                    address,
                    timeout_ms,
                } => {
                    self.spawn_network_check(id, address, timeout_ms);
                }
                TaskCommand::Cancel { id } => {
                    self.cancel(&id);
                }
                TaskCommand::Shutdown => {
                    warn!(
                        "TaskManager shutdown requested; aborting {} active task(s)",
                        self.active.len()
                    );
                    // Best effort: abort all tasks
                    let ids: Vec<_> = self.active.keys().cloned().collect();
                    for id in ids {
                        self.cancel(&id);
                    }
                    break;
                }
            }
        }

        info!("TaskManager loop terminating");
    }

    fn spawn_simulated(&mut self, id: TaskId, spec: TaskSpec, steps: u32, step_delay_ms: u64) {
        debug!("Spawn simulated task: {} ({})", id, spec.label);
        let _ = self
            .action_tx
            .send(Action::Logic(LogicAction::TaskStarted { id: id.clone() }));

        let tx = self.action_tx.clone();
        let label = spec.label.clone();
        let payload = spec.payload_json.clone();

        let handle_id = id.clone();
        let handle = tokio::spawn({
            let id = handle_id;
            async move {
                let steps = steps.max(1);
                let delay = std::time::Duration::from_millis(step_delay_ms.max(1));
                for i in 1..=steps {
                    tokio::time::sleep(delay).await;
                    let _ = tx.send(Action::Logic(LogicAction::TaskProgress(TaskProgress {
                        id: id.clone(),
                        fraction: Some(i as f32 / steps as f32),
                        message: Some(format!("{label} — step {i}/{steps}")),
                    })));
                }

                let _ = tx.send(Action::Logic(LogicAction::TaskCompleted(TaskResult {
                    id: id.clone(),
                    success: true,
                    result_json: payload,
                    message: Some(format!("{label} — done")),
                })));
            }
        });

        self.active.insert(
            id.clone(),
            ActiveTask {
                handle,
                _kind: spec.kind,
                _label: spec.label,
            },
        );
    }

    fn spawn_load_config(&mut self, id: TaskId) {
        debug!("Spawn load-config task: {}", id);
        let spec = TaskSpec {
            kind: TaskKind::Io,
            label: "Load config".to_string(),
            payload_json: None,
        };
        let _ = self
            .action_tx
            .send(Action::Logic(LogicAction::TaskStarted { id: id.clone() }));

        let tx = self.action_tx.clone();
        let handle_id = id.clone();
        let handle = tokio::spawn({
            let id = handle_id;
            async move {
                // TODO: Replace with real load logic (settings/aether_config)
                for i in 1..=3 {
                    tokio::time::sleep(std::time::Duration::from_millis(120)).await;
                    let _ = tx.send(Action::Logic(LogicAction::TaskProgress(TaskProgress {
                        id: id.clone(),
                        fraction: Some(i as f32 / 3.0),
                        message: Some("Reading configuration...".to_string()),
                    })));
                }
                let _ = tx.send(Action::Logic(LogicAction::TaskCompleted(TaskResult {
                    id: id.clone(),
                    success: true,
                    result_json: None,
                    message: Some("Loaded configuration (stub)".to_string()),
                })));
            }
        });

        self.active.insert(
            id.clone(),
            ActiveTask {
                handle,
                _kind: spec.kind,
                _label: spec.label,
            },
        );
    }

    fn spawn_save_config(&mut self, id: TaskId) {
        debug!("Spawn save-config task: {}", id);
        let spec = TaskSpec {
            kind: TaskKind::Io,
            label: "Save config".to_string(),
            payload_json: None,
        };
        let _ = self
            .action_tx
            .send(Action::Logic(LogicAction::TaskStarted { id: id.clone() }));

        let tx = self.action_tx.clone();
        let handle_id = id.clone();
        let handle = tokio::spawn({
            let id = handle_id;
            async move {
                // TODO: Replace with real save logic (settings/aether_config)
                for i in 1..=3 {
                    tokio::time::sleep(std::time::Duration::from_millis(120)).await;
                    let _ = tx.send(Action::Logic(LogicAction::TaskProgress(TaskProgress {
                        id: id.clone(),
                        fraction: Some(i as f32 / 3.0),
                        message: Some("Writing configuration...".to_string()),
                    })));
                }
                let _ = tx.send(Action::Logic(LogicAction::TaskCompleted(TaskResult {
                    id: id.clone(),
                    success: true,
                    result_json: None,
                    message: Some("Saved configuration (stub)".to_string()),
                })));
            }
        });

        self.active.insert(
            id.clone(),
            ActiveTask {
                handle,
                _kind: spec.kind,
                _label: spec.label,
            },
        );
    }

    fn spawn_network_check(&mut self, id: TaskId, address: String, timeout_ms: u64) {
        debug!("Spawn network-check task: {} to {}", id, address);
        let spec = TaskSpec {
            kind: TaskKind::Network,
            label: format!("Net check {}", address),
            payload_json: None,
        };
        let _ = self
            .action_tx
            .send(Action::Logic(LogicAction::TaskStarted { id: id.clone() }));

        let tx = self.action_tx.clone();
        let handle_id = id.clone();
        let handle = tokio::spawn({
            let id = handle_id;
            async move {
                let timeout = std::time::Duration::from_millis(timeout_ms.max(1));
                let msg_try = format!("Connecting to {}", address);
                let _ = tx.send(Action::Logic(LogicAction::TaskProgress(TaskProgress {
                    id: id.clone(),
                    fraction: None,
                    message: Some(msg_try.clone()),
                })));

                let result = tokio::time::timeout(timeout, async {
                    use tokio::net::TcpStream;
                    TcpStream::connect(&address).await
                })
                .await;

                match result {
                    Ok(Ok(_stream)) => {
                        let _ = tx.send(Action::Logic(LogicAction::TaskProgress(TaskProgress {
                            id: id.clone(),
                            fraction: Some(1.0),
                            message: Some("Connected".to_string()),
                        })));
                        let _ = tx.send(Action::Logic(LogicAction::TaskCompleted(TaskResult {
                            id: id.clone(),
                            success: true,
                            result_json: None,
                            message: Some(format!("Network OK: {}", address)),
                        })));
                    }
                    Ok(Err(e)) => {
                        error!("Network check error: {}", e);
                        let _ = tx.send(Action::Logic(LogicAction::TaskCompleted(TaskResult {
                            id: id.clone(),
                            success: false,
                            result_json: None,
                            message: Some(format!("Network error: {}", e)),
                        })));
                    }
                    Err(_) => {
                        warn!("Network check timeout");
                        let _ = tx.send(Action::Logic(LogicAction::TaskCompleted(TaskResult {
                            id: id.clone(),
                            success: false,
                            result_json: None,
                            message: Some(format!("Timeout after {}ms", timeout_ms)),
                        })));
                    }
                }
            }
        });

        self.active.insert(
            id.clone(),
            ActiveTask {
                handle,
                _kind: spec.kind,
                _label: spec.label,
            },
        );
    }

    fn cancel(&mut self, id: &TaskId) {
        if let Some(active) = self.active.remove(id) {
            warn!("Cancelling task {}", id);
            active.handle.abort();
            // Emit a completion event so UI can update immediately.
            let _ = self
                .action_tx
                .send(Action::Logic(LogicAction::TaskCompleted(TaskResult {
                    id: id.clone(),
                    success: false,
                    result_json: None,
                    message: Some("Cancelled".to_string()),
                })));
        } else {
            debug!("Cancel requested for unknown task {}", id);
        }
    }
}

/// Structured log event (reserved for future use).
/// This will be wired to a dedicated log view. For now, we piggyback on TaskProgress
/// `message` updates and completion summaries.
///
/// Fields are chosen to be friendly to on-screen filtering and future persistence.
#[derive(Debug, Clone)]
pub struct TaskLog {
    pub id: TaskId,
    pub level: LogLevel,
    pub message: String,
    pub ts_unix_ms: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
