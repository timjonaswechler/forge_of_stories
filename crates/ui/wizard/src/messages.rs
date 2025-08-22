#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::cmp::Eq;
/// Shared messaging types for Wizard orchestration.
///
/// This module defines the enums used to communicate between the TUI wizard,
/// the game logic (Aether), and the optional web UI (Oracle).
///
/// Suggested usage:
/// - `WizardToAether` from the wizard to control Aether.
/// - `AetherToWizard` from Aether to report status back.
/// - `WizardToOracle` from the wizard to control the web UI.
/// - `OracleToWizard` from the web UI back to the wizard.
/// - `WizardSettingsMsg` for pre-start settings modifications.

/// Messages intended for the settings layer (editable before the server starts).
#[derive(Debug, Clone)]
pub enum WizardSettingsMsg {
    /// Apply a single key/value setting (placeholder; replace with structured settings as you evolve).
    ApplyKeyValue { key: String, value: String },
    /// Persist settings to disk.
    Save,
    /// Reload settings from disk.
    Reload,
}

/// Control messages from the TUI wizard to the game logic (Aether).
#[derive(Debug, Clone)]
pub enum WizardToAether {
    /// Start the game server (only valid when currently stopped).
    StartServer,
    /// Stop the game server (graceful).
    StopServer,
    /// Shutdown request that should end the current server task.
    Shutdown,
    /// Example runtime tuning hook; replace with structured commands as needed.
    ApplyRuntimeSetting { key: String, value: String },
}

/// Lightweight status snapshot for dashboard updates.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AetherStatsSnapshot {
    pub uptime_secs: u64,
    pub players: usize,
    // Optional extensions: tick rate, jitter, network stats, etc.
    // pub tick_hz: u32,
    // pub tick_ms_avg: f32,
    // pub tick_ms_p99: f32,
    // pub quic_bind_addr: String,
    // pub conns: usize,
}

/// Status/events flowing from the game logic (Aether) back to the wizard.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum AetherToWizard {
    /// The server has started successfully.
    ServerStarted,
    /// The server has stopped (either gracefully or due to an error).
    ServerStopped,
    /// Lightweight status payload for dashboard updates.
    Stats(AetherStatsSnapshot),
    /// A non-fatal error or notification from the server.
    Error(String),
}

/// Messages to the optional web UI (Oracle).
#[derive(Debug, Clone)]
pub enum WizardToOracle {
    /// Ask the web UI layer to broadcast the latest status to connected clients.
    BroadcastStatus,
    /// Request the web UI layer to shutdown gracefully.
    Shutdown,
}

/// Messages from the optional web UI (Oracle) back to the wizard.
#[derive(Debug, Clone)]
pub enum OracleToWizard {
    /// Web UI requests the latest status snapshot from the wizard.
    RequestStatus,
    /// Web UI proposes a settings change (e.g., via form submission).
    ApplySetting { key: String, value: String },
}
