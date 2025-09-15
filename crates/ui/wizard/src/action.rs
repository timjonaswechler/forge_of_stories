//! Action system for Wizard
//!
//! Goals of this refactor:
//! - Keep existing core actions working (Tick, Render, Resize, Suspend, Resume, Quit, ClearScreen, Error, Help).
//! - Introduce a split between UI actions, App actions, and App-logic actions.
//! - Add primitives for a layer system (popups and notifications) and background tasks.
//!
//! Migration strategy:
//! - Existing code matching on the legacy variants continues to work.
//! - New flows can send `Action::Ui(..)`, `Action::App(..)`, and `Action::Logic(..)` over the same channel.
//! - Over time we can move app code to match the structured variants.

use crossterm::event::KeyEvent;
use serde::{Deserialize, Serialize};
use strum::Display;

/// Top-level action routed through the application.
///
/// Back-compat note:
/// - Legacy variants are kept so current `match` arms keep compiling.
/// - New structured variants allow a cleaner separation of concerns.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum Action {
    // ----------------
    // Legacy/core actions (kept for compatibility with existing code)
    // ----------------
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,

    // ----------------
    // New: Structured actions
    // ----------------
    /// UI-only intents (focus, layers, visual state, edit mode, notifications).
    Ui(UiAction),
    /// App-level intents (navigation, page selection, settings application).
    App(AppAction),
    /// Application logic / background tasks / IO / long-running operations.
    Logic(LogicAction),
}

//
// UI: focus, layers, edit-mode, notifications
//

/// UI Mode of interaction. Normal vs Edit (e.g., when modifying values).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum UiMode {
    Normal,
    Edit,
}

/// Layers that may be drawn above base content.
/// - Popup: modal, blocks interaction with lower layers; covers full available area by policy.
/// - ModalOverlay: semi-modal overlay (e.g. dim / focus trap) that blocks component interaction but may allow certain global keys.
/// - Notification: ephemeral, highest visual layer; does not necessarily block interaction.
/// - Overlay: non-modal decorative or informational layer drawn above base components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum LayerKind {
    Popup,
    ModalOverlay,
    Notification,
    Overlay,
}

/// Notification severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// A UI notification payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub level: NotificationLevel,
    pub message: String,
    /// Optional auto-dismiss timeout in milliseconds.
    pub timeout_ms: Option<u64>,
}

/// UI-scoped actions: focus, layers, notifications, edit-mode toggles.
/// These should not perform IO or mutate app-wide state directly.
#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum UiAction {
    // Focus management on the current page
    FocusNext,
    FocusPrev,
    /// Focus component by an identifier (page-defined).
    FocusById(String),

    // Component-level navigation actions
    /// Navigate up within a component (e.g., list item up)
    NavigateUp,
    /// Navigate down within a component (e.g., list item down)
    NavigateDown,
    /// Navigate left within a component
    NavigateLeft,
    /// Navigate right within a component
    NavigateRight,
    /// Activate/select the current item in a component
    ActivateSelected,

    // Page-level selection for UI-only routing (does not change app routing by itself)
    /// Informative: the UI reports the focused component name (for status bars/tooltips).
    ReportFocusedComponent(String),

    // Edit mode
    ToggleEditMode,
    EnterEditMode,
    ExitEditMode,

    // Layering primitives
    /// Push a popup layer with a known ID (page/component decides what to render).
    /// Optional priority overrides auto-incremented stacking order (higher draws later).
    OpenPopup {
        id: String,
        priority: Option<i32>,
    },
    /// Close a popup layer by ID (no-op if missing).
    ClosePopup {
        id: String,
    },
    /// Open a modal overlay (blocks component focus/interaction; sits beneath popups by priority rules).
    OpenModalOverlay {
        id: String,
        priority: Option<i32>,
    },
    /// Close a modal overlay layer by ID (no-op if missing).
    CloseModalOverlay {
        id: String,
    },
    /// Close all modal overlay layers (non-popups).
    CloseAllModalOverlays,
    /// Push a generic layer kind (e.g., overlay). Concrete meaning is page-defined.
    PushLayer(LayerKind),
    /// Pop the top-most layer (if any).
    PopLayer,
    /// Close only the top-most popup layer (if any).
    CloseTopPopup,
    /// Close all popup layers.
    CloseAllPopups,

    // Notifications
    ShowNotification(Notification),
    /// Informative: the UI reports current visible notifications count (for status bars).
    ReportNotificationCount(u32),
    /// Informative: highest severity present among current notifications (None if no toasts).
    ReportNotificationSeverity(Option<NotificationLevel>),
    /// Informative: whether the Help popup is currently visible.
    ReportHelpVisible(bool),
    /// Dismiss a notification by ID.
    DismissNotification {
        id: String,
    },

    // Help pop-up controls
    /// Toggle inclusion of global key bindings in Help.
    HelpToggleGlobal,
    /// Begin interactive input flow for Help search (UI should open an input prompt).
    BeginHelpSearch,
    /// Forward a raw KeyEvent to the help prompt widget.
    HelpPromptKey(KeyEvent),
    /// Live report of the current help search input buffer.
    ReportHelpSearchBuffer(String),
    /// Set a search query for Help (case-insensitive).
    HelpSearch(String),
    /// Clear the active Help search filter.
    HelpSearchClear,

    /// Persist the current 'show_global' preference for Help into settings.
    PersistHelpShowGlobal(bool),
    /// Toggle line wrapping in Help content.
    HelpToggleWrap,
    /// Persist the current 'wrap_on' preference for Help into settings.
    PersistHelpWrapOn(bool),
    /// Cycle to next page (cyclic order)
    NextPage,
    /// Cycle to previous page (cyclic order)
    PrevPage,
}

