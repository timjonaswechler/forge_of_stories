//! Reducer Prototype (Phase 4.2)
//!
//! This module introduces an initial, minimalist reducer function operating
//! solely on the high-level `RootState` (see `core/state.rs`). The goal is to
//! begin centralizing state transitions triggered by user/system "intents"
//! (currently aliased to the existing `Action` enum via `intent.rs`).
//!
//! Scope (Phase 4.2):
//!   * Handle a small subset of intents: Quit, Navigate, Resize
//!   * Mutate only `RootState`; do NOT yet alter `WizardApp` fields directly
//!   * Prepare ground for future effect handling by returning a `Vec<Effect>`
//!
//! Non-goals (for now):
//!   * No side-effects (I/O, async tasks)
//!   * No direct changes to `WizardApp` (integration will follow incrementally)
//!   * No focus management (will be added in Phase 4.3)
//!
//! Integration Strategy:
//!   - The event loop (AppLoop) will call `reduce(&mut app.root_state, intent)`
//!     before (or instead of) mutating `WizardApp` directly for the handled intents.
//!   - A small adapter layer (to be written) can then reconcile `RootState` flags
//!     (`quit_requested`, `pending_navigation`) back into the legacy page system
//!     until that system is deprecated.
//!
//! Navigation Mapping (temporary):
//!   * We map `Navigate(usize)` indices heuristically to semantic `AppState`
//!     variants to demonstrate state machine progression.
//!     - 0 => Setup
//!     - 1 => Settings
//!     - 2 => Dashboard
//!     - 3 => Health
//!     - >=4 => Dashboard (default fallback)
//!
//! Future Enhancements:
//!   * Introduce a distinct `Intent` enum diverging from `Action`
//!   * Add effect enumeration (`Effect::{Log, Async(TaskKind), ...}`)
//!   * Fold focus & popup routing into reducer
//!   * Replace index-based navigation with explicit semantic intents
//!
//! Testing:
//!   * See unit tests at bottom verifying each handled intent mutation.
//!
//! NOTE: This file is additive and does not remove or break existing logic.
//!       The legacy imperative updates inside the loop will be gradually
//!       replaced by reducer-driven state transitions.

use crate::action::{Action as Intent, UiOutcome};
use crate::core::effects::{Effect, TaskKind, TaskResultKind};
use crate::core::state::{
    AppState, DashboardState, HealthState, RootState, SettingsState, SetupState,
};
use crate::domain::certs::SelfSignedParams;
use serde_json::Value;

/// Reduce a single intent into state transitions + produced effects.
///
/// Policy:
///   - Side-effect free (effects are descriptive only)
///   - Unhandled intents are ignored
pub fn reduce(state: &mut RootState, intent: Intent) -> Vec<Effect> {
    let mut effects: Vec<Effect> = Vec::new();

    match intent {
        Intent::Quit => {
            state.quit_requested = true;
            effects.push(Effect::Log("Quit requested".into()));
        }
        Intent::Resize(w, h) => {
            state.last_resize = Some((w, h));
        }
        Intent::Navigate(idx) => {
            let target = map_index_to_app_state(idx, &state.app_state);
            state.pending_navigation = Some(target);
            effects.push(Effect::Log(format!("Navigate -> {}", idx)));
        }
        Intent::FocusNext => {
            if state.focus_total > 0 {
                state.focus_index = (state.focus_index + 1) % state.focus_total;
            }
        }
        Intent::FocusPrev => {
            if state.focus_total > 0 {
                if state.focus_index == 0 {
                    state.focus_index = state.focus_total - 1;
                } else {
                    state.focus_index -= 1;
                }
            }
        }
        // Certificate form submission (Phase 9.2 stub):
        // Detect a form JSON submission that looks like the certificate wizard output.
        Intent::UiOutcome(UiOutcome::SubmitJson(ref value)) => {
            if let Some(params) = extract_cert_params(value) {
                effects.push(Effect::Log(format!(
                    "Certificate form submitted (CN={})",
                    params.common_name
                )));
                effects.push(Effect::Async(TaskKind::GenerateCert(params)));
            }
        }
        Intent::TaskFinished(_id, ref res) => {
            state.last_task = Some(res.clone());
            match res {
                TaskResultKind::CertGenerated { cn, .. } => {
                    effects.push(Effect::Log(format!("Certificate generated for {cn}")));
                }
                TaskResultKind::CertFailed { cn, error } => {
                    effects.push(Effect::Log(format!(
                        "Certificate generation failed for {cn}: {error}"
                    )));
                }
            }
        }
        _ => {}
    }

    effects
}

