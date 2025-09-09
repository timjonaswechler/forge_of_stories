//! Core application state (Phase 4.1)
//!
//! Introduces the transitional high-level `AppState` enum that will
//! eventually replace ad-hoc `pages: Vec<Box<dyn Page>>` + `active_page`
//! navigation logic.
//!
//! Phase Scope (4.1):
//!   - Define `AppState` enum and lightweight sub-state structs.
//!   - Provide constructors + a helper to derive the initial state
//!     from CLI arguments (`Cli` / `Cmd` / `RunMode`).
//!   - Do NOT remove existing page system yet (coexistence).
//!
//! Upcoming (4.2 / 4.3):
//!   - Introduce a reducer (`reduce(...)`) that consumes Intents and
//!     mutates this state.
//!   - Centralize navigation & focus handling.
//!
//! Design Notes:
//!   - Sub-states are intentionally minimal; they will accumulate
//!     domain-specific data as we migrate logic out of Pages.
//!   - The enum enables pattern matching for future reducers / effects.
//!   - `RootState` is added as an integration point for the future
//!     reducer (will wrap `AppState` and possibly other top-level
//!     concerns like async task registries or effect queues).
//!
//! Migration Strategy:
//!   1. Introduce `AppState` (this file).
//!   2. Start populating / mirroring data from existing Page objects.
//!   3. Gradually route Intents through a reducer operating on `RootState`.
//!
//! All code here is additive and side-effect free for now.

use crate::cli::{Cli, Cmd, RunMode};

/// High-level application mode/state machine anchor.
///
/// Each variant owns a lightweight struct that will grow independently
/// as feature logic is migrated out of page trait objects.
///
/// For now these are simple markers with potential placeholders.
#[derive(Debug)]
pub enum AppState {
    Setup(SetupState),
    Settings(SettingsState),
    Dashboard(DashboardState),
    Health(HealthState),
}

impl AppState {
    /// Human-readable label (useful for debugging / statuslines if needed later).
    pub fn label(&self) -> &'static str {
        match self {
            AppState::Setup(_) => "setup",
            AppState::Settings(_) => "settings",
            AppState::Dashboard(_) => "dashboard",
            AppState::Health(_) => "health",
        }
    }

    /// Returns true if this state corresponds to a read-only / passive view.
    pub fn is_read_only(&self) -> bool {
        matches!(self, AppState::Health(_))
    }
}

/// Root container that will be the target of the future reducer.
/// This allows the reducer signature to evolve without reshaping
/// `WizardApp` prematurely.
#[derive(Debug)]
pub struct RootState {
    pub app_state: AppState,
    // Future: pub focus_ring: FocusRing,
    // Future: pub effects: Vec<Effect>,
}

impl RootState {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    /// Convenience accessor for pattern matching outside.
    pub fn app_state(&self) -> &AppState {
        &self.app_state
    }

    /// Mutable accessor (for reducers).
    pub fn app_state_mut(&mut self) -> &mut AppState {
        &mut self.app_state
    }
}

/// Placeholder state for the setup flow.
#[derive(Debug, Default)]
pub struct SetupState {
    // Example future fields:
    // pub progress: u8,
    // pub checklist: Vec<ChecklistItem>,
}

/// Placeholder state for the settings page.
#[derive(Debug, Default)]
pub struct SettingsState {
    // Example future fields:
    // pub applied_profile: String,
    // pub dirty: bool,
}

/// Placeholder state for the dashboard.
#[derive(Debug, Default)]
pub struct DashboardState {
    // Example future fields:
    // pub server_status: Option<ServerStatus>,
    // pub recent_activity: Vec<ActivityItem>,
}

/// Placeholder state for the health probe UI.
#[derive(Debug, Default)]
pub struct HealthState {
    // Example future fields:
    // pub last_check: Option<Instant>,
    // pub summary: HealthSummary,
}

/// Derive the initial `AppState` based on CLI invocation.
///
/// This mirrors the existing startup branching in `WizardApp::new`.
pub fn initial_app_state(cli: &Cli) -> AppState {
    match &cli.cmd {
        Cmd::Run { mode } => match mode {
            RunMode::Setup => AppState::Setup(SetupState::default()),
            RunMode::Dashboard => AppState::Dashboard(DashboardState::default()),
        },
        Cmd::Health => AppState::Health(HealthState::default()),
    }
}

/// Transitional helper to wrap the initial state in a `RootState`.
pub fn initial_root_state(cli: &Cli) -> RootState {
    RootState::new(initial_app_state(cli))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cmd, RunMode};

    fn dummy_cli(cmd: Cmd) -> Cli {
        // Reusing clap's derive structure; constructing manually is fine for tests.
        Cli { cmd }
    }

    #[test]
    fn setup_mode_initializes_setup_state() {
        let cli = dummy_cli(Cmd::Run { mode: RunMode::Setup });
        let rs = initial_root_state(&cli);
        assert!(matches!(rs.app_state(), AppState::Setup(_)));
    }

    #[test]
    fn dashboard_mode_initializes_dashboard_state() {
        let cli = dummy_cli(Cmd::Run { mode: RunMode::Dashboard });
        let rs = initial_root_state(&cli);
        assert!(matches!(rs.app_state(), AppState::Dashboard(_)));
    }

    #[test]
    fn health_mode_initializes_health_state() {
        let cli = dummy_cli(Cmd::Health);
        let rs = initial_root_state(&cli);
        assert!(matches!(rs.app_state(), AppState::Health(_)));
    }

    #[test]
    fn labels_are_stable() {
        let cli = dummy_cli(Cmd::Run { mode: RunMode::Setup });
        let rs = initial_root_state(&cli);
        assert_eq!(rs.app_state().label(), "setup");
    }
}
