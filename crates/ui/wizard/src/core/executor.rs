/*!
Task Executor Skeleton (Phase 10 – foundational)

This module introduces a first-cut asynchronous task executor to decouple
the reducer’s declarative `Effect::Async(TaskKind)` outputs from concrete
runtime side-effects (spawning background work, logging, producing follow-up
actions/events).

Current Scope (minimal viable):
- Accept scheduling requests (`TaskKind`) via an internal unbounded channel.
- Spawn a single background worker task (Tokio) that pulls and handles tasks.
- For each received task, log a stub message (e.g. certificate generation placeholder).
- Provide a `spawn(kind)` method for the event loop (or future dispatcher) to call.
- Keep API surface intentionally small to allow iterative evolution.

Out-of-Scope (future phases):
- Returning structured completion events (InternalEvent / Action variants).
- Cancellation, task status queries, progress streaming.
- Batching, prioritization, rate limiting, backoff.
- Telemetry / metrics (task durations, counts, failures).
- A pluggable policies layer (e.g., concurrency caps per task class).

Integration Model (roadmap):
1. Reducer emits `Effect::Async(TaskKind::Xyz)`.
2. Event loop forwards those TaskKinds into `TaskExecutor::spawn(...)`.
3. Executor performs (or simulates) the work.
4. Upon completion, executor will (future) emit a domain result via an
   action channel (e.g. `Action::TaskFinished(id, TaskResultKind)`), or
   an internal event channel after the Intent/Action split (Phase 11+).

Safety / Guarantees:
- All task handling is moved off the main UI thread (Tokio runtime required).
- The executor never panics intentionally; errors are logged as warnings.
- Pending tasks are dropped silently on shutdown (can be enhanced later).

Design Notes:
- A task ID is allocated (monotonic u64) per scheduled task to support
  future correlation (e.g. for completion callbacks).
- The current implementation does not store a registry/map; lightweight
  logging only. A `HashMap<TaskId, TaskMeta>` can be added easily when needed.
- `TaskExecutor` is `Clone` (cheap – clones the sender).
*/

use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::mpsc;

use crate::action::Action;
use crate::core::effects::{TaskKind, TaskResultKind};
use crate::domain::certs::{CertTaskResult, generate_self_signed_task};
use log::{info, warn};

/// Monotonic task identifier type.
pub type TaskId = u64;

/// Public handle for scheduling background tasks.
///
/// Cloneable & cheap: internally only wraps an `mpsc::UnboundedSender`.
#[derive(Clone)]
pub struct TaskExecutor {
    tx: mpsc::UnboundedSender<Dispatch>,
    action_tx: Option<mpsc::UnboundedSender<Action>>,
}

/// Internal dispatch envelope.
struct Dispatch {
    id: TaskId,
    kind: TaskKind,
}

impl TaskExecutor {
    /// Create a new executor and spawn its worker loop.
    ///
    /// `action_tx` is reserved for future integration (sending completion events).
    /// For now it is unused (to avoid premature introduction of new Action variants).
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<Dispatch>();
        let worker = Worker::new(rx, None);
        worker.spawn();
        Self {
            tx,
            action_tx: None,
        }
    }

    /// Create a new executor and spawn its worker loop with an action channel.
    ///
    /// The provided `action_tx` is used by the executor worker to emit
    /// Task lifecycle actions back into the main app loop:
    /// - TaskStarted(id, label)
    /// - TaskLog(id, msg)
    /// - TaskFinished(id, TaskResultKind)
    pub fn new_with_action_tx(action_tx: mpsc::UnboundedSender<Action>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<Dispatch>();
        let worker = Worker::new(rx, Some(action_tx.clone()));
        worker.spawn();
        Self {
            tx,
            action_tx: Some(action_tx),
        }
    }

    /// Schedule a new asynchronous task.
    ///
    /// Returns the allocated TaskId (can be used later for correlation).
    pub fn spawn(&self, kind: TaskKind) -> TaskId {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        if let Err(e) = self.tx.send(Dispatch { id, kind }) {
            warn!(
                "TaskExecutor channel closed; failed to schedule task: {}",
                e
            );
        }
        id
    }
}

/// Background worker driving task execution.
///
/// Future evolution:
/// - Maintain a registry of active tasks
/// - Support graceful shutdown / join
/// - Multi-worker pool (configurable parallelism)
struct Worker {
    rx: mpsc::UnboundedReceiver<Dispatch>,
    action_tx: Option<mpsc::UnboundedSender<Action>>,
}

impl Worker {
    fn new(
        rx: mpsc::UnboundedReceiver<Dispatch>,
        action_tx: Option<mpsc::UnboundedSender<Action>>,
    ) -> Self {
        Self { rx, action_tx }
    }

    fn emit(&self, action: Action) {
        if let Some(tx) = &self.action_tx {
            let _ = tx.send(action);
        }
    }

    fn spawn(mut self) {
        tokio::spawn(async move {
            while let Some(dispatch) = self.rx.recv().await {
                if let Err(e) = self.handle(dispatch).await {
                    warn!("Task execution failed: {e}");
                }
            }
            // Channel closed: executor is shutting down.
            info!("TaskExecutor worker stopped (channel closed)");
        });
    }

