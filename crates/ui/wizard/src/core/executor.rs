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

use crate::core::effects::TaskKind;
use crate::domain::certs::SelfSignedParams;
use log::{info, warn};

/// Monotonic task identifier type.
pub type TaskId = u64;

/// Public handle for scheduling background tasks.
///
/// Cloneable & cheap: internally only wraps an `mpsc::UnboundedSender`.
#[derive(Clone)]
pub struct TaskExecutor {
    tx: mpsc::UnboundedSender<Dispatch>,
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
        let worker = Worker::new(rx);
        worker.spawn();
        Self { tx }
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
}

impl Worker {
    fn new(rx: mpsc::UnboundedReceiver<Dispatch>) -> Self {
        Self { rx }
    }

    fn spawn(mut self) {
        tokio::spawn(async move {
            while let Some(dispatch) = self.rx.recv().await {
                if let Err(e) = Self::handle(dispatch).await {
                    warn!("Task execution failed: {e}");
                }
            }
            // Channel closed: executor is shutting down.
            info!("TaskExecutor worker stopped (channel closed)");
        });
    }

    async fn handle(dispatch: Dispatch) -> Result<(), String> {
        match dispatch.kind {
            TaskKind::GenerateCert(params) => {
                Self::handle_generate_cert(dispatch.id, params).await?;
            }
        }
        Ok(())
    }

    async fn handle_generate_cert(id: TaskId, params: SelfSignedParams) -> Result<(), String> {
        // Phase 10 stub: only log intent. Real implementation will call
        // `domain::certs::generate_self_signed` and propagate result.
        info!(
            "[task:{id}] (stub) Would generate self-signed certificate CN={} \
             dns_names={:?} days={} key_bits={}",
            params.common_name, params.dns_names, params.valid_days, params.key_bits
        );
        // Simulate minor async yield (optional):
        tokio::task::yield_now().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::effects::TaskKind;

    #[tokio::test]
    async fn executor_accepts_task() {
        let exec = TaskExecutor::new();
        let params = SelfSignedParams {
            common_name: "test.local".into(),
            dns_names: vec!["test.local".into()],
            valid_days: 30,
            key_bits: 2048,
        };
        let id = exec.spawn(TaskKind::GenerateCert(params));
        assert!(id > 0);
        // Allow worker to process (no deterministic signal yet; future TaskFinished event will improve this).
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}
