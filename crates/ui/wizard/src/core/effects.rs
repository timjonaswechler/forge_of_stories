/*!
Core effect & task model (Phase 10 / groundwork for Executor + Phase 11 Intent split)

This module consolidates the effect system introduced incrementally in earlier
phases (initially prototyped inside `core::reducer`). It will become the single
source of truth for:

- Effect enum (declarative side‑effect descriptions)
- TaskKind enum (asynchronous / long‑running or IO bound work units)
- Future: TaskResult / InternalEvent mapping
- Helper constructors & utilities

Temporary State:
Currently `Effect` and `TaskKind` are still also defined inside `core::reducer`.
After adding this file, the next step is to:
  1. Remove the duplicate definitions from `core::reducer`
  2. Adjust imports there and in `core::loop` to use `crate::core::effects::*`

Doing it in two steps keeps this change isolated (safer incremental commit).

Design Principles:
- Reducer stays pure: it only returns `Vec<Effect>`
- Event loop (or a dedicated TaskExecutor) interprets `Effect::Async(TaskKind)` and spawns tasks
- Completion of tasks re-enters the system as an Action / InternalEvent (migration coming in Phase 11)
- Effect variants are intentionally minimal & additive

Planned Extensions:
- Effect::Batch(Vec<Effect>) for atomic grouping
- TaskKind variants for: PreflightRefresh, PersistSettings, FetchHealthStatus
- Result channel: TaskResultKind + mapping to InternalEvent enum (post Intent/Action split)
- Telemetry hooks & structured logging wrappers

*/

use std::fmt;

use crate::domain::certs::SelfSignedParams;

/// Declarative instruction emitted by the reducer.
///
/// The event loop (or a future TaskExecutor) interprets these.
/// Keep variants cohesive and low in number; push specifics into `TaskKind`
/// or attach structured data as needed.
#[derive(Debug, Clone)]
pub enum Effect {
    /// Explicit "no effect" marker (can be filtered out easily).
    None,
    /// Schedule / spawn an asynchronous task (background work).
    Async(TaskKind),
    /// Lightweight side-effect: log a message (info-level semantic).
    Log(String),
}

impl Effect {
    pub fn log<T: Into<String>>(msg: T) -> Self {
        Effect::Log(msg.into())
    }
    pub fn async_task(kind: TaskKind) -> Self {
        Effect::Async(kind)
    }
    pub fn none() -> Self {
        Effect::None
    }
}

/// Enumeration of all asynchronous task intents.
///
/// Each variant should hold enough data to execute the task *without* additional
/// mutable global context (pure input). Enrich with future domain structs as
/// they stabilize (e.g., PreflightSpec, PersistRequest).
#[derive(Debug, Clone)]
pub enum TaskKind {
    /// Generate (or plan to generate) a self-signed certificate.
    /// Currently a stub; once implemented this will call into `domain::certs`.
    GenerateCert(SelfSignedParams),
}

impl fmt::Display for TaskKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskKind::GenerateCert(p) => write!(f, "GenerateCert(CN={})", p.common_name),
        }
    }
}

/// Future placeholder for task results returned from the executor.
/// Will be introduced when InternalEvent / TaskFinished plumbing is added.
///
/// Example sketch:
/// ```ignore
/// pub enum TaskResultKind {
///     CertGenerated { cn: String, cert_pem: String, key_pem: String },
///     CertFailed { cn: String, error: String },
/// }
/// ```
///
/// Rationale for deferring:
/// - Keeps this commit focused on introducing the canonical Effect + TaskKind home.
/// - Avoids unused-type warnings until the executor + return channel land.
#[allow(dead_code)]
pub enum _TaskResultKind {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_log_effect() {
        let e = Effect::log("hello");
        match e {
            Effect::Log(s) => assert_eq!(s, "hello"),
            _ => panic!("expected Log variant"),
        }
    }

    #[test]
    fn construct_async_effect() {
        let params = SelfSignedParams {
            common_name: "example.test".into(),
            dns_names: vec!["example.test".into()],
            valid_days: 90,
            key_bits: 2048,
        };
        let e = Effect::async_task(TaskKind::GenerateCert(params.clone()));
        match e {
            Effect::Async(TaskKind::GenerateCert(p)) => {
                assert_eq!(p.common_name, "example.test");
                assert_eq!(p.valid_days, 90);
            }
            _ => panic!("expected Async GenerateCert"),
        }
    }
}