//
// App-level: navigation, page selection, settings, keymap contexts
//

/// App-scoped actions: navigation between pages, keymap context changes,
/// app mode exposure to UI, and other "application shell" state changes.
#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum AppAction {
    /// Set the active page by its stable ID (page registry decides mapping).
    SetActivePage {
        id: String,
    },

    /// Update the current keymap context (e.g., "global", "setup", "dashboard").
    SetKeymapContext {
        name: String,
    },

    /// Expose UI mode change as app-level state (the UI may request, app confirms).
    SetUiMode(UiMode),

    /// Persist/Load app settings, if supported by the current page/app state.
    SaveSettings,
    LoadSettings,
}

//
// Logic: background tasks, async IO, domain operations
//

/// Identifier for background tasks.
pub type TaskId = String;

/// Background task category (for metrics/UX).
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum TaskKind {
    Io,
    Network,
    Compute,
    Other,
}

/// Spawn specification for a background task.
/// For early scaffolding we keep this generic; callers can encode JSON payloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskSpec {
    pub kind: TaskKind,
    /// Human-friendly label for UX elements (task list, status line).
    pub label: String,
    /// Optional opaque payload (e.g., serialized params).
    pub payload_json: Option<String>,
}

/// Progress update for a background task.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskProgress {
    pub id: TaskId,
    /// 0.0 to 1.0, if known. Use None for indeterminate progress.
    pub fraction: Option<f32>,
    /// Optional status message for UI.
    pub message: Option<String>,
}

/// Result summary for a completed task. Keep generic for now.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskResult {
    pub id: TaskId,
    pub success: bool,
    /// Optional machine-readable payload.
    pub result_json: Option<String>,
    /// Optional human-readable message (errors or success summary).
    pub message: Option<String>,
}

/// App-logic actions: spawning and reporting background tasks and other domain actions.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum LogicAction {
    // Background tasks lifecycle
    SpawnTask(TaskSpec),
    CancelTask { id: TaskId },
    TaskStarted { id: TaskId },
    TaskProgress(TaskProgress),
    TaskCompleted(TaskResult),

    // Domain operations (config, IO) – extend as needed
    LoadConfig,
    SaveConfig,
}

// Convenience conversions to ease migration to structured actions.
// These allow sending structured actions while legacy handlers continue to match on core variants.

impl From<UiAction> for Action {
    fn from(value: UiAction) -> Self {
        Action::Ui(value)
    }
}

impl From<AppAction> for Action {
    fn from(value: AppAction) -> Self {
        Action::App(value)
    }
}

impl From<LogicAction> for Action {
    fn from(value: LogicAction) -> Self {
        Action::Logic(value)
    }
}