    async fn handle(&self, dispatch: Dispatch) -> Result<(), String> {
        // Emit TaskStarted for visibility in the app
        let label = format!("{}", dispatch.kind);
        self.emit(Action::TaskStarted(dispatch.id, label));
        match dispatch.kind {
            TaskKind::GenerateCert(params) => {
                self.emit(Action::TaskLog(
                    dispatch.id,
                    "Generating certificate".into(),
                ));
                match generate_self_signed_task(&params) {
                    CertTaskResult::Success { artifacts } => {
                        info!(
                            "[task:{}] cert generated CN={} dns_names={:?} days={} key_bits={} cert_len={} key_len={}",
                            dispatch.id,
                            params.common_name,
                            params.dns_names,
                            params.valid_days,
                            params.key_bits,
                            artifacts.cert_pem.len(),
                            artifacts.key_pem.len()
                        );
                        self.emit(Action::TaskFinished(
                            dispatch.id,
                            TaskResultKind::CertGenerated {
                                cn: params.common_name.clone(),
                                cert_pem: artifacts.cert_pem,
                                key_pem: artifacts.key_pem,
                                output_path: params.output_path.clone(),
                            },
                        ));
                        Ok(())
                    }
                    CertTaskResult::Error { message } => {
                        warn!(
                            "[task:{}] certificate generation failed CN={} error={}",
                            dispatch.id, params.common_name, message
                        );
                        self.emit(Action::TaskFinished(
                            dispatch.id,
                            TaskResultKind::CertFailed {
                                cn: params.common_name.clone(),
                                error: message.clone(),
                            },
                        ));
                        Err(message)
                    }
                }
            }
            TaskKind::PersistCert {
                output_path,
                cert_pem,
                key_pem,
            } => {
                self.emit(Action::TaskLog(
                    dispatch.id,
                    format!("Persisting certificate to {}", output_path),
                ));
                // Determine target file paths
                let out = std::path::Path::new(&output_path);
                let (cert_path, key_path) = if out.is_dir() {
                    (out.join("cert.pem"), out.join("key.pem"))
                } else {
                    match out.extension().and_then(|e| e.to_str()) {
                        Some("pem") => {
                            let cert = out.to_path_buf();
                            let stem = out.file_stem().and_then(|s| s.to_str()).unwrap_or("key");
                            let key = out.with_file_name(format!("{stem}.key.pem"));
                            (cert, key)
                        }
                        _ => (out.with_extension("pem"), out.with_extension("key.pem")),
                    }
                };

                // Ensure directories exist
                if let Some(dir) = cert_path.parent() {
                    if let Err(e) = std::fs::create_dir_all(dir) {
                        let path_str = cert_path.to_string_lossy().to_string();
                        warn!(
                            "[task:{}] failed to create directory for {}: {}",
                            dispatch.id, path_str, e
                        );
                        self.emit(Action::TaskFinished(
                            dispatch.id,
                            TaskResultKind::PersistFailed {
                                path: path_str,
                                error: e.to_string(),
                            },
                        ));
                        return Err(e.to_string());
                    }
                }
                if let Some(dir) = key_path.parent() {
                    if let Err(e) = std::fs::create_dir_all(dir) {
                        let path_str = key_path.to_string_lossy().to_string();
                        warn!(
                            "[task:{}] failed to create directory for {}: {}",
                            dispatch.id, path_str, e
                        );
                        self.emit(Action::TaskFinished(
                            dispatch.id,
                            TaskResultKind::PersistFailed {
                                path: path_str,
                                error: e.to_string(),
                            },
                        ));
                        return Err(e.to_string());
                    }
                }

                // Write files
                if let Err(e) = std::fs::write(&cert_path, cert_pem) {
                    let path_str = cert_path.to_string_lossy().to_string();
                    warn!(
                        "[task:{}] failed to write certificate {}: {}",
                        dispatch.id, path_str, e
                    );
                    self.emit(Action::TaskFinished(
                        dispatch.id,
                        TaskResultKind::PersistFailed {
                            path: path_str,
                            error: e.to_string(),
                        },
                    ));
                    return Err(e.to_string());
                }
                if let Err(e) = std::fs::write(&key_path, key_pem) {
                    let path_str = key_path.to_string_lossy().to_string();
                    warn!(
                        "[task:{}] failed to write key {}: {}",
                        dispatch.id, path_str, e
                    );
                    self.emit(Action::TaskFinished(
                        dispatch.id,
                        TaskResultKind::PersistFailed {
                            path: path_str,
                            error: e.to_string(),
                        },
                    ));
                    return Err(e.to_string());
                }

                let saved_path = cert_path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| cert_path.to_string_lossy().to_string());

                info!(
                    "[task:{}] persisted certificate and key at {}",
                    dispatch.id, saved_path
                );
                self.emit(Action::TaskFinished(
                    dispatch.id,
                    TaskResultKind::Persisted { path: saved_path },
                ));
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::effects::TaskKind;
    use crate::domain::certs::SelfSignedParams;

    #[tokio::test]
    async fn executor_accepts_task() {
        let exec = TaskExecutor::new();
        let params = SelfSignedParams {
            common_name: "test.local".into(),
            dns_names: vec!["test.local".into()],
            valid_days: 30,
            key_bits: 2048,
            output_path: None,
        };
        let id = exec.spawn(TaskKind::GenerateCert(params));
        assert!(id > 0);
        // Allow worker to process (no deterministic signal yet; future TaskFinished event will improve this).
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}