/// Attempt to parse certificate form submission JSON into `SelfSignedParams`.
/// Returns None if the JSON does not appear to be a certificate wizard payload.
fn extract_cert_params(v: &Value) -> Option<SelfSignedParams> {
    let obj = v.as_object()?;
    // Heuristic: must contain at least "cn" and "output_path"
    let cn = obj.get("cn")?.as_str()?.trim().to_string();
    if cn.is_empty() {
        return None;
    }
    let validity_raw = obj
        .get("validity_days")
        .and_then(|x| x.as_str())
        .unwrap_or("365");
    let validity_days: u32 = validity_raw.parse().unwrap_or(365);

    let algorithm = obj
        .get("algorithm")
        .and_then(|x| x.as_str())
        .unwrap_or("RSA")
        .to_string();

    let _self_signed = obj
        .get("self_signed")
        .and_then(|x| x.as_str())
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(true);

    let dns_names: Vec<String> = obj
        .get("sans")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Some(SelfSignedParams {
        common_name: cn,
        dns_names,
        valid_days: validity_days,
        key_bits: if algorithm.eq_ignore_ascii_case("ECDSA") {
            256
        } else {
            2048
        },
    })
}

/// Map a numeric navigation index (legacy) to a semantic `AppState` variant.
///
/// This is a transitional adapter; once semantic Intents replace index-based
/// navigation, this function can be removed.
fn map_index_to_app_state(index: usize, current: &AppState) -> AppState {
    match index {
        0 => AppState::Setup(SetupState::default()),
        1 => AppState::Settings(SettingsState::default()),
        2 => AppState::Dashboard(DashboardState::default()),
        3 => AppState::Health(HealthState::default()),
        _ => {
            // Fallback:
            // Keep user on a "stable" state; prefer Dashboard unless currently Health.
            match current {
                AppState::Health(_) => AppState::Health(HealthState::default()),
                _ => AppState::Dashboard(DashboardState::default()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Cmd, RunMode};
    use crate::core::state::initial_root_state;

    fn cli_setup() -> Cli {
        Cli {
            cmd: Cmd::Run {
                mode: RunMode::Setup,
            },
        }
    }

    #[test]
    fn quit_sets_quit_requested() {
        let mut rs = initial_root_state(&cli_setup());
        assert!(!rs.quit_requested);
        reduce(&mut rs, Intent::Quit);
        assert!(rs.quit_requested);
    }

    #[test]
    fn resize_updates_size() {
        let mut rs = initial_root_state(&cli_setup());
        reduce(&mut rs, Intent::Resize(120, 40));
        assert_eq!(rs.last_resize, Some((120, 40)));
    }

    #[test]
    fn navigate_sets_pending_state() {
        let mut rs = initial_root_state(&cli_setup());
        assert!(rs.pending_navigation.is_none());
        reduce(&mut rs, Intent::Navigate(2));
        assert!(matches!(
            rs.pending_navigation,
            Some(AppState::Dashboard(_))
        ));
    }

    #[test]
    fn navigate_out_of_range_defaults() {
        let mut rs = initial_root_state(&cli_setup());
        reduce(&mut rs, Intent::Navigate(99));
        assert!(matches!(
            rs.pending_navigation,
            Some(AppState::Dashboard(_))
        ));
    }

    #[test]
    fn map_index_respects_health_stability() {
        // If current is Health and index is unknown, remain in Health.
        let current = AppState::Health(HealthState::default());
        let mapped = map_index_to_app_state(999, &current);
        assert!(matches!(mapped, AppState::Health(_)));
    }

    #[test]
    fn quit_produces_log_effect() {
        let mut rs = initial_root_state(&cli_setup());
        let eff = reduce(&mut rs, Intent::Quit);
        assert!(
            eff.iter()
                .any(|e| matches!(e, Effect::Log(msg) if msg.contains("Quit"))),
            "expected a Log effect for Quit"
        );
    }

    #[test]
    fn certificate_form_generates_async_effect() {
        let mut rs = initial_root_state(&cli_setup());
        let json = serde_json::json!({
            "cn": "example.com",
            "country": "US",
            "validity_days": "365",
            "algorithm": "RSA",
            "self_signed": "true",
            "output_path": "/tmp/cert.pem"
        });
        let eff = reduce(&mut rs, Intent::UiOutcome(UiOutcome::SubmitJson(json)));
        assert!(
            eff.iter()
                .any(|e| matches!(e, Effect::Async(TaskKind::GenerateCert(_)))),
            "expected Async GenerateCert effect"
        );
    }
}
