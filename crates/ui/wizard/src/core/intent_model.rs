//! Phase 11 – Intent / UI Command / Internal Event Scaffold
//!
//! This module introduces a forward-looking separation model intended to
//! decouple the currently monolithic `Action` enum into clearer semantic
//! layers. It is NOT yet integrated; the purpose of this scaffold is to:
//!
//! - Provide documentation + target enums (`Intent`, `UiCommand`, `InternalEvent`)
//! - Offer an experimental classification helper (`classify_action`) that
//!   maps a legacy `Action` reference into these conceptual buckets
//! - Allow incremental adoption without breaking existing code
//!
//! Rationale
//! ---------
//! The existing `Action` enum currently models a mixture of:
//! - User intentions (e.g. `Quit`, `Navigate`, `FocusNext`)
//! - UI control commands (e.g. `OpenPopup`, `ClosePopup`, `ToggleKeymapOverlay`)
//! - System / domain events (e.g. `PreflightResults`, future TaskFinished callbacks)
//! - UI outcome channel (`UiOutcome`) from popups/forms
//!
//! Splitting these concerns improves:
//! - Testability (pure reducer on `Intent`)
//! - Architectural clarity (UI side-effects separated from domain mutations)
//! - Future replay / determinism (intents become a clean log)
//!
//! Future Direction (Phase Outline)
//! --------------------------------
//! Phase 11.1  Introduce this scaffold (current file) + adapter
//! Phase 11.2  Event loop: translate raw inputs -> Intent / UiCommand
//! Phase 11.3  Reducer refactor to take only `Intent` (and later `InternalEvent`)
//! Phase 11.4  Replace direct `Action` emission internally
//! Phase 11.5  Remove deprecated `Action` variants / slim legacy bridge
//!
//! Notes
//! -----
//! - This file intentionally keeps the data model minimal; unnecessary
//!   complexity is deferred until the first integration step.
//! - Some duplication with existing `UiOutcome` is normal during migration.
//!
//! License: Same as crate.
//!
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use crate::action::{Action, PreflightItem, UiOutcome};

/// High-level user intention (pure, declarative).
///
/// These should be derivable from user input (keys, mouse, CLI signals)
/// or synthesized from higher-level UI flows (e.g. wizard step commit).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    Quit,
    NavigateIndex(usize),
    FocusNext,
    FocusPrev,
    Submit,
    Refresh,
    Resize(u16, u16),
    /// Generic navigate alternative (future replacement for index-based navigation)
    #[allow(unused)]
    NavigateDashboard,
    #[allow(unused)]
    NavigateSettings,
    #[allow(unused)]
    NavigateSetup,
    #[allow(unused)]
    NavigateHealth,
    /// Placeholder for text input submission (distinct from generic Submit if needed)
    #[allow(unused)]
    SubmitText,
}

/// Imperative UI control commands (side-effectful at the presentation layer).
///
/// These are consumed by the UI / rendering / popup manager – not by pure reducers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiCommand {
    OpenPopup,
    ClosePopup,
    ToggleKeymapOverlay,
    OpenAlert {
        title: String,
        message: String,
    },
    /// Force a redraw (explicit render request)
    #[allow(unused)]
    RenderNow,
}

/// Internal / domain-driven events injected back into the system
/// (e.g., results from async tasks, environment changes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InternalEvent {
    PreflightUpdated(Vec<PreflightItem>),
    /// Placeholder for future async task completion:
    #[allow(unused)]
    TaskFinished {
        id: u64,
        kind: String,
        success: bool,
        message: Option<String>,
    },
    /// Legacy error propagation (will evolve into structured error events)
    #[allow(unused)]
    Error(String),
}

/// Result of classifying a legacy `Action` into the target conceptual model.
///
/// Any field may be `Some(...)` if the legacy action spans multiple roles,
/// but most actions classify cleanly into a single bucket.
#[derive(Debug, Clone)]
pub struct ActionSplit {
    pub intent: Option<Intent>,
    pub ui_command: Option<UiCommand>,
    pub internal_event: Option<InternalEvent>,
    pub ui_outcome: Option<UiOutcome>,
    /// Original action (by reference identity) hint for debugging (label only).
    pub debug_label: &'static str,
}

impl ActionSplit {
    pub fn empty() -> Self {
        Self {
            intent: None,
            ui_command: None,
            internal_event: None,
            ui_outcome: None,
            debug_label: "empty",
        }
    }

    pub fn with_intent(i: Intent, label: &'static str) -> Self {
        Self {
            intent: Some(i),
            debug_label: label,
            ..Self::empty()
        }
    }

    pub fn with_ui_command(c: UiCommand, label: &'static str) -> Self {
        Self {
            ui_command: Some(c),
            debug_label: label,
            ..Self::empty()
        }
    }

    pub fn with_internal(ev: InternalEvent, label: &'static str) -> Self {
        Self {
            internal_event: Some(ev),
            debug_label: label,
            ..Self::empty()
        }
    }

    pub fn with_outcome(o: UiOutcome, label: &'static str) -> Self {
        Self {
            ui_outcome: Some(o),
            debug_label: label,
            ..Self::empty()
        }
    }
}

/// Classify a legacy `Action` into (Intent | UiCommand | InternalEvent | UiOutcome).
///
/// This is a non-invasive helper used during migration. It does not consume
/// the action and therefore cannot extract owned popup instances (for
/// `OpenPopup` we only emit the semantic command, not the boxed component).
///
/// IMPORTANT:
/// - This function will be extended as new Action variants are migrated.
/// - For unhandled variants we return an empty split (caller decides what to do).
pub fn classify_action(a: &Action) -> ActionSplit {
    use Action::*;
    match a {
        Quit => ActionSplit::with_intent(Intent::Quit, "Quit"),
        Navigate(idx) => ActionSplit::with_intent(Intent::NavigateIndex(*idx), "Navigate"),
        FocusNext => ActionSplit::with_intent(Intent::FocusNext, "FocusNext"),
        FocusPrev => ActionSplit::with_intent(Intent::FocusPrev, "FocusPrev"),
        Submit => ActionSplit::with_intent(Intent::Submit, "Submit"),
        Resize(w, h) => ActionSplit::with_intent(Intent::Resize(*w, *h), "Resize"),
        Refresh => ActionSplit::with_intent(Intent::Refresh, "Refresh"),
        OpenPopup(_) => ActionSplit::with_ui_command(UiCommand::OpenPopup, "OpenPopup"),
        ClosePopup => ActionSplit::with_ui_command(UiCommand::ClosePopup, "ClosePopup"),
        ToggleKeymapOverlay => {
            ActionSplit::with_ui_command(UiCommand::ToggleKeymapOverlay, "ToggleKeymapOverlay")
        }
        PreflightResults(items) => ActionSplit::with_internal(
            InternalEvent::PreflightUpdated(items.clone()),
            "PreflightResults",
        ),
        UiOutcome(outcome) => ActionSplit::with_outcome(outcome.clone(), "UiOutcome"),
        // Ignored / legacy passthrough (can be extended later):
        PopupResult(p) => ActionSplit {
            ui_outcome: Some(p.clone().into()),
            debug_label: "PopupResult(legacy)",
            ..ActionSplit::empty()
        },
        // Unclassified (yet): Tick, Render, Suspend, Resume, Refresh, Error, Help,
        // Up/Down, SwitchInputMode, SetMode, CycleMode, Update, IdleTimeout, etc.
        _ => ActionSplit::empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, PreflightItem, PreflightStatus, UiOutcome as LegacyOutcome};

    #[test]
    fn classify_quit_as_intent() {
        let split = classify_action(&Action::Quit);
        assert!(matches!(split.intent, Some(Intent::Quit)));
        assert_eq!(split.debug_label, "Quit");
    }

    #[test]
    fn classify_navigation() {
        let split = classify_action(&Action::Navigate(2));
        assert!(matches!(split.intent, Some(Intent::NavigateIndex(2))));
    }

    #[test]
    fn classify_preflight_internal_event() {
        let items = vec![PreflightItem {
            label: "X".into(),
            status: PreflightStatus::Present,
            message: None,
        }];
        let split = classify_action(&Action::PreflightResults(items.clone()));
        match split.internal_event {
            Some(InternalEvent::PreflightUpdated(v)) => assert_eq!(v.len(), 1),
            other => panic!("unexpected internal_event: {:?}", other),
        }
    }

    #[test]
    fn classify_ui_outcome_submit_json() {
        let json = serde_json::json!({"k":"v"});
        let split = classify_action(&Action::UiOutcome(LegacyOutcome::SubmitJson(json.clone())));
        match split.ui_outcome {
            Some(UiOutcome::SubmitJson(v)) => assert_eq!(v["k"], "v"),
            _ => panic!("expected SubmitJson outcome"),
        }
    }

    #[test]
    fn open_popup_maps_to_ui_command() {
        // We cannot construct a real popup here easily without pulling extra deps;
        // we only assert classification pattern.
        // Using a dummy closure to produce an `Action::OpenPopup` is not possible
        // without a concrete Component; thus we skip direct instantiation and
        // only verify non-crash classification pathway on a representative sample.
        // (Future: add a lightweight dummy popup implementing Component for tests.)
        // For now we simply ensure no panic on pattern:
        let a = Action::ToggleKeymapOverlay;
        let split = classify_action(&a);
        assert!(split.ui_command.is_some());
    }
}
