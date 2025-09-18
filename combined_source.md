# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/action.rs

```rs
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
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/app.rs

```rs
//! Wizard application shell
//!
//! Responsibilities:
//! - Initialize TUI, settings, pages, components
//! - Maintain active page + component focus cycle
//! - Manage layer stack (Overlay, Popup, Notification)
//! - Background task orchestration + action routing
//! - Normal/Edit mode state & keymap context propagation
//!
//! Rendering Pipeline (final form without toast/overlay as components):
//! 1. Base Page (active page)
//! 2. Base Components (page-provided, focusable)
//! 3. Overlay Layers (LayerKind::Overlay; non-modal, drawn in insertion order)
//! 4. Popup Layers (LayerKind::Popup; modal, newest on top; focus lock active while any popup exists)
//! 5. Notifications (LayerKind::Notification; rendered last in a dedicated pass)
//!
//! Toasts + Overlays are now rendered directly by the App / Layer system, not as Components:
//! - Overlay visuals come from the layer registry render pass (no synthetic overlay component).
//! - Notification/Toast drawing uses internal App toast state (lifetime, severity aggregation).
//!
//! Focus Rules:
//! - While any Popup active: component focus traversal disabled (popup_focus_lock).
//! - Closing last popup restores first entry in focus cycle (if available).
//!
//! Navigation:
//! - Page cycling via UiAction::NextPage / UiAction::PrevPage (cyclic).
//!
//! This module intentionally keeps gameplay/UI specifics out; it is infrastructure only.

pub(crate) mod keymap_registry;
pub(crate) mod settings;
pub(crate) mod task_manager;

use self::task_manager::{TaskManager, TaskManagerHandle};
use crate::layers::LayerRegistry;
use crate::layers::page::WelcomePage;
use crate::{
    action::{
        Action, AppAction, LayerKind, LogicAction, NotificationLevel, TaskId, TaskKind,
        TaskProgress, TaskResult, TaskSpec, UiAction, UiMode,
    },
    cli::{Cli, Cmd, RunMode},
    layers::{
        page::{DashboardPage, Page},
        popup::Popup,
    },
    tui::{Event, Tui},
    ui::{
        components::{Component, StatusBar},
        ids::{NotifId, PageId, PopupId},
    },
};
use app::AppBase;
pub use app::init;
use color_eyre::Result;
use crossterm::event::KeyEvent;
use keymap_registry::WizardActionRegistry;
use ratatui::{
    layout::{Rect, Size},
    style::{Color, Style},
    widgets::{Block, Borders, Clear},
};
use settings::ActionRegistry;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Main Wizard application.
pub struct App {
    // Platform
    base: AppBase,

    // Settings stores
    settings: Arc<settings::SettingsStore>,
    aether_settings: Arc<settings::SettingsStore>,

    // UI collections
    pages: HashMap<PageId, Box<dyn Page>>,
    popups: HashMap<PopupId, Box<dyn Popup>>,
    status_bar: StatusBar,

    // Navigation and focus
    active_page_index: Option<usize>,
    focused_component_index: Option<usize>,
    /// Ordered cycle of focusable component indices (derived from page.focus_order()).
    keymap_context: String,
    /// Stack of active contexts for the new keymap system
    active_contexts: Vec<String>,
    /// Action registry for resolving keymap actions
    action_registry: WizardActionRegistry,
    comp_id_to_index: HashMap<String, usize>,

    // Layers
    layers: Vec<LayerEntry>,

    // Background tasks
    tasks: HashMap<TaskId, TaskState>,
    next_task_seq: u64,

    // App state
    should_quit: bool,
    should_suspend: bool,
    ui_mode: UiMode,

    // Timing / perf
    tick_rate: f64,
    frame_rate: f64,
    last_tick_key_events: Vec<KeyEvent>,

    // Action channel
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    task_mgr: TaskManagerHandle,

    // --- Toast / Notification state ---
    toast_notifications: Vec<crate::action::Notification>,
    toast_last_visible: u32,
    toast_last_highest: Option<crate::action::NotificationLevel>,
}

impl App {
    pub fn new(base: AppBase, cli: Cli) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        // Settings stores (wizard + aether). We can split/merge later if needed.
        let settings = Arc::new(settings::build_wizard_settings_store()?);
        let aether_settings = Arc::new(settings::build_wizard_settings_store()?);

        // Initial UI collections; pages/components can be registered here based on CLI.
        let (pages, components, active_page_index, page_label): (
            Vec<Box<dyn Page>>,
            Vec<Box<dyn Component>>,
            Option<usize>,
            String,
        ) = match cli.cmd {
            Cmd::Run { mode } => match mode {
                RunMode::Setup => (
                    vec![Box::new(WelcomePage::new())],
                    vec![],
                    Some(0),
                    "Welcome".to_string(),
                ),
                RunMode::Dashboard => (
                    vec![Box::new(DashboardPage::new())],
                    vec![],
                    Some(0),
                    "Dashboard".to_string(),
                ),
            },
        };

        // Default rates; in the future read from settings::GeneralCfg
        let tick_rate = settings.get::<settings::Wizard>().unwrap().tick_rate;
        let frame_rate = settings.get::<settings::Wizard>().unwrap().fps;
        // Start TaskManager loop; keep a handle to send commands. Join handle is currently discarded.
        let (task_mgr, _task_mgr_join) = TaskManager::new(action_tx.clone());
        let settings_for_layer = settings.clone();

        Ok(Self {
            base,
            settings,
            aether_settings,
            pages,
            components,
            status_bar: StatusBar::new(&page_label),
            active_page_index,
            focused_component_index: None,
            focus_cycle: Vec::new(),
            popup_focus_lock: false,
            keymap_context: "global".to_string(),
            active_contexts: vec!["global".to_string()],
            action_registry: WizardActionRegistry::new(),
            comp_id_to_index: HashMap::new(),

            tasks: HashMap::new(),
            next_task_seq: 0,
            should_quit: false,
            should_suspend: false,
            ui_mode: UiMode::Normal,

            tick_rate,
            frame_rate,
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
            task_mgr,
            toast_notifications: Vec::new(),
            toast_last_visible: 0,
            toast_last_highest: None,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            // .mouse(true) // enable when needed
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        tui.enter()?;

        // Register actions and settings for all pages, then ask them to provide components for registration
        for page in self.pages.iter_mut() {
            page.register_action_handler(self.action_tx.clone())?;
            page.init()?;
        }
        {
            let rect = tui.size()?;
            let size = Size {
                width: rect.width,
                height: rect.height,
            };
            let mut provided_all = Vec::new();
            for page in self.pages.iter_mut() {
                let provided = page.provide_components();
                if !provided.is_empty() {
                    provided_all.extend(provided);
                }
            }
            if !provided_all.is_empty() {
                let new_ids: std::collections::HashSet<String> =
                    provided_all.iter().map(|(id, _)| id.clone()).collect();
                let current_ids: std::collections::HashSet<String> =
                    self.comp_id_to_index.keys().cloned().collect();

                let changed = self.components.len() != provided_all.len() || new_ids != current_ids;

                if changed {
                    self.components.clear();
                    self.comp_id_to_index.clear();
                    self.focused_component_index = None;
                    self.focus_cycle.clear();
                    let _ = self.register_components(provided_all, size)?;
                    if let Some(ix) = self.active_page_index {
                        if let Some(page) = self.pages.get(ix) {
                            for id in page.focus_order() {
                                if let Some(&cix) = self.comp_id_to_index.get(*id) {
                                    self.focus_cycle.push(cix);
                                }
                            }
                            // Auto-derive focus order if page provided none
                            if self.focus_cycle.is_empty() {
                                let mut pairs: Vec<(&String, &usize)> =
                                    self.comp_id_to_index.iter().collect();
                                // Preserve registration / visual order by component index
                                pairs.sort_by_key(|(_, ix)| *ix);
                                for (_, ix) in pairs {
                                    self.focus_cycle.push(*ix);
                                }
                            }
                        }
                    }
                } else if self.focus_cycle.is_empty() {
                    if let Some(ix) = self.active_page_index {
                        if let Some(page) = self.pages.get(ix) {
                            for id in page.focus_order() {
                                if let Some(&cix) = self.comp_id_to_index.get(*id) {
                                    self.focus_cycle.push(cix);
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(ix) = self.active_page_index {
            if let Some(page) = self.pages.get_mut(ix) {
                self.keymap_context = page.keymap_context().to_string();
                let _ = page.focus();
                if self.focused_component_index.is_none() && !self.focus_cycle.is_empty() {
                    self.focused_component_index = Some(self.focus_cycle[0]);
                }
                self.status_bar.set_page(page.id());
            }
        }
        self.apply_focus_state();

        for component in self.components.iter_mut() {
            component.register_action_handler(self.action_tx.clone())?;
        }
        for component in self.components.iter_mut() {
            component.register_settings_handler(self.settings.clone())?;
        }
        {
            let rect = tui.size()?;
            let size = Size {
                width: rect.width,
                height: rect.height,
            };
            for component in self.components.iter_mut() {
                component.init(size)?;
            }
            // Initialize status bar
            self.status_bar
                .register_action_handler(self.action_tx.clone())?;
            self.status_bar
                .register_settings_handler(self.settings.clone())?;
            self.status_bar.init(size)?;
        }

        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx.send(Action::Quit)?,
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,
            _ => {}
        }

        if let Some(ix) = self.active_page_index {
            if let Some(page) = self.pages.get_mut(ix) {
                if let Some(action) = page.handle_events(Some(event.clone()))? {
                    action_tx.send(action)?;
                }
            }
        }
        for component in self.components.iter_mut() {
            if let Some(action) = component.handle_events(Some(event.clone()))? {
                action_tx.send(action)?;
            }
        }
        // Status bar consumes events only for potential future interactive features
        let _ = self.status_bar.handle_events(Some(event));
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        use crossterm::event::{KeyCode, KeyModifiers};
        if self.help_search_active {
            match key.code {
                KeyCode::Enter => {
                    let q = self.help_search_buffer.clone();
                    self.help_search_active = false;
                    self.help_search_buffer.clear();
                    self.action_tx.send(Action::Ui(UiAction::HelpSearch(q)))?;
                    return Ok(());
                }
                KeyCode::Esc => {
                    self.help_search_active = false;
                    self.help_search_buffer.clear();
                    self.action_tx.send(Action::Ui(UiAction::HelpSearchClear))?;
                    return Ok(());
                }
                KeyCode::Backspace | KeyCode::Char(_) => {
                    let _ = self
                        .action_tx
                        .send(Action::Ui(UiAction::HelpPromptKey(key)));
                }
                _ => {
                    let _ = self
                        .action_tx
                        .send(Action::Ui(UiAction::HelpPromptKey(key)));
                }
            }
            return Ok(());
        }
        {
            if key.code == KeyCode::Esc {
                if key.modifiers.contains(KeyModifiers::ALT) {
                    let _ = self.action_tx.send(Action::Ui(UiAction::CloseAllPopups));
                } else if self
                    .layers
                    .iter()
                    .any(|l| matches!(l.kind, LayerKind::Popup | LayerKind::ModalOverlay))
                {
                    let _ = self.action_tx.send(Action::Ui(UiAction::CloseTopPopup));
                } else if self.ui_mode == UiMode::Edit {
                    let _ = self.action_tx.send(Action::Ui(UiAction::ExitEditMode));
                }
                return Ok(());
            }
        }
        let mut mods: Vec<&'static str> = Vec::new();
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            mods.push("ctrl");
        }
        if key.modifiers.contains(KeyModifiers::ALT) {
            mods.push("alt");
        }
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            mods.push("shift");
        }
        let key_str = match key.code {
            KeyCode::Char(c) => c.to_ascii_lowercase().to_string(),
            KeyCode::Enter => "enter".into(),
            KeyCode::Esc => "esc".into(),
            KeyCode::Tab => "tab".into(),
            KeyCode::Backspace => "backspace".into(),
            KeyCode::Left => "left".into(),
            KeyCode::Right => "right".into(),
            KeyCode::Up => "up".into(),
            KeyCode::Down => "down".into(),
            _ => {
                self.last_tick_key_events.push(key);
                return Ok(());
            }
        };
        let chord = if mods.is_empty() {
            key_str
        } else {
            format!("{}+{}", mods.join("+"), key_str)
        };
        // Refresh dynamic contexts (page, mode, layers, focus, etc.) per key event
        self.update_active_contexts();
        let active_contexts = &self.active_contexts;
        if let Some(action) =
            self.settings
                .resolve_action_for_key(&chord, &active_contexts, &self.action_registry)
        {
            self.action_tx.send(action)?;
        } else {
            // println!("DEBUG: No action found for key '{}'", chord);
            self.last_tick_key_events.push(key);
        }
        Ok(())
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            // First, try to route component-specific actions to focused component
            if let Some(response_action) = self.route_action_to_focused_component(&action)? {
                self.action_tx.send(response_action)?;
            }

            match action.clone() {
                Action::Tick => {
                    self.last_tick_key_events.clear();
                    if let Some(ix) = self.active_page_index {
                        if let Some(page) = self.pages.get(ix) {
                            let new_ctx = page.keymap_context().to_string();
                            if new_ctx != self.keymap_context {
                                self.keymap_context = new_ctx.clone();
                                let _ =
                                    self.action_tx
                                        .send(Action::App(AppAction::SetKeymapContext {
                                            name: new_ctx,
                                        }));
                            }
                        }
                    }
                    // Update active contexts on every tick
                    self.update_active_contexts();
                }
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.clear()?,
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::Render => self.render(tui)?,
                Action::Error(msg) => {
                    warn!("UI error: {msg}");
                }
                Action::Help => {
                    if let Some(pos) = self.layers.iter().rposition(|l| {
                        l.kind == LayerKind::Popup && l.id.as_deref() == Some("help")
                    }) {
                        self.layers.remove(pos);
                        let _ = self
                            .action_tx
                            .send(Action::Ui(UiAction::ReportHelpVisible(false)));
                    } else {
                        let next_prio =
                            self.layers.iter().map(|l| l.priority).max().unwrap_or(0) + 1;
                        self.layers.push(LayerEntry {
                            kind: LayerKind::Popup,
                            id: Some("help".into()),
                            priority: next_prio,
                        });
                        let _ = self
                            .action_tx
                            .send(Action::Ui(UiAction::ReportHelpVisible(true)));
                    }
                }
                Action::Ui(ui) => self.handle_ui_action(tui, ui)?,
                Action::App(app) => self.handle_app_action(app)?,
                Action::Logic(logic) => self.handle_logic_action(logic)?,
            }

            self.layer_registry.update_from_action(&action);
            if let Some(ix) = self.active_page_index {
                if let Some(page) = self.pages.get_mut(ix) {
                    if let Some(follow_up) = page.update(action.clone())? {
                        self.action_tx.send(follow_up)?
                    }
                }
            }
            for component in self.components.iter_mut() {
                if let Some(follow_up) = component.update(action.clone())? {
                    self.action_tx.send(follow_up)?
                }
            }
            if let Some(follow_up) = self.status_bar.update(action.clone())? {
                self.action_tx.send(follow_up)?;
            }
        }
        Ok(())
    }

    fn handle_ui_action(&mut self, _tui: &mut Tui, ui: UiAction) -> Result<()> {
        match ui {
            UiAction::FocusNext => {
                if self.popup_focus_lock {
                } else if self.focus_cycle.is_empty() {
                    self.focused_component_index = None;
                } else {
                    let pos = self
                        .focused_component_index
                        .and_then(|cur| self.focus_cycle.iter().position(|&c| c == cur))
                        .unwrap_or(0);
                    let next_pos = (pos + 1) % self.focus_cycle.len();
                    self.focused_component_index = Some(self.focus_cycle[next_pos]);
                    self.apply_focus_state();
                    if let Some((id, _)) = self
                        .comp_id_to_index
                        .iter()
                        .find(|(_, ix)| **ix == self.focused_component_index.unwrap())
                    {
                        let _ = self
                            .action_tx
                            .send(Action::Ui(UiAction::ReportFocusedComponent(id.clone())));
                    }
                }
            }
            UiAction::FocusPrev => {
                if self.popup_focus_lock {
                } else if self.focus_cycle.is_empty() {
                    self.focused_component_index = None;
                } else {
                    let pos = self
                        .focused_component_index
                        .and_then(|cur| self.focus_cycle.iter().position(|&c| c == cur))
                        .unwrap_or(0);
                    let prev_pos = if pos == 0 {
                        self.focus_cycle.len() - 1
                    } else {
                        pos - 1
                    };
                    self.focused_component_index = Some(self.focus_cycle[prev_pos]);
                    self.apply_focus_state();
                    if let Some((id, _)) = self
                        .comp_id_to_index
                        .iter()
                        .find(|(_, ix)| **ix == self.focused_component_index.unwrap())
                    {
                        let _ = self
                            .action_tx
                            .send(Action::Ui(UiAction::ReportFocusedComponent(id.clone())));
                    }
                }
            }
            UiAction::FocusById(id) => {
                if let Some(ix) = self.comp_id_to_index.get(&id).cloned() {
                    self.focused_component_index = Some(ix);
                    self.apply_focus_state();
                }
            }
            UiAction::ReportFocusedComponent(id) => {
                if let Some(ix) = self.comp_id_to_index.get(&id).cloned() {
                    self.focused_component_index = Some(ix);
                    self.apply_focus_state();
                }
            }
            UiAction::ToggleEditMode => {
                self.ui_mode = match self.ui_mode {
                    UiMode::Normal => UiMode::Edit,
                    UiMode::Edit => UiMode::Normal,
                };
                // info!("UI mode changed to {:?}", self.ui_mode);
            }
            UiAction::EnterEditMode => {
                self.ui_mode = UiMode::Edit;
                // info!("UI mode changed to Edit");
            }
            UiAction::ExitEditMode => {
                self.ui_mode = UiMode::Normal;
                // info!("UI mode changed to Normal");
            }
            UiAction::OpenPopup { id, priority } => {
                let is_help = id == "help";
                let next_prio = priority.unwrap_or_else(|| {
                    self.layers.iter().map(|l| l.priority).max().unwrap_or(0) + 1
                });
                self.layers.push(LayerEntry {
                    kind: LayerKind::Popup,
                    id: Some(id.clone()),
                    priority: next_prio,
                });
                self.popup_focus_lock = true;
                self.focused_component_index = None;
                if is_help {
                    let _ = self
                        .action_tx
                        .send(Action::Ui(UiAction::ReportHelpVisible(true)));
                }
            }
            UiAction::ClosePopup { id } => {
                if let Some(pos) = self
                    .layers
                    .iter()
                    .rposition(|l| l.kind == LayerKind::Popup && l.id.as_deref() == Some(&id))
                {
                    self.layers.remove(pos);
                }
                let any_popup = self.layers.iter().any(|l| l.kind == LayerKind::Popup);
                if !any_popup {
                    self.popup_focus_lock = false;
                    if !self.focus_cycle.is_empty() {
                        self.focused_component_index = Some(self.focus_cycle[0]);
                        self.apply_focus_state();
                        if let Some((id, _)) = self
                            .comp_id_to_index
                            .iter()
                            .find(|(_, ix)| **ix == self.focused_component_index.unwrap())
                        {
                            let _ = self
                                .action_tx
                                .send(Action::Ui(UiAction::ReportFocusedComponent(id.clone())));
                        }
                    }
                }
                if id == "help"
                    && !self
                        .layers
                        .iter()
                        .any(|l| l.kind == LayerKind::Popup && l.id.as_deref() == Some("help"))
                {
                    let _ = self
                        .action_tx
                        .send(Action::Ui(UiAction::ReportHelpVisible(false)));
                }
            }
            UiAction::PushLayer(kind) => {
                let next_prio = self.layers.iter().map(|l| l.priority).max().unwrap_or(0) + 1;
                self.layers.push(LayerEntry {
                    kind,
                    id: None,
                    priority: next_prio,
                });
            }
            UiAction::PopLayer => {
                let help_before = self
                    .layers
                    .iter()
                    .any(|l| l.kind == LayerKind::Popup && l.id.as_deref() == Some("help"));
                self.layers.pop();
                let help_after = self
                    .layers
                    .iter()
                    .any(|l| l.kind == LayerKind::Popup && l.id.as_deref() == Some("help"));
                if help_before && !help_after {
                    let _ = self
                        .action_tx
                        .send(Action::Ui(UiAction::ReportHelpVisible(false)));
                }
            }
            UiAction::CloseTopPopup => {
                if self.close_top_popup() {
                    if !self
                        .layers
                        .iter()
                        .any(|l| l.kind == LayerKind::Popup && l.id.as_deref() == Some("help"))
                    {
                        let _ = self
                            .action_tx
                            .send(Action::Ui(UiAction::ReportHelpVisible(false)));
                    }
                }
            }
            UiAction::CloseAllPopups => {
                let had_help = self
                    .layers
                    .iter()
                    .any(|l| l.kind == LayerKind::Popup && l.id.as_deref() == Some("help"));
                self.close_all_popups();
                if had_help {
                    let _ = self
                        .action_tx
                        .send(Action::Ui(UiAction::ReportHelpVisible(false)));
                }
            }
            UiAction::ShowNotification(n) => {
                self.toast_insert(n);
            }
            UiAction::ReportNotificationCount(_n) => {}
            UiAction::ReportNotificationSeverity(_sev) => {}
            UiAction::DismissNotification { id } => {
                self.toast_notifications.retain(|n| n.id != id);
                self.toast_post_prune();
            }
            UiAction::HelpToggleGlobal => {
                self.layer_registry.toggle_show_global();
                let value = self.layer_registry.is_show_global();
                let _ = self
                    .action_tx
                    .send(Action::Ui(UiAction::PersistHelpShowGlobal(value)));
            }
            UiAction::HelpSearch(query) => {
                self.layer_registry.set_help_search(Some(query.clone()));
                if query.trim().is_empty() {
                    None
                } else {
                    Some(query)
                };
            }
            UiAction::HelpSearchClear => {
                self.layer_registry.clear_help_search();
            }
            UiAction::BeginHelpSearch => {
                self.help_search_active = true;
                self.help_search_buffer.clear();
            }
            UiAction::ReportHelpVisible(_v) => {}
            UiAction::PersistHelpShowGlobal(value) => {
                let _ = self
                    .settings
                    .update::<settings::Wizard>(|w| w.help_show_global = value);
            }
            UiAction::HelpToggleWrap => {
                self.layer_registry.toggle_wrap();
                let current = self
                    .settings
                    .get::<settings::Wizard>()
                    .map(|w| w.help_wrap_on)
                    .unwrap_or(true);
                let new_value = !current;
                let _ = self
                    .action_tx
                    .send(Action::Ui(UiAction::PersistHelpWrapOn(new_value)));
            }
            UiAction::PersistHelpWrapOn(value) => {
                let _ = self
                    .settings
                    .update::<settings::Wizard>(|w| w.help_wrap_on = value);
            }
            UiAction::HelpPromptKey(_key) => {}
            UiAction::ReportHelpSearchBuffer(_buf) => {}
            UiAction::OpenModalOverlay { .. } => {}
            UiAction::CloseModalOverlay { .. } => {}
            UiAction::CloseAllModalOverlays => {}
            UiAction::NextPage => {
                if self.pages.len() > 1 {
                    let cur = self.active_page_index.unwrap_or(0);
                    let next = (cur + 1) % self.pages.len();
                    if let Some(page) = self.pages.get(next) {
                        let _ = self.action_tx.send(Action::App(AppAction::SetActivePage {
                            id: page.id().to_string(),
                        }));
                    }
                }
            }
            UiAction::PrevPage => {
                if self.pages.len() > 1 {
                    let cur = self.active_page_index.unwrap_or(0);
                    let prev = if cur == 0 {
                        self.pages.len() - 1
                    } else {
                        cur - 1
                    };
                    if let Some(page) = self.pages.get(prev) {
                        let _ = self.action_tx.send(Action::App(AppAction::SetActivePage {
                            id: page.id().to_string(),
                        }));
                    }
                }
            }
            // Component navigation actions - these are handled by components directly
            UiAction::NavigateUp => {
                // Routed to focused component in route_action_to_focused_component
            }
            UiAction::NavigateDown => {
                // Routed to focused component in route_action_to_focused_component
            }
            UiAction::NavigateLeft => {
                // Routed to focused component in route_action_to_focused_component
            }
            UiAction::NavigateRight => {
                // Routed to focused component in route_action_to_focused_component
            }
            UiAction::ActivateSelected => {
                // Routed to focused component in route_action_to_focused_component
            }
        }
        Ok(())
    }

    fn handle_app_action(&mut self, app: AppAction) -> Result<()> {
        match app {
            AppAction::SetActivePage { id } => {
                if let Some((new_ix, _)) = self.pages.iter().enumerate().find(|(_, p)| p.id() == id)
                {
                    if self.active_page_index != Some(new_ix) {
                        if let Some(old_ix) = self.active_page_index {
                            if let Some(page) = self.pages.get_mut(old_ix) {
                                let _ = page.unfocus();
                            }
                        }
                        self.active_page_index = Some(new_ix);
                        self.components.clear();
                        self.comp_id_to_index.clear();
                        self.focus_cycle.clear();
                        self.focused_component_index = None;
                        let (provided, order_ids): (
                            Vec<(String, Box<dyn Component>)>,
                            Vec<&'static str>,
                        );
                        {
                            let page = self.pages.get_mut(new_ix).expect("page index valid");
                            let p = page.provide_components();
                            let order_slice = page.focus_order();
                            order_ids = order_slice.iter().copied().collect::<Vec<&'static str>>();
                            provided = p;
                            self.status_bar.set_page(page.id());
                            let _ = page.focus();
                        }
                        let size = Size {
                            width: 0,
                            height: 0,
                        };
                        for (cid, comp) in provided {
                            let _ = self.register_component(cid, comp, size);
                        }
                        for id in order_ids {
                            if let Some(&ix) = self.comp_id_to_index.get(id) {
                                self.focus_cycle.push(ix);
                            }
                        }
                        // Auto-derive focus order if page declares none
                        if self.focus_cycle.is_empty() {
                            let mut pairs: Vec<(&String, &usize)> =
                                self.comp_id_to_index.iter().collect();
                            pairs.sort_by_key(|(_, ix)| *ix);
                            for (_, ix) in pairs {
                                self.focus_cycle.push(*ix);
                            }
                        }
                        if let Some(page) = self.pages.get(new_ix) {
                            self.keymap_context = page.keymap_context().to_string();
                            let _ = self
                                .action_tx
                                .send(Action::App(AppAction::SetKeymapContext {
                                    name: self.keymap_context.clone(),
                                }));
                        }
                        if self.focused_component_index.is_none() && !self.focus_cycle.is_empty() {
                            self.focused_component_index = Some(self.focus_cycle[0]);
                            self.apply_focus_state();
                        }
                    }
                } else {
                    info!("Requested page id '{id}' not found");
                }
            }
            AppAction::SetKeymapContext { name } => {
                self.layer_registry.set_keymap_context(name.clone());
                self.keymap_context = name;
            }
            AppAction::SetUiMode(mode) => {
                self.ui_mode = mode;
            }
            AppAction::SaveSettings => {
                info!("SaveSettings requested (not implemented)");
            }
            AppAction::LoadSettings => {
                info!("LoadSettings requested (not implemented)");
            }
        }
        Ok(())
    }

    fn handle_logic_action(&mut self, logic: LogicAction) -> Result<()> {
        match logic {
            LogicAction::SpawnTask(spec) => {
                self.next_task_seq += 1;
                let id = format!("task-{}", self.next_task_seq);
                let state = TaskState {
                    id: id.clone(),
                    spec: spec.clone(),
                    started: false,
                    finished: false,
                    cancelled: false,
                    progress: None,
                    message: None,
                    success: None,
                    result_json: None,
                };
                self.tasks.insert(id.clone(), state);
                let _ = self
                    .action_tx
                    .send(Action::Logic(LogicAction::TaskStarted { id: id.clone() }));
                let _ = self.task_mgr.spawn_simulated(
                    id.clone(),
                    spec.label.clone(),
                    10,
                    150,
                    spec.payload_json.clone(),
                );
            }
            LogicAction::CancelTask { id } => {
                let _ = self.task_mgr.cancel(id.clone());
                if let Some(state) = self.tasks.get_mut(&id) {
                    state.cancelled = true;
                    state.finished = true;
                    state.success = Some(false);
                    state.message = Some("Cancelled".to_string());
                }
            }
            LogicAction::TaskStarted { id } => {
                if let Some(state) = self.tasks.get_mut(&id) {
                    state.started = true;
                }
            }
            LogicAction::TaskProgress(TaskProgress {
                id,
                fraction,
                message,
            }) => {
                if let Some(state) = self.tasks.get_mut(&id) {
                    state.progress = fraction;
                    if let Some(m) = message {
                        state.message = Some(m);
                    }
                }
            }
            LogicAction::TaskCompleted(TaskResult {
                id,
                success,
                result_json,
                message,
            }) => {
                if let Some(state) = self.tasks.get_mut(&id) {
                    state.finished = true;
                    state.success = Some(success);
                    state.result_json = result_json;
                    state.message = message;
                }
            }
            LogicAction::LoadConfig => {
                self.next_task_seq += 1;
                let id = format!("task-{}", self.next_task_seq);
                let spec = TaskSpec {
                    kind: TaskKind::Io,
                    label: "Load config".to_string(),
                    payload_json: None,
                };
                self.tasks.insert(
                    id.clone(),
                    TaskState {
                        id: id.clone(),
                        spec: spec.clone(),
                        started: false,
                        finished: false,
                        cancelled: false,
                        progress: None,
                        message: None,
                        success: None,
                        result_json: None,
                    },
                );
                let _ = self.task_mgr.spawn_load_config(id);
            }
            LogicAction::SaveConfig => {
                self.next_task_seq += 1;
                let id = format!("task-{}", self.next_task_seq);
                let spec = TaskSpec {
                    kind: TaskKind::Io,
                    label: "Save config".to_string(),
                    payload_json: None,
                };
                self.tasks.insert(
                    id.clone(),
                    TaskState {
                        id: id.clone(),
                        spec: spec.clone(),
                        started: false,
                        finished: false,
                        cancelled: false,
                        progress: None,
                        message: None,
                        success: None,
                        result_json: None,
                    },
                );
                let _ = self.task_mgr.spawn_save_config(id);
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            let area = frame.area();
            let status_h: u16 = 1;
            let (main_area, status_area) = if area.height > status_h {
                (
                    ratatui::layout::Rect::new(area.x, area.y, area.width, area.height - status_h),
                    ratatui::layout::Rect::new(
                        area.x,
                        area.y + area.height - status_h,
                        area.width,
                        status_h,
                    ),
                )
            } else {
                (area, area)
            };

            // Page first
            if let Some(ix) = self.active_page_index {
                if let Some(page) = self.pages.get_mut(ix) {
                    if let Err(err) = page.draw(frame, main_area) {
                        let _ = self
                            .action_tx
                            .send(Action::Error(format!("Failed to draw page: {:?}", err)));
                    }
                }
            }
            // Page layout mapping (component id -> Rect) provided by active page
            let page_layout = if let Some(ix) = self.active_page_index {
                if let Some(page) = self.pages.get(ix) {
                    page.layout(main_area)
                } else {
                    crate::layers::page::PageLayout::empty()
                }
            } else {
                crate::layers::page::PageLayout::empty()
            };
            // Draw components using page-provided rects (fallback: main_area)
            for (cid, &cix) in self.comp_id_to_index.iter() {
                if let Some(component) = self.components.get_mut(cix) {
                    let target = page_layout.regions.get(cid).cloned().unwrap_or(main_area);
                    if let Err(err) = component.draw(frame, target) {
                        let _ = self.action_tx.send(Action::Error(format!(
                            "Failed to draw component {cid}: {:?}",
                            err
                        )));
                    }
                }
            }

            // Layers
            let mut overlays = Vec::new();
            let mut popups = Vec::new();
            let mut notifications_present = false;
            for l in &self.layers {
                match l.kind {
                    LayerKind::Overlay | LayerKind::ModalOverlay => overlays.push(l.clone()),
                    LayerKind::Popup => popups.push(l.clone()),
                    LayerKind::Notification => notifications_present = true,
                }
            }
            overlays.sort_by_key(|l| l.priority);
            popups.sort_by_key(|l| l.priority);
            for layer in overlays.into_iter() {
                if let Err(err) = self.layer_registry.render_layer(
                    frame,
                    main_area,
                    layer.kind,
                    layer.id.as_deref(),
                    Some(&self.active_contexts),
                ) {
                    let _ = self.action_tx.send(Action::Error(format!(
                        "Failed to render overlay layer: {:?}",
                        err
                    )));
                }
            }
            if !popups.is_empty() {
                frame.render_widget(Clear, area);
                let dim = Block::default().borders(Borders::NONE).style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::Rgb(200, 200, 200)),
                );
                frame.render_widget(dim, area);
            }
            for layer in popups.into_iter() {
                if let Err(err) = self.layer_registry.render_layer(
                    frame,
                    main_area,
                    layer.kind,
                    layer.id.as_deref(),
                    Some(&self.active_contexts),
                ) {
                    let _ = self.action_tx.send(Action::Error(format!(
                        "Failed to render popup layer: {:?}",
                        err
                    )));
                }
            }
            if notifications_present || !self.toast_notifications.is_empty() {
                self.render_toasts(frame, main_area);
            }
            // Global status bar (drawn last so it stays visible above page baseline)
            if let Err(err) = self.status_bar.draw(frame, status_area) {
                let _ = self.action_tx.send(Action::Error(format!(
                    "Failed to draw status bar: {:?}",
                    err
                )));
            }
        })?;
        Ok(())
    }

    pub fn register_component(
        &mut self,
        id: String,
        mut component: Box<dyn Component>,
        initial_size: Size,
    ) -> Result<usize> {
        component.register_action_handler(self.action_tx.clone())?;
        component.register_settings_handler(self.settings.clone())?;
        component.init(initial_size)?;
        let index = self.components.len();
        self.components.push(component);
        self.comp_id_to_index.insert(id, index);
        if self.focused_component_index.is_none() {
            self.focused_component_index = Some(index);
        }
        self.apply_focus_state();
        Ok(index)
    }

    pub fn register_components(
        &mut self,
        list: Vec<(String, Box<dyn Component>)>,
        initial_size: Size,
    ) -> Result<Vec<usize>> {
        let mut indices = Vec::with_capacity(list.len());
        for (id, comp) in list {
            let ix = self.register_component(id, comp, initial_size)?;
            indices.push(ix);
        }
        Ok(indices)
    }

    fn toast_insert(&mut self, mut n: crate::action::Notification) {
        if n.timeout_ms.is_none() {
            let (lifetime_ms, _) = self.toast_cfg();
            n.timeout_ms = Some(self.now_unix_ms() + lifetime_ms);
        }
        if let Some(pos) = self.toast_notifications.iter().position(|x| x.id == n.id) {
            self.toast_notifications[pos] = n;
        } else {
            self.toast_notifications.push(n);
        }
        self.toast_post_prune();
    }

    fn toast_post_prune(&mut self) {
        let now = self.now_unix_ms();
        self.toast_notifications
            .retain(|n| n.timeout_ms.map(|dl| now < dl).unwrap_or(true));
        let (_, max_visible) = self.toast_cfg();
        if self.toast_notifications.len() > max_visible {
            let keep = self
                .toast_notifications
                .split_off(self.toast_notifications.len() - max_visible);
            self.toast_notifications = keep;
        }
        let visible = self.toast_notifications.len() as u32;
        if visible != self.toast_last_visible {
            self.toast_last_visible = visible;
            let _ = self
                .action_tx
                .send(Action::Ui(UiAction::ReportNotificationCount(visible)));
        }
        let highest =
            self.toast_notifications
                .iter()
                .map(|n| n.level)
                .max_by_key(|lvl| match lvl {
                    NotificationLevel::Error => 4,
                    NotificationLevel::Warning => 3,
                    NotificationLevel::Success => 2,
                    NotificationLevel::Info => 1,
                });
        if highest != self.toast_last_highest {
            self.toast_last_highest = highest;
            let _ = self
                .action_tx
                .send(Action::Ui(UiAction::ReportNotificationSeverity(highest)));
        }
    }

    fn render_toasts(&mut self, frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect) {
        self.toast_post_prune();
        if self.toast_notifications.is_empty() {
            return;
        }
        use ratatui::{
            style::{Color, Style},
            widgets::{Block, Borders, Clear, Paragraph},
        };
        let width = area.width.min(50);
        let height_per = 3u16;
        for (i, notif) in self.toast_notifications.iter().rev().enumerate() {
            let i = i as u16;
            let (x, y) = match self.toast_position {
                crate::layers::ToastPosition::TopRight => (
                    area.x + area.width.saturating_sub(width),
                    area.y + i * height_per,
                ),
                crate::layers::ToastPosition::BottomRight => (
                    area.x + area.width.saturating_sub(width),
                    area.y
                        + area
                            .height
                            .saturating_sub((i + 1) * height_per)
                            .saturating_sub(1),
                ),
                crate::layers::ToastPosition::TopLeft => (area.x, area.y + i * height_per),
                crate::layers::ToastPosition::BottomLeft => (
                    area.x,
                    area.y
                        + area
                            .height
                            .saturating_sub((i + 1) * height_per)
                            .saturating_sub(1),
                ),
            };
            let toast_area = ratatui::layout::Rect::new(
                x,
                y,
                width,
                height_per.min(area.height.saturating_sub(y)),
            );
            let (fg, bg) = match notif.level {
                NotificationLevel::Info => (Color::White, Color::Reset),
                NotificationLevel::Success => (Color::Green, Color::Reset),
                NotificationLevel::Warning => (Color::Yellow, Color::Reset),
                NotificationLevel::Error => (Color::Red, Color::Reset),
            };
            let style = Style::default().fg(fg).bg(bg);
            let block = Block::default()
                .borders(Borders::ALL)
                .style(style)
                .title(format!("Notif — {:?}", notif.level));
            let para = Paragraph::new(notif.message.clone())
                .style(style)
                .block(block);
            frame.render_widget(Clear, toast_area);
            frame.render_widget(para, toast_area);
        }
    }

    fn now_unix_ms(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    fn toast_cfg(&self) -> (u64, usize) {
        let map = self.settings.effective_settings();
        let wiz_tbl = map.get("wizard").and_then(|v| v.as_table());
        let lifetime_ms: u64 = wiz_tbl
            .and_then(|t| t.get("notification_lifetime_ms"))
            .and_then(|v| v.as_integer())
            .map(|n| n as u64)
            .unwrap_or(4000);
        let max_visible: usize = wiz_tbl
            .and_then(|t| t.get("notification_max"))
            .and_then(|v| v.as_integer())
            .map(|n| n as usize)
            .unwrap_or(3);
        (lifetime_ms, max_visible)
    }

    fn close_top_popup(&mut self) -> bool {
        if let Some(pos) = self
            .layers
            .iter()
            .rposition(|l| matches!(l.kind, LayerKind::Popup))
        {
            self.layers.remove(pos);
            if !self
                .layers
                .iter()
                .any(|l| matches!(l.kind, LayerKind::Popup))
            {
                self.popup_focus_lock = false;
                if !self.focus_cycle.is_empty() {
                    self.focused_component_index = Some(self.focus_cycle[0]);
                    self.apply_focus_state();
                }
            }
            return true;
        }
        false
    }

    fn close_all_popups(&mut self) {
        self.layers.retain(|l| !matches!(l.kind, LayerKind::Popup));
        self.popup_focus_lock = false;
        if !self.focus_cycle.is_empty() {
            self.focused_component_index = Some(self.focus_cycle[0]);
            self.apply_focus_state();
        }
    }

    fn apply_focus_state(&mut self) {
        for (i, c) in self.components.iter_mut().enumerate() {
            let is_focused = Some(i) == self.focused_component_index && !self.popup_focus_lock;
            c.set_focused(is_focused);
        }
    }

    /// Updates the active contexts based on current app state
    fn update_active_contexts(&mut self) {
        let mut contexts = vec!["global".to_string()];

        // Add current page context
        if let Some(page_idx) = self.active_page_index {
            if let Some(page) = self.pages.get(page_idx) {
                let page_context = page.keymap_context();
                if page_context != "global" {
                    contexts.push(page_context.to_string());
                }
            }
        }

        // Add UI mode context
        match self.ui_mode {
            UiMode::Edit => contexts.push("edit-mode".to_string()),
            UiMode::Normal => contexts.push("normal-mode".to_string()),
        }

        // Add popup context if any popups are active
        let has_popups = self
            .layers
            .iter()
            .any(|l| matches!(l.kind, LayerKind::Popup));
        if has_popups {
            contexts.push("popup-visible".to_string());
        }

        // Add specific help popup context if help popup is visible
        let help_popup_visible = self
            .layers
            .iter()
            .any(|l| matches!(l.kind, LayerKind::Popup) && l.id.as_deref() == Some("help"));
        if help_popup_visible {
            contexts.push("help-active".to_string());
        }

        // Add modal overlay context if any modal overlays are active
        let has_modal_overlays = self
            .layers
            .iter()
            .any(|l| matches!(l.kind, LayerKind::ModalOverlay));
        if has_modal_overlays {
            contexts.push("modal-overlay-visible".to_string());
        }

        // Add help search context if help search is active
        if self.help_search_active {
            contexts.push("help-search-active".to_string());
        }

        // Add component-specific context if a component is focused
        if let Some(focused_idx) = self.focused_component_index {
            if let Some(component_id) = self.get_component_id_by_index(focused_idx) {
                contexts.push(component_id);
            }
        }

        self.active_contexts = contexts;
    }

    /// Gets the component ID for a given component index
    fn get_component_id_by_index(&self, index: usize) -> Option<String> {
        self.comp_id_to_index
            .iter()
            .find(|(_, ix)| **ix == index)
            .map(|(id, _)| id.clone())
    }

    /// Routes component-specific actions to the currently focused component
    fn route_action_to_focused_component(&mut self, action: &Action) -> Result<Option<Action>> {
        // Only route specific component navigation actions
        match action {
            Action::Ui(UiAction::NavigateUp)
            | Action::Ui(UiAction::NavigateDown)
            | Action::Ui(UiAction::NavigateLeft)
            | Action::Ui(UiAction::NavigateRight)
            | Action::Ui(UiAction::ActivateSelected) => {
                if let Some(focused_idx) = self.focused_component_index {
                    if let Some(component) = self.components.get_mut(focused_idx) {
                        return component.handle_action(action);
                    } else {
                        println!("DEBUG: No component found at focused index {}", focused_idx);
                    }
                } else {
                    println!("DEBUG: No focused component");
                }
            }
            _ => {
                // For other actions, always send to focused component for processing
                if let Some(focused_idx) = self.focused_component_index {
                    if let Some(component) = self.components.get_mut(focused_idx) {
                        return component.handle_action(action);
                    }
                }
            }
        }
        Ok(None)
    }
}

/// Implementation des ActionRegistry-Traits für die Wizard-App
impl ActionRegistry for App {
    type Action = Action;

    fn resolve_action(
        &self,
        action_name: &str,
        action_data: Option<&toml::Value>,
    ) -> Option<Self::Action> {
        debug!(
            "ActionRegistry: Resolving action '{}' with data {:?}",
            action_name, action_data
        );
        match action_name {
            // Legacy/Core Actions
            "Quit" => Some(Action::Quit),
            "Help" => Some(Action::Help),
            "Tick" => Some(Action::Tick),
            "Render" => Some(Action::Render),
            "ClearScreen" => Some(Action::ClearScreen),
            "Suspend" => Some(Action::Suspend),
            "Resume" => Some(Action::Resume),

            // UI Actions
            "FocusNext" | "NextField" => Some(Action::Ui(UiAction::FocusNext)),
            "FocusPrev" | "PrevField" | "PreviousField" => Some(Action::Ui(UiAction::FocusPrev)),

            // Component navigation
            "NavigateUp" => Some(Action::Ui(UiAction::NavigateUp)),
            "NavigateDown" => Some(Action::Ui(UiAction::NavigateDown)),
            "NavigateLeft" => Some(Action::Ui(UiAction::NavigateLeft)),
            "NavigateRight" => Some(Action::Ui(UiAction::NavigateRight)),
            "ActivateSelected" => Some(Action::Ui(UiAction::ActivateSelected)),
            "ToggleEditMode" | "ModeCycle" => Some(Action::Ui(UiAction::ToggleEditMode)),
            "EnterEditMode" | "ModeInsert" => Some(Action::Ui(UiAction::EnterEditMode)),
            "ExitEditMode" | "ModeNormal" => Some(Action::Ui(UiAction::ExitEditMode)),
            "NextPage" => Some(Action::Ui(UiAction::NextPage)),
            "PrevPage" | "PreviousPage" => Some(Action::Ui(UiAction::PrevPage)),

            // Popup/Layer Actions
            "OpenPopup" => {
                if let Some(data) = action_data {
                    if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                        let priority = data
                            .get("priority")
                            .and_then(|v| v.as_integer())
                            .map(|i| i as i32);
                        return Some(Action::Ui(UiAction::OpenPopup {
                            id: id.to_string(),
                            priority,
                        }));
                    }
                }
                // Default: open help popup
                Some(Action::Ui(UiAction::OpenPopup {
                    id: "help".to_string(),
                    priority: None,
                }))
            }
            "ClosePopup" | "CloseTopPopup" => Some(Action::Ui(UiAction::CloseTopPopup)),
            "CloseAllPopups" => Some(Action::Ui(UiAction::CloseAllPopups)),

            // Help Actions
            "HelpToggleGlobal" => Some(Action::Ui(UiAction::HelpToggleGlobal)),
            "HelpToggleWrap" => Some(Action::Ui(UiAction::HelpToggleWrap)),
            "HelpSearch" | "BeginHelpSearch" => Some(Action::Ui(UiAction::BeginHelpSearch)),
            "HelpSearchClear" => Some(Action::Ui(UiAction::HelpSearchClear)),

            // Notification Actions
            "ShowNotification" => {
                if let Some(data) = action_data {
                    let id = data
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("notification")
                        .to_string();
                    let level = data.get("level").and_then(|v| v.as_str()).unwrap_or("info");
                    let message = data
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let timeout_ms = data
                        .get("timeout_ms")
                        .and_then(|v| v.as_integer())
                        .map(|i| i as u64);

                    let notification_level = match level {
                        "error" => crate::action::NotificationLevel::Error,
                        "warning" => crate::action::NotificationLevel::Warning,
                        "success" => crate::action::NotificationLevel::Success,
                        _ => crate::action::NotificationLevel::Info,
                    };

                    return Some(Action::Ui(UiAction::ShowNotification(
                        crate::action::Notification {
                            id,
                            level: notification_level,
                            message,
                            timeout_ms,
                        },
                    )));
                }
                None
            }
            "DismissNotification" => {
                if let Some(data) = action_data {
                    if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                        return Some(Action::Ui(UiAction::DismissNotification {
                            id: id.to_string(),
                        }));
                    }
                }
                None
            }

            // App Actions
            "SetActivePage" => {
                if let Some(data) = action_data {
                    if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                        return Some(Action::App(AppAction::SetActivePage { id: id.to_string() }));
                    }
                }
                None
            }
            "SaveSettings" => Some(Action::App(AppAction::SaveSettings)),
            "LoadSettings" => Some(Action::App(AppAction::LoadSettings)),

            _ => None,
        }
    }

    fn get_action_names(&self) -> Vec<String> {
        vec![
            // Core
            "Quit".to_string(),
            "Help".to_string(),
            "Tick".to_string(),
            "Render".to_string(),
            "ClearScreen".to_string(),
            "Suspend".to_string(),
            "Resume".to_string(),
            // UI Navigation
            "FocusNext".to_string(),
            "NextField".to_string(),
            "FocusPrev".to_string(),
            "PrevField".to_string(),
            "PreviousField".to_string(),
            "NextPage".to_string(),
            "PrevPage".to_string(),
            "PreviousPage".to_string(),
            // Component Navigation
            "NavigateUp".to_string(),
            "NavigateDown".to_string(),
            "NavigateLeft".to_string(),
            "NavigateRight".to_string(),
            "ActivateSelected".to_string(),
            // UI Mode
            "ToggleEditMode".to_string(),
            "ModeCycle".to_string(),
            "EnterEditMode".to_string(),
            "ModeInsert".to_string(),
            "ExitEditMode".to_string(),
            "ModeNormal".to_string(),
            // Popups/Layers
            "OpenPopup".to_string(),
            "ClosePopup".to_string(),
            "CloseTopPopup".to_string(),
            "CloseAllPopups".to_string(),
            // Help
            "HelpToggleGlobal".to_string(),
            "HelpToggleWrap".to_string(),
            "HelpScrollUp".to_string(),
            "HelpScrollDown".to_string(),
            "HelpPageUp".to_string(),
            "HelpPageDown".to_string(),
            "HelpSearch".to_string(),
            "BeginHelpSearch".to_string(),
            "HelpSearchClear".to_string(),
            // Notifications
            "ShowNotification".to_string(),
            "DismissNotification".to_string(),
            // App
            "SetActivePage".to_string(),
            "SaveSettings".to_string(),
            "LoadSettings".to_string(),
        ]
    }
}
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/cli.rs

```rs
// src/cli.rs
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "wizard", version, about = "Forge of Stories admin tool")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Run interactive TUI
    Run {
        #[command(subcommand)]
        mode: RunMode,
    },
    // Health,
    // /// Install components
    // Install {
    //     #[arg(value_enum)]
    //     what: InstallWhat,
    //     version: String,
    // },
    // /// List installed items / versions / mods
    // List {
    //     #[arg(value_enum)]
    //     what: ListWhat,
    //     #[arg(long)]
    //     json: bool,
    // },
}

#[derive(Subcommand)]
pub enum RunMode {
    Setup,
    Dashboard,
}

#[derive(Copy, Clone, ValueEnum)]
pub enum LifecycleAction {
    Start,
    Stop,
    Restart,
}

#[derive(Clone, ValueEnum)]
pub enum InstallWhat {
    Aether,
    Dlc,
    Mod,
}

#[derive(Copy, Clone, ValueEnum)]
pub enum ListWhat {
    Versions,
    Dlcs,
    Mods,
}
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/layers.rs

```rs
pub(crate) mod help;
pub(crate) mod notify;
pub(crate) mod page;
pub(crate) mod popup;
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/main.rs

```rs
mod action;
mod app;
mod cli;
mod layers;
mod tui;
mod ui;

use clap::Parser;
use color_eyre::Result;
use crate::app::App as WizardApp;
use crate::cli::Cli;
use tracing_subscriber::EnvFilter;

// A zero-sized type implementing the platform `Application` trait used by `app::init`.
struct Wizard;

impl ::app::Application for Wizard {
    type Error = ::app::BoxError;

    // Stable application ID for config/data/logs directories.
    const APP_ID: &'static str = "wizard";

    fn init_platform() -> Result<(), Self::Error> {
        // Hook for platform-specific initialization if needed.
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Install logging/tracing (env overrideable via RUST_LOG / RUST_TRACE)
    // Example: RUST_LOG=debug,wizard=debug,settings=debug
    let default_filter = "info,wizard=debug,settings=debug";
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter)),
        )
        .with_target(true)
        .compact()
        .init();

    let cli = Cli::parse();

    // Initialize platform base (paths, etc.)
    let base = crate::app::init::<Wizard>().expect("Initialization went wrong");

    // Build and run the Wizard TUI
    let mut app = WizardApp::new(base, cli)?;
    app.run().await?;
    Ok(())
}
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/tui.rs

```rs
use std::{
    io::{Stdout, stdout},
    ops::{Deref, DerefMut},
    time::Duration,
};
use color_eyre::Result;
use crossterm::{
    cursor,
    event::{
        DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind, MouseEvent,
    },
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{FutureExt, StreamExt};
use ratatui::backend::CrosstermBackend as Backend;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
    time::interval,
};
use tokio_util::sync::CancellationToken;
use tracing::error;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Init,
    Quit,
    Error,
    Closed,
    Tick,
    Render,
    FocusGained,
    FocusLost,
    Paste(String),
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

pub struct Tui {
    pub terminal: ratatui::Terminal<Backend<Stdout>>,
    pub task: JoinHandle<()>,
    pub cancellation_token: CancellationToken,
    pub event_rx: UnboundedReceiver<Event>,
    pub event_tx: UnboundedSender<Event>,
    pub frame_rate: f64,
    pub tick_rate: f64,
    pub mouse: bool,
    pub paste: bool,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        Ok(Self {
            terminal: ratatui::Terminal::new(Backend::new(stdout()))?,
            task: tokio::spawn(async {}),
            cancellation_token: CancellationToken::new(),
            event_rx,
            event_tx,
            frame_rate: 60.0,
            tick_rate: 4.0,
            mouse: false,
            paste: false,
        })
    }

    pub fn tick_rate(mut self, tick_rate: f64) -> Self {
        self.tick_rate = tick_rate;
        self
    }

    pub fn frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    pub fn mouse(mut self, mouse: bool) -> Self {
        self.mouse = mouse;
        self
    }

    pub fn paste(mut self, paste: bool) -> Self {
        self.paste = paste;
        self
    }

    pub fn start(&mut self) {
        self.cancel(); // Cancel any existing task
        self.cancellation_token = CancellationToken::new();
        let event_loop = Self::event_loop(
            self.event_tx.clone(),
            self.cancellation_token.clone(),
            self.tick_rate,
            self.frame_rate,
        );
        self.task = tokio::spawn(async {
            event_loop.await;
        });
    }

    async fn event_loop(
        event_tx: UnboundedSender<Event>,
        cancellation_token: CancellationToken,
        tick_rate: f64,
        frame_rate: f64,
    ) {
        let mut event_stream = EventStream::new();
        let mut tick_interval = interval(Duration::from_secs_f64(1.0 / tick_rate));
        let mut render_interval = interval(Duration::from_secs_f64(1.0 / frame_rate));

        // if this fails, then it's likely a bug in the calling code
        event_tx
            .send(Event::Init)
            .expect("failed to send init event");
        loop {
            let event = tokio::select! {
                _ = cancellation_token.cancelled() => {
                    break;
                }
                _ = tick_interval.tick() => Event::Tick,
                _ = render_interval.tick() => Event::Render,
                crossterm_event = event_stream.next().fuse() => match crossterm_event {
                    Some(Ok(event)) => match event {
                        CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => Event::Key(key),
                        CrosstermEvent::Mouse(mouse) => Event::Mouse(mouse),
                        CrosstermEvent::Resize(x, y) => Event::Resize(x, y),
                        CrosstermEvent::FocusLost => Event::FocusLost,
                        CrosstermEvent::FocusGained => Event::FocusGained,
                        CrosstermEvent::Paste(s) => Event::Paste(s),
                        _ => continue, // ignore other events
                    }
                    Some(Err(_)) => Event::Error,
                    None => break, // the event stream has stopped and will not produce any more events
                },
            };
            if event_tx.send(event).is_err() {
                // the receiver has been dropped, so there's no point in continuing the loop
                break;
            }
        }
        cancellation_token.cancel();
    }

    pub fn stop(&self) -> Result<()> {
        self.cancel();
        let mut counter = 0;
        while !self.task.is_finished() {
            std::thread::sleep(Duration::from_millis(1));
            counter += 1;
            if counter > 50 {
                self.task.abort();
            }
            if counter > 100 {
                error!("Failed to abort task in 100 milliseconds for unknown reason");
                break;
            }
        }
        Ok(())
    }

    pub fn enter(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;
        if self.mouse {
            crossterm::execute!(stdout(), EnableMouseCapture)?;
        }
        if self.paste {
            crossterm::execute!(stdout(), EnableBracketedPaste)?;
        }
        self.start();
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        self.stop()?;
        if crossterm::terminal::is_raw_mode_enabled()? {
            self.flush()?;
            if self.paste {
                crossterm::execute!(stdout(), DisableBracketedPaste)?;
            }
            if self.mouse {
                crossterm::execute!(stdout(), DisableMouseCapture)?;
            }
            crossterm::execute!(stdout(), LeaveAlternateScreen, cursor::Show)?;
            crossterm::terminal::disable_raw_mode()?;
        }
        Ok(())
    }

    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    pub fn suspend(&mut self) -> Result<()> {
        self.exit()?;
        #[cfg(not(windows))]
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        self.enter()?;
        Ok(())
    }

    pub async fn next_event(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }
}

impl Deref for Tui {
    type Target = ratatui::Terminal<Backend<Stdout>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/ui.rs

```rs
pub(crate) mod components;
pub(crate) mod ids;
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/ui/components.rs

```rs
use std::sync::Arc;

use color_eyre::Result;
use crossterm::event::MouseEvent;
use ratatui::{
    Frame,
    layout::{Rect, Size},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, app::settings::SettingsStore, tui::Event};

mod aether_status_component;
mod logo;
mod status_bar;
mod task_list;

pub(crate) use aether_status_component::AetherStatusListComponent;
pub(crate) use logo::WizardLogoComponent;
pub(crate) use status_bar::StatusBar;
pub(crate) use task_list::TaskList;

/// `Component` is a trait that represents a visual and interactive element of the user interface.
///
/// Implementors of this trait can be registered with the main application loop and will be able to
/// receive events, update state, and be rendered on the screen.
pub trait Component {
    /// Called once when the component is created to provide initial terminal size.
    fn init(&mut self, area: Size) -> Result<()> {
        let _ = area; // to appease clippy
        Ok(())
    }

    /// Inform the component that its focus state changed (true = focused).
    /// Default: no-op. Override to store focus state for custom rendering.
    fn set_focused(&mut self, focused: bool) {
        let _ = focused; // default no-op
    }

    /// Route a high-level TUI event to this component.
    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let action = match event {
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_event(mouse_event)?,
            _ => None,
        };
        Ok(action)
    }

    /// Handle an action dispatched to this component. Return an Action to be re-dispatched if appropriate.
    fn handle_action(&mut self, action: &Action) -> Result<Option<Action>> {
        let _ = action; // to appease clippy
        Ok(None)
    }

    /// Handle a mouse event. Return an Action to be dispatched if appropriate.
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let _ = mouse; // to appease clippy
        Ok(None)
    }

    /// Update this component in response to a dispatched action.
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let _ = action; // to appease clippy
        Ok(None)
    }

    /// Draw this component within the provided area.
    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> Result<()>;
}
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/ui/ids.rs

```rs
// ui/ids.rs
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct PageId(pub u16);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct PopupId(pub u16);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct CompId(pub u32);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TaskId(pub u64);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct NotifId(pub u64);
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/ui/components/aether_status_component.rs

```rs
use crate::{
    action::{Action, UiAction},
    ui::components::Component,
};
use color_eyre::Result;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

#[derive(Clone, Copy, Debug)]
enum StatusItemKind {
    Settings,
    Certificate,
    Uds,
}

struct StatusSnapshot {
    settings_found: bool,
    settings_valid: bool,
    cert_found: bool,
    uds_found: bool,
}

impl StatusSnapshot {
    fn probe() -> Self {
        Self {
            settings_found: aether_config::settings_found(),
            settings_valid: aether_config::settings_valid(),
            cert_found: aether_config::certificate_found(),
            uds_found: aether_config::uds_found(),
        }
    }
}

pub(crate) struct AetherStatusListComponent {
    focused: bool,
    selected: usize,
    items: Vec<StatusItemKind>,
    snapshot: StatusSnapshot,
}

impl AetherStatusListComponent {
    pub(crate) fn new() -> Self {
        Self {
            focused: false,
            selected: 0,
            items: vec![
                StatusItemKind::Settings,
                StatusItemKind::Certificate,
                StatusItemKind::Uds,
            ],
            snapshot: StatusSnapshot::probe(),
        }
    }

    fn on_tick(&mut self) {
        // Re-probe; later we might debounce or compare.
        self.snapshot = StatusSnapshot::probe();
    }

    fn handle_nav(&mut self, dir: i32) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let cur = self.selected as i32;
        let next = (cur + dir).rem_euclid(len as i32);
        self.selected = next as usize;
    }

    fn popup_id(&self) -> &'static str {
        match self.items[self.selected] {
            StatusItemKind::Settings => "settings_popup",
            StatusItemKind::Certificate => "certificate_popup",
            StatusItemKind::Uds => "uds_popup",
        }
    }

    fn draw_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        for (idx, kind) in self.items.iter().enumerate() {
            match kind {
                StatusItemKind::Settings => {
                    let sel = idx == self.selected;
                    let (found, valid) =
                        (self.snapshot.settings_found, self.snapshot.settings_valid);

                    let marker: Vec<Span> = if found {
                        if valid {
                            vec![
                                " [  ".into(),
                                Span::styled("ok", Style::default().fg(Color::Green)),
                                "  ] ".into(),
                            ]
                        } else {
                            vec![
                                " [ ".into(),
                                Span::styled("failed", Style::default().fg(Color::Red)),
                                " ] ".into(),
                            ]
                        }
                    } else {
                        vec![
                            " [ ".into(),
                            Span::styled("      ", Style::default().fg(Color::Gray)),
                            " ] ".into(),
                        ]
                    };
                    let arrow_style = if sel {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    let text_style = if sel {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    let mut line_spans =
                        vec![Span::styled(if sel { "> " } else { "  " }, arrow_style)];
                    line_spans.extend(marker);
                    line_spans.push(Span::styled(" Aether Settings ", text_style));
                    lines.push(Line::from(line_spans));
                }
                StatusItemKind::Certificate => {
                    let sel = idx == self.selected;
                    let found = self.snapshot.cert_found;
                    let marker: Vec<Span> = if found {
                        vec![
                            " [  ".into(),
                            Span::styled("ok", Style::default().fg(Color::Green)),
                            "  ] ".into(),
                        ]
                    } else {
                        vec![
                            " [ ".into(),
                            Span::styled("    ", Style::default().fg(Color::Gray)),
                            " ] ".into(),
                        ]
                    };
                    let arrow_style = if sel {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    let text_style = if sel {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    let mut line_spans =
                        vec![Span::styled(if sel { "> " } else { "  " }, arrow_style)];
                    line_spans.extend(marker);
                    line_spans.push(Span::styled(" Certificate ", text_style));
                    lines.push(Line::from(line_spans));
                }
                StatusItemKind::Uds => {
                    let sel = idx == self.selected;
                    let found = self.snapshot.uds_found;
                    let marker: Vec<Span> = if found {
                        vec![
                            " [  ".into(),
                            Span::styled("ok", Style::default().fg(Color::Green)),
                            "  ] ".into(),
                        ]
                    } else {
                        vec![
                            " [ ".into(),
                            Span::styled("    ", Style::default().fg(Color::Gray)),
                            " ] ".into(),
                        ]
                    };
                    let arrow_style = if sel {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };
                    let text_style = if sel {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    let mut line_spans =
                        vec![Span::styled(if sel { "> " } else { "  " }, arrow_style)];
                    line_spans.extend(marker);
                    line_spans.push(Span::styled(" UDS Socket  ", text_style));
                    lines.push(Line::from(line_spans));
                }
            }
            // Blank spacer line after each group except last (for readability)
            if idx != self.items.len() - 1 {
                lines.push(Line::raw(""));
            }
        }
        lines
    }
}

impl Component for AetherStatusListComponent {
    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn handle_action(&mut self, action: &Action) -> Result<Option<Action>> {
        match action {
            Action::Ui(UiAction::NavigateUp) => {
                self.handle_nav(-1);
                Ok(None)
            }
            Action::Ui(UiAction::NavigateDown) => {
                self.handle_nav(1);
                Ok(None)
            }
            Action::Ui(UiAction::ActivateSelected) => {
                // Open popup for selected item
                Ok(Some(Action::Ui(UiAction::OpenPopup {
                    id: self.popup_id().to_string(),
                    priority: None,
                })))
            }
            _ => Ok(None),
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::Tick = action {
            self.on_tick();
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, body: Rect) -> Result<()> {
        let [area] = Layout::horizontal([Constraint::Length(28)])
            .flex(Flex::Center)
            .areas(body);
        let lines = self.draw_lines();
        let paragraph = Paragraph::new(lines).style(Style::default());
        f.render_widget(paragraph, area);
        Ok(())
    }
}
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/ui/components/logo.rs

```rs
use super::Component;
use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::{prelude::*, widgets::*};
use std::collections::HashMap;

#[derive(Default)]
pub struct LogoComponent {}
impl LogoComponent {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for LogoComponent {
    fn draw(&mut self, frame: &mut Frame, body: Rect) -> Result<()> {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Max(16), Constraint::Min(0)])
            .split(body);
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Max(71), Constraint::Min(0)])
            .split(vertical[1]);

        let logo_lines = vec![
            "███████████                                                     ██████",
            " ███      █                                                    ███  ███",
            " ███   █     ██████  ████████   ███████  ██████      ██████    ███     ",
            " ███████    ███  ███  ███  ███ ███  ███ ███  ███    ███  ███ ███████   ",
            " ███   █    ███  ███  ███      ███  ███ ███████     ███  ███   ███     ",
            " ███        ███  ███  ███      ███  ███ ███         ███  ███   ███     ",
            "█████        ██████  █████      ███████  ██████      ██████   █████    ",
            "                                    ███                                ",
            "                               ███  ███                                ",
            "    █████████    ███            ██████       ███                       ",
            "   ███     ███   ███                                                   ",
            "   ███         ███████    ██████  ████████  ████   ██████   █████      ",
            "    █████████    ███     ███  ███  ███  ███  ███  ███  ███ ███         ",
            "           ███   ███     ███  ███  ███       ███  ███████   █████      ",
            "   ███     ███   ███ ███ ███  ███  ███       ███  ███          ███     ",
            "    █████████     █████   ██████  █████     █████  ██████  ██████      ",
        ];

        let logo_color = vec![
            "AAAAAAAAAAA                                                     AAAAAA",
            " BBB      B                                                    BBB  BBB",
            " CCC   C     CCCCCC  CCCCCCCC   CCCCCCC  CCCCCC      CCCCCC    CCC     ",
            " DDDDDDD    DDD  DDD  DDD  DDD DDD  DDD DDD  DDD    DDD  DDD DDDDDDD   ",
            " EEE   E    EEE  EEE  EEE      EEE  EEE EEEEEEE     EEE  EEE   EEE     ",
            " FFF        FFF  FFF  FFF      FFF  FFF FFF         FFF  FFF   FFF     ",
            "GGGGG        GGGGGG  GGGGG      GGGGGGG  GGGGGG      GGGGGG   GGGGG    ",
            "                                    HHH                                ",
            "                               III  III                                ",
            "    JJJJJJJJJ    JJJ            JJJJJJ       JJJ                       ",
            "   KKK     KKK   KKK                                                   ",
            "   LLL         LLLLLLL    LLLLLL  LLLLLLLL  LLLL   LLLLLL   LLLLL      ",
            "    MMMMMMMMM    MMM     MMM  MMM  MMM  MMM  MMM  MMM  MMM MMM         ",
            "           NNN   NNN     NNN  NNN  NNN       NNN  NNNNNNN   NNNNN      ",
            "   OOO     OOO   OOO OOO OOO  OOO  OOO       OOO  OOO          OOO     ",
            "    PPPPPPPPP     PPPPP   PPPPPP  PPPPP     PPPPP  PPPPPP  PPPPPP      ",
        ];

        // let color_map: HashMap<char, Color> = [
        //     ('A', Color::Rgb(255, 246, 161)),
        //     ('B', Color::Rgb(255, 235, 151)),
        //     ('C', Color::Rgb(255, 225, 141)),
        //     ('D', Color::Rgb(255, 208, 127)),
        //     ('E', Color::Rgb(255, 201, 121)),
        //     ('F', Color::Rgb(255, 193, 113)),
        //     ('G', Color::Rgb(255, 185, 106)),
        //     ('H', Color::Rgb(255, 176, 98)),
        //     ('I', Color::Rgb(255, 164, 88)),
        //     ('J', Color::Rgb(255, 154, 79)),
        //     ('K', Color::Rgb(255, 145, 72)),
        //     ('L', Color::Rgb(255, 134, 62)),
        //     ('M', Color::Rgb(255, 119, 48)),
        //     ('N', Color::Rgb(255, 109, 39)),
        //     ('O', Color::Rgb(255, 99, 30)),
        //     ('P', Color::Rgb(255, 85, 18)),
        // ]
        // .iter()
        // .cloned()
        // .collect();

        let mut styled_lines = Vec::new();

        for (_, (logo_line, color_line)) in logo_lines.iter().zip(logo_color.iter()).enumerate() {
            let mut spans = Vec::new();
            let logo_chars: Vec<char> = logo_line.chars().collect();
            let color_chars: Vec<char> = color_line.chars().collect();

            for (j, &logo_char) in logo_chars.iter().enumerate() {
                let color = if j < color_chars.len() {
                    // color_map
                    //     .get(&color_chars[j])
                    //     .copied()
                    //     .unwrap_or(Color::White)
                    Color::DarkGray
                } else {
                    Color::White
                };

                spans.push(Span::styled(
                    logo_char.to_string(),
                    Style::default().fg(color),
                ));
            }

            styled_lines.push(Line::from(spans));
        }

        let logo = Paragraph::new(styled_lines)
            .block(Block::default())
            .wrap(ratatui::widgets::Wrap { trim: false });

        frame.render_widget(logo, horizontal[1]);
        Ok(())
    }
}

#[derive(Default)]
pub struct WizardLogoComponent {}
impl WizardLogoComponent {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Component for WizardLogoComponent {
    fn draw(&mut self, frame: &mut Frame, body: Rect) -> Result<()> {
        let [area] = Layout::horizontal([Constraint::Length(47)])
            .flex(Flex::Center)
            .areas(body);

        let logo_lines = vec![
            "                                               ",
            "██        ██ ██ ███████  █████  ██████  ██████ ",
            "██   ██   ██ ██      ██ ██   ██ ██   ██ ██   ██",
            " ██ ████ ██  ██   ███   ███████ ██████  ██   ██",
            " ████  ████  ██ ██      ██   ██ ██   ██ ██   ██",
            "  ██    ██   ██ ███████ ██   ██ ██   ██ ██████ ",
            "                                               ",
        ];

        let logo_color = vec![
            "                                               ",
            "AA        DD EE EFFFGGG  HHIII  JJKKKL  LMMMNN ",
            "AA   BC   DD EE      GG HH   IJ JJ   LL LM   NN",
            " AA BBCC DD  EE   FFG   HHHIIIJ JJKKKL  LM   NN",
            " AABB  CCDD  EE EF      HH   IJ JJ   LL LM   NN",
            "  AB    CD   EE EFFFGGG HH   IJ JJ   LL LMMMNN ",
            "                                               ",
        ];

        let color_map: HashMap<char, Color> = [
            ('A', Color::Rgb(91, 0, 130)),
            ('B', Color::Rgb(85, 1, 129)),
            ('C', Color::Rgb(68, 3, 127)),
            ('D', Color::Rgb(54, 4, 126)),
            ('E', Color::Rgb(38, 6, 124)),
            ('F', Color::Rgb(30, 7, 123)),
            ('G', Color::Rgb(20, 8, 122)),
            ('H', Color::Rgb(13, 9, 121)),
            ('I', Color::Rgb(8, 17, 129)),
            ('J', Color::Rgb(7, 38, 151)),
            ('K', Color::Rgb(5, 64, 178)),
            ('L', Color::Rgb(3, 89, 203)),
            ('M', Color::Rgb(1, 116, 230)),
            ('N', Color::Rgb(0, 140, 255)),
        ]
        .iter()
        .cloned()
        .collect();

        let mut styled_lines = Vec::new();

        for (_, (logo_line, color_line)) in logo_lines.iter().zip(logo_color.iter()).enumerate() {
            let mut spans = Vec::new();
            let logo_chars: Vec<char> = logo_line.chars().collect();
            let color_chars: Vec<char> = color_line.chars().collect();

            for (j, &logo_char) in logo_chars.iter().enumerate() {
                let color = if j < color_chars.len() {
                    color_map
                        .get(&color_chars[j])
                        .copied()
                        .unwrap_or(Color::White)
                } else {
                    Color::White
                };

                spans.push(Span::styled(
                    logo_char.to_string(),
                    Style::default().fg(color),
                ));
            }

            styled_lines.push(Line::from(spans));
        }
        let logo = Paragraph::new(styled_lines)
            .block(Block::default())
            .wrap(ratatui::widgets::Wrap { trim: false });
        // frame.render_widget(Block::new().style(Style::default().bg(Color::Green)), area);
        frame.render_widget(logo, area);
        Ok(())
    }
}
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/ui/components/status_bar.rs

```rs
use color_eyre::Result;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::{Action, AppAction, NotificationLevel, UiAction, UiMode},
    app::settings::SettingsStore,
    tui::Event,
    ui::components::Component,
};

/// StatusBar component
///
/// Renders a compact bottom bar with:
/// - Page name / context
/// - Focused component id
/// - UI mode (Normal/Edit)
/// - Key hints for the current context (e.g., F1 Help, Esc Normal)
/// - Notification counter
///
/// The component updates its state from incoming Actions:
/// - AppAction::SetKeymapContext sets the current keymap context for hint export
/// - AppAction::SetUiMode and UiAction::{EnterEditMode,ExitEditMode,ToggleEditMode}
/// - UiAction::ReportFocusedComponent updates the focused component id
/// - UiAction::ReportNotificationCount updates the visible notification count
/// - UiAction::ReportHelpVisible toggles whether help-specific hints are shown
pub struct StatusBar {
    tx: Option<UnboundedSender<Action>>,
    settings: Option<Arc<SettingsStore>>,

    // UX data
    page: String,
    context: String,
    focused: Option<String>,
    mode: UiMode,
    notif_count: u32,
    highest_severity: Option<NotificationLevel>,
    help_visible: bool,
}

impl StatusBar {
    /// Create a new status bar for the given page name.
    /// The initial keymap context defaults to "global" and should be updated via AppAction::SetKeymapContext.
    pub fn new(page: &str) -> Self {
        Self {
            tx: None,
            settings: None,
            page: page.to_string(),
            context: "global".to_string(),
            focused: None,
            mode: UiMode::Normal,
            notif_count: 0,
            highest_severity: None,
            help_visible: false,
        }
    }
    fn register_settings_handler(&mut self, settings: Arc<SettingsStore>) -> Result<()> {
        self.settings = Some(settings);
        Ok(())
    }

    /// Update the page label shown on the left side of the status bar.
    /// This is useful when the StatusBar is owned centrally by the App instead
    /// of being recreated per page.
    pub fn set_page<S: Into<String>>(&mut self, page: S) {
        self.page = page.into();
    }

    fn left_text(&self) -> Line<'static> {
        let page = Span::styled(
            format!(" {} ", self.page),
            Style::default().add_modifier(Modifier::BOLD),
        );
        let ctx = Span::raw(format!("({})  ", self.context));
        // Highlight focused component if present
        let focused_val = self.focused.as_deref().unwrap_or("-");
        let focused = if self.focused.is_some() {
            Span::styled(
                format!("Focus: {}  ", focused_val),
                Style::default().add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw(format!("Focus: {}  ", focused_val))
        };
        let mode_str = match self.mode {
            UiMode::Normal => "Normal",
            UiMode::Edit => "Edit",
        };
        let mode = Span::raw(format!("Mode: {}", mode_str));
        Line::from(vec![page, ctx, focused, mode])
    }

    fn right_text(&self) -> Line<'static> {
        // Key hints derived from settings keymap (if available)
        let hints = if let Some(settings) = &self.settings {
            let map = settings.export_keymap_for(settings::DeviceFilter::Keyboard, &self.context);
            let mut parts: Vec<String> = Vec::new();

            if self.help_visible {
                if let Some(k) = first_key(&map, "HelpToggleGlobal") {
                    parts.push(format!("{} Global", k));
                }
                if let Some(k) = first_key(&map, "HelpToggleWrap") {
                    let wrap_on = if let Some(store) = &self.settings {
                        store
                            .get::<crate::app::settings::Wizard>()
                            .map(|w| w.help_wrap_on)
                            .unwrap_or(true)
                    } else {
                        true
                    };
                    let wrap_label = if wrap_on { "Wrap On" } else { "Wrap Off" };
                    parts.push(format!("{} {}", k, wrap_label));
                }
                if let Some(k) = first_key(&map, "HelpSearch") {
                    parts.push(format!("{} Search", k));
                }
                if let Some(k) = first_key(&map, "HelpPageUp") {
                    parts.push(format!("{} PgUp", k));
                }
                if let Some(k) = first_key(&map, "HelpPageDown") {
                    parts.push(format!("{} PgDown", k));
                }
                if let Some(k) = first_key(&map, "HelpScrollUp") {
                    parts.push(format!("{} Up", k));
                }
                if let Some(k) = first_key(&map, "HelpScrollDown") {
                    parts.push(format!("{} Down", k));
                }
            } else {
                if let Some(k) = first_key(&map, "Help") {
                    parts.push(format!("{} Help", k));
                }
                if let Some(k) = first_key(&map, "ModeNormal") {
                    parts.push(format!("{} Normal", k));
                }
                if let Some(k) = first_key(&map, "ModeInsert") {
                    parts.push(format!("{} Edit", k));
                }
                if let Some(k) = first_key(&map, "OpenPopup") {
                    parts.push(format!("{} Popup", k));
                }
            }

            if parts.is_empty() {
                "Keys: n/a".to_string()
            } else {
                parts.join(" · ")
            }
        } else {
            "Keys: n/a".to_string()
        };

        let mut spans: Vec<Span<'static>> = vec![Span::raw(hints)];
        if self.notif_count > 0 {
            let color = match self.highest_severity {
                Some(NotificationLevel::Error) => Color::Red,
                Some(NotificationLevel::Warning) => Color::Yellow,
                Some(NotificationLevel::Success) => Color::Green,
                Some(NotificationLevel::Info) => Color::Blue,
                None => Color::White,
            };
            let bell = Span::styled(
                format!("  🔔 {}", self.notif_count),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            );
            spans.push(bell);
        }

        Line::from(spans)
    }
}

impl Component for StatusBar {
    fn handle_events(&mut self, _event: Option<Event>) -> Result<Option<Action>> {
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Ui(UiAction::ReportFocusedComponent(id)) => {
                self.focused = Some(id);
            }
            Action::Ui(UiAction::EnterEditMode) => self.mode = UiMode::Edit,
            Action::Ui(UiAction::ExitEditMode) => self.mode = UiMode::Normal,
            Action::Ui(UiAction::ToggleEditMode) => {
                self.mode = match self.mode {
                    UiMode::Normal => UiMode::Edit,
                    UiMode::Edit => UiMode::Normal,
                };
            }
            Action::Ui(UiAction::ReportNotificationCount(n)) => {
                self.notif_count = n;
            }
            Action::Ui(UiAction::ReportNotificationSeverity(sev)) => {
                self.highest_severity = sev;
            }
            Action::Ui(UiAction::ReportHelpVisible(v)) => {
                self.help_visible = v;
            }
            Action::App(AppAction::SetUiMode(m)) => self.mode = m,
            Action::App(AppAction::SetKeymapContext { name }) => self.context = name,
            _ => {}
        }
        Ok(None)
    }

    fn set_focused(&mut self, _focused: bool) {
        // StatusBar is not a focus target; ignore focus state.
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        // Split horizontally into left (info) and right (hints+notif)
        let right_w = area.width.min(50);
        let left_w = area.width.saturating_sub(right_w);

        let left_area = Rect::new(area.x, area.y, left_w, area.height);
        let right_area = Rect::new(area.x + left_w, area.y, right_w, area.height);

        let left = Paragraph::new(self.left_text());
        let right = Paragraph::new(self.right_text());

        f.render_widget(left, left_area);
        f.render_widget(right, right_area);

        Ok(())
    }
}

// Helpers

fn first_key(
    map: &std::collections::BTreeMap<String, Vec<String>>,
    action: &str,
) -> Option<String> {
    // Try exact key, then case-insensitive
    if let Some(v) = map.get(action) {
        return v.get(0).cloned().map(prettify_chord);
    }
    if let Some((_, v)) = map
        .iter()
        .find(|(k, _)| k.to_ascii_lowercase() == action.to_ascii_lowercase())
    {
        return v.get(0).cloned().map(prettify_chord);
    }
    None
}

fn prettify_chord(s: String) -> String {
    // Make it look nicer: ctrl+p -> Ctrl+P, f1 -> F1, esc -> Esc
    let mut out = String::new();
    for (i, part) in s.split('+').enumerate() {
        if i > 0 {
            out.push('+');
        }
        out.push_str(&capitalize_key(part));
    }
    out
}

fn capitalize_key(k: &str) -> String {
    let lower = k.to_ascii_lowercase();
    match lower.as_str() {
        "ctrl" => "Ctrl".to_string(),
        "alt" => "Alt".to_string(),
        "shift" => "Shift".to_string(),
        "meta" => "Meta".to_string(),
        "esc" => "Esc".to_string(),
        "enter" => "Enter".to_string(),
        "tab" => "Tab".to_string(),
        s if s.starts_with('f') && s[1..].chars().all(|c| c.is_ascii_digit()) => {
            s.to_ascii_uppercase()
        }
        s if s.len() == 1 => s.to_ascii_uppercase(),
        _ => k.to_string(),
    }
}
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/ui/components/task_list.rs

```rs
use crate::{
    action::{
        Action, LogicAction, Notification, NotificationLevel, TaskId, TaskProgress, TaskResult,
        UiAction,
    },
    ui::components::Component,
};
use color_eyre::Result;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{Duration, sleep};

/// Task manager + list: tracks task progress and emits notifications on completion.
pub(crate) struct TaskList {
    tx: Option<UnboundedSender<Action>>,
    tasks: HashMap<TaskId, TaskInfo>,
    focused: bool,
}

#[derive(Clone, Default)]
pub(crate) struct TaskInfo {
    label: String,
    progress: Option<f32>,
    message: Option<String>,
    success: Option<bool>,
}

impl TaskList {
    pub(crate) fn new() -> Self {
        Self {
            tx: None,
            tasks: HashMap::new(),
            focused: false,
        }
    }

    /// Example: spawn a demo background task (scaffolding).
    /// Emits TaskStarted, periodic TaskProgress, and TaskCompleted via action channel.
    #[allow(dead_code)]
    pub(crate) fn spawn_demo_task(&self, id: TaskId, label: String) {
        if let Some(tx) = &self.tx {
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(Action::Logic(LogicAction::TaskStarted { id: id.clone() }));
                for i in 1..=10 {
                    sleep(Duration::from_millis(200)).await;
                    let _ = tx.send(Action::Logic(LogicAction::TaskProgress(TaskProgress {
                        id: id.clone(),
                        fraction: Some(i as f32 / 10.0),
                        message: Some(format!("{} — step {}/10", label, i)),
                    })));
                }
                let _ = tx.send(Action::Logic(LogicAction::TaskCompleted(TaskResult {
                    id: id.clone(),
                    success: true,
                    result_json: None,
                    message: Some(format!("{} — done", label)),
                })));
            });
        }
    }
}

impl Component for TaskList {
    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Logic(LogicAction::TaskStarted { id }) => {
                // Ensure a task entry exists
                self.tasks.entry(id).or_insert_with(|| TaskInfo {
                    label: "Task".to_string(),
                    ..Default::default()
                });
            }
            Action::Logic(LogicAction::TaskProgress(TaskProgress {
                id,
                fraction,
                message,
            })) => {
                let entry = self.tasks.entry(id).or_insert_with(Default::default);
                entry.progress = fraction;
                if let Some(m) = message {
                    entry.message = Some(m);
                }
            }
            Action::Logic(LogicAction::TaskCompleted(TaskResult {
                id,
                success,
                result_json: _,
                message,
            })) => {
                let entry = self
                    .tasks
                    .entry(id.clone())
                    .or_insert_with(Default::default);
                entry.success = Some(success);
                entry.message = message.clone();

                // Emit a notification on completion
                if let Some(tx) = &self.tx {
                    let level = if success {
                        NotificationLevel::Success
                    } else {
                        NotificationLevel::Error
                    };
                    let msg = message.unwrap_or_else(|| "Task completed".to_string());
                    let _ = tx.send(Action::Ui(UiAction::ShowNotification(Notification {
                        id: format!("task-{}", id),
                        level,
                        message: msg,
                        timeout_ms: None, // App will apply default lifetime
                    })));
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        // Render a simple list of tasks with progress and status
        let mut lines: Vec<String> = Vec::new();
        for (id, t) in self.tasks.iter() {
            let pct = t
                .progress
                .map(|p| format!("{:>3}%", (p * 100.0) as u32))
                .unwrap_or("--%".to_string());
            let status = match t.success {
                Some(true) => "OK",
                Some(false) => "ERR",
                None => "RUN",
            };
            let msg = t.message.as_deref().unwrap_or(if t.success.is_some() {
                ""
            } else {
                "working..."
            });
            lines.push(format!("[{}] {} {} {}", status, pct, id, msg));
        }
        if lines.is_empty() {
            lines.push("No running tasks".to_string());
        }
        let mut block = Block::default().borders(Borders::ALL).title("Tasks");
        if self.focused {
            block = block.style(Style::default().fg(Color::Yellow));
        }
        let para = Paragraph::new(lines.join("\n")).block(block);
        f.render_widget(para, area);
        Ok(())
    }
}
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/app/keymap_registry.rs

```rs
use crate::action::{Action, AppAction, LogicAction, NotificationLevel, UiAction};
use settings::keymap::ActionRegistry;

pub struct WizardActionRegistry {
    action_names: Vec<String>,
}

impl WizardActionRegistry {
    pub fn new() -> Self {
        Self {
            action_names: vec![
                // Legacy actions
                "Quit".to_string(),
                "Help".to_string(),
                "Tick".to_string(),
                "Render".to_string(),
                "Resize".to_string(),
                "ClearScreen".to_string(),
                // UI Actions
                "FocusNext".to_string(),
                "FocusPrev".to_string(),
                "FocusById".to_string(),
                "NavigateUp".to_string(),
                "NavigateDown".to_string(),
                "NavigateLeft".to_string(),
                "NavigateRight".to_string(),
                "ActivateSelected".to_string(),
                "ToggleEditMode".to_string(),
                "EnterEditMode".to_string(),
                "ExitEditMode".to_string(),
                "OpenPopup".to_string(),
                "ClosePopup".to_string(),
                "CloseTopPopup".to_string(),
                "CloseAllPopups".to_string(),
                "ShowNotification".to_string(),
                "DismissNotification".to_string(),
                "HelpToggleGlobal".to_string(),
                "BeginHelpSearch".to_string(),
                "HelpSearchClear".to_string(),
                "NextPage".to_string(),
                "PrevPage".to_string(),
                // App Actions
                "SetActivePage".to_string(),
                "SetKeymapContext".to_string(),
                "SaveSettings".to_string(),
                "LoadSettings".to_string(),
                // Logic Actions
                "LoadConfig".to_string(),
                "SaveConfig".to_string(),
            ],
        }
    }
}

impl ActionRegistry for WizardActionRegistry {
    type Action = Action;

    fn resolve_action(
        &self,
        action_name: &str,
        action_data: Option<&toml::Value>,
    ) -> Option<Self::Action> {
        match action_name {
            // Legacy actions
            "Quit" => Some(Action::Quit),
            "Help" => Some(Action::Help),
            "Tick" => Some(Action::Tick),
            "Render" => Some(Action::Render),
            "ClearScreen" => Some(Action::ClearScreen),

            // UI Actions - Focus
            "FocusNext" => Some(Action::Ui(UiAction::FocusNext)),
            "FocusPrev" => Some(Action::Ui(UiAction::FocusPrev)),
            "FocusById" => action_data
                .and_then(|data| data.get("id"))
                .and_then(|id| id.as_str())
                .map(|id| Action::Ui(UiAction::FocusById(id.to_string()))),

            // UI Actions - Navigation
            "NavigateUp" => Some(Action::Ui(UiAction::NavigateUp)),
            "NavigateDown" => Some(Action::Ui(UiAction::NavigateDown)),
            "NavigateLeft" => Some(Action::Ui(UiAction::NavigateLeft)),
            "NavigateRight" => Some(Action::Ui(UiAction::NavigateRight)),
            "ActivateSelected" => Some(Action::Ui(UiAction::ActivateSelected)),

            // UI Actions - Edit Mode
            "ToggleEditMode" => Some(Action::Ui(UiAction::ToggleEditMode)),
            "EnterEditMode" => Some(Action::Ui(UiAction::EnterEditMode)),
            "ExitEditMode" => Some(Action::Ui(UiAction::ExitEditMode)),

            // UI Actions - Popups
            "OpenPopup" => action_data
                .and_then(|data| data.get("id"))
                .and_then(|id| id.as_str())
                .map(|id| {
                    let priority = action_data
                        .and_then(|data| data.get("priority"))
                        .and_then(|p| p.as_integer())
                        .map(|p| p as i32);
                    Action::Ui(UiAction::OpenPopup {
                        id: id.to_string(),
                        priority,
                    })
                }),
            "ClosePopup" => action_data
                .and_then(|data| data.get("id"))
                .and_then(|id| id.as_str())
                .map(|id| Action::Ui(UiAction::ClosePopup { id: id.to_string() })),
            "CloseTopPopup" => Some(Action::Ui(UiAction::CloseTopPopup)),
            "CloseAllPopups" => Some(Action::Ui(UiAction::CloseAllPopups)),

            // UI Actions - Notifications
            "ShowNotification" => {
                if let Some(data) = action_data {
                    let id = data.get("id")?.as_str()?.to_string();
                    let message = data.get("message")?.as_str()?.to_string();
                    let level = data
                        .get("level")
                        .and_then(|l| l.as_str())
                        .and_then(|l| match l {
                            "info" => Some(NotificationLevel::Info),
                            "success" => Some(NotificationLevel::Success),
                            "warning" => Some(NotificationLevel::Warning),
                            "error" => Some(NotificationLevel::Error),
                            _ => None,
                        })
                        .unwrap_or(NotificationLevel::Info);
                    let timeout_ms = data
                        .get("timeout_ms")
                        .and_then(|t| t.as_integer())
                        .map(|t| t as u64);

                    Some(Action::Ui(UiAction::ShowNotification(
                        crate::action::Notification {
                            id,
                            level,
                            message,
                            timeout_ms,
                        },
                    )))
                } else {
                    None
                }
            }
            "DismissNotification" => action_data
                .and_then(|data| data.get("id"))
                .and_then(|id| id.as_str())
                .map(|id| Action::Ui(UiAction::DismissNotification { id: id.to_string() })),

            // UI Actions - Help
            "HelpToggleGlobal" => Some(Action::Ui(UiAction::HelpToggleGlobal)),
            "BeginHelpSearch" => Some(Action::Ui(UiAction::BeginHelpSearch)),
            "HelpSearchClear" => Some(Action::Ui(UiAction::HelpSearchClear)),

            // UI Actions - Page Navigation
            "NextPage" => Some(Action::Ui(UiAction::NextPage)),
            "PrevPage" => Some(Action::Ui(UiAction::PrevPage)),

            // App Actions
            "SetActivePage" => action_data
                .and_then(|data| data.get("id"))
                .and_then(|id| id.as_str())
                .map(|id| Action::App(AppAction::SetActivePage { id: id.to_string() })),
            "SetKeymapContext" => action_data
                .and_then(|data| data.get("name"))
                .and_then(|name| name.as_str())
                .map(|name| {
                    Action::App(AppAction::SetKeymapContext {
                        name: name.to_string(),
                    })
                }),
            "SaveSettings" => Some(Action::App(AppAction::SaveSettings)),
            "LoadSettings" => Some(Action::App(AppAction::LoadSettings)),

            // Logic Actions
            "LoadConfig" => Some(Action::Logic(LogicAction::LoadConfig)),
            "SaveConfig" => Some(Action::Logic(LogicAction::SaveConfig)),

            _ => None,
        }
    }

    fn get_action_names(&self) -> Vec<String> {
        self.action_names.clone()
    }
}
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/app/settings.rs

```rs
use serde::{Deserialize, Serialize};

pub(crate) use settings::{ActionRegistry, Settings, SettingsStore};

// pub enum WizardSettingField {
//     MetaVersion,
//     WizardTickRate,
//     WizardFps,
//     WizardHelpShowGlobal,
//     WizardHelpWrapOn,
// }

// fn default_value_for(field: WizardSettingField) -> Option<toml::Value> {
//     let txt = settings::default_settings_wizard();
//     let Ok(root) = toml::from_str::<toml::Value>(txt.as_ref()) else {
//         return None;
//     };
//     let tbl = root.as_table()?;

//     let path = match field {
//         WizardSettingField::MetaVersion => &["meta", "version"][..],
//         WizardSettingField::WizardTickRate => &["wizard", "tick_rate"][..],
//         WizardSettingField::WizardFps => &["wizard", "fps"][..],
//         WizardSettingField::WizardHelpShowGlobal => &["wizard", "help_show_global"][..],
//         WizardSettingField::WizardHelpWrapOn => &["wizard", "help_wrap_on"][..],
//     };

//     let mut cur = toml::Value::Table(tbl.clone());
//     for seg in path {
//         match cur {
//             toml::Value::Table(ref t) => {
//                 let Some(v) = t.get(*seg) else {
//                     return None;
//                 };
//                 cur = v.clone();
//             }
//             _ => return None,
//         }
//     }
//     Some(cur)
// }

/// Wendet eine einzelne Feldänderung an. Leerer String => Feld wird auf den eingebetteten Default zurückgesetzt (falls vorhanden).
// pub fn apply_wizard_setting(
//     store: &settings::SettingsStore,
//     field: WizardSettingField,
//     raw_value: &str,
// ) -> color_eyre::Result<()> {
//     let s = raw_value.trim();

//     match field {
//         WizardSettingField::MetaVersion => {
//             let v: String = if s.is_empty() {
//                 default_value_for(field)
//                     .and_then(|v| v.as_str().map(|n| n.to_string()))
//                     .unwrap_or_default()
//             } else {
//                 s.parse()?
//             };
//             store.update::<Meta>(|m| m.version = v)?;
//         }
//         WizardSettingField::WizardTickRate => {
//             let v: f64 = if s.is_empty() {
//                 default_value_for(field)
//                     .and_then(|v| v.as_float().map(|n| n as f64))
//                     .unwrap_or_default()
//             } else {
//                 s.parse()?
//             };
//             store.update::<Wizard>(|w| w.tick_rate = v)?;
//         }
//         WizardSettingField::WizardFps => {
//             let v: f64 = if s.is_empty() {
//                 default_value_for(field)
//                     .and_then(|v| v.as_float().map(|n| n as f64))
//                     .unwrap_or_default()
//             } else {
//                 s.parse()?
//             };
//             store.update::<Wizard>(|w| w.fps = v)?;
//         }
//         WizardSettingField::WizardHelpShowGlobal => {
//             let v: bool = if s.is_empty() {
//                 default_value_for(field)
//                     .and_then(|v| v.as_bool())
//                     .unwrap_or(true)
//             } else {
//                 parse_bool(s)?
//             };
//             store.update::<Wizard>(|w| w.help_show_global = v)?;
//         }
//         WizardSettingField::WizardHelpWrapOn => {
//             let v: bool = if s.is_empty() {
//                 default_value_for(field)
//                     .and_then(|v| v.as_bool())
//                     .unwrap_or(true)
//             } else {
//                 parse_bool(s)?
//             };
//             store.update::<Wizard>(|w| w.help_wrap_on = v)?;
//         }
//     }

//     Ok(())
// }

// 1) Typisierte Modelle
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct MetaCfg {
    pub version: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct WizardCfg {
    pub tick_rate: f64,
    pub fps: f64,
    /// Whether Help should include global key bindings by default.
    pub help_show_global: bool,
    /// Whether Help content wraps long lines by default.
    pub help_wrap_on: bool,
}

// 2) SECTION-Bindings
pub struct Meta;
impl Settings for Meta {
    const SECTION: &'static str = "meta";
    type Model = MetaCfg;
}

pub struct Wizard;
impl Settings for Wizard {
    const SECTION: &'static str = "wizard";
    type Model = WizardCfg;
}

// 3) Zentraler Builder: überall gleich aufrufbar (Wizard & Runtime)
pub fn build_wizard_settings_store() -> color_eyre::Result<SettingsStore> {
    let builder = SettingsStore::builder()
        .with_embedded_setting_asset("settings/wizard-default.toml")
        .with_settings_file_optional(paths::config_dir().join("wizard.toml"))
        .with_embedded_keymap_asset("keymaps/wizard-default.toml");
    // .with_keymap_file_optional(paths::config_dir().join("wizard-keymap.toml"));

    let store = builder.build()?;
    store.register::<Meta>()?;
    store.register::<Wizard>()?;

    Ok(store)
}
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/app/task_manager.rs

```rs
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

/// Background task state tracked by the application.
#[derive(Debug, Clone)]
pub struct TaskState {
    pub id: TaskId,
    pub spec: TaskSpec,
    pub started: bool,
    pub finished: bool,
    pub cancelled: bool,
    pub progress: Option<f32>,
    pub message: Option<String>,
    pub success: Option<bool>,
    pub result_json: Option<String>,
}

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
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/layers/help.rs

```rs
// //! Help popup that displays keybinding information
// //!
// //! Shows a searchable table of available keybindings with context information.

// use color_eyre::Result;
// use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
// use ratatui::{
//     Frame,
//     layout::{Constraint, Layout, Rect},
//     style::{Color, Modifier, Style},
//     symbols,
//     widgets::{Block, BorderType, Borders, Clear, Paragraph, Row, Table, TableState},
// };
// use std::sync::Arc;
// use tokio::sync::mpsc::UnboundedSender;
// use tui_input::Input;

// use crate::{
//     action::{Action, UiAction},
//     app::settings::SettingsStore,
//     popup::{Popup, PopupConfig, PopupPosition, PopupSize},
// };

// /// Help popup displaying keybinding information
// #[derive(Clone)]
// pub struct HelpPopup {
//     action_tx: Option<UnboundedSender<Action>>,
//     settings: Option<Arc<SettingsStore>>,
//     keymap_context: String,

//     // Search functionality
//     help_search: Option<String>,
//     help_input: Input,
//     help_prompt_active: bool,

//     // Table state
//     help_table_state: TableState,
//     help_table_len: usize,
// }

// impl Default for HelpPopup {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl HelpPopup {
//     pub fn new() -> Self {
//         Self {
//             action_tx: None,
//             settings: None,
//             keymap_context: "global".to_string(),
//             help_search: None,
//             help_input: Input::default(),
//             help_prompt_active: false,
//             help_table_state: TableState::default(),
//             help_table_len: 0,
//         }
//     }

//     /// Set the settings store for this help popup
//     pub fn set_settings(&mut self, settings: Arc<SettingsStore>) {
//         self.settings = Some(settings);
//     }

//     /// Set the current keymap context
//     pub fn set_keymap_context(&mut self, context: String) {
//         self.keymap_context = context;
//     }

//     /// Toggle search mode
//     fn toggle_search(&mut self) -> Result<Option<Action>> {
//         if self.help_prompt_active {
//             // Exit search mode and apply filter
//             self.help_prompt_active = false;
//             let input_value = self.help_input.value().trim();
//             if input_value.is_empty() {
//                 self.help_search = None;
//             } else {
//                 self.help_search = Some(input_value.to_string());
//             }
//             // Reset selection when search changes
//             self.help_table_state.select(Some(0));
//         } else {
//             // Enter search mode
//             self.help_prompt_active = true;
//             if let Some(current) = &self.help_search {
//                 self.help_input = Input::new(current.clone());
//             } else {
//                 self.help_input = Input::default();
//             }
//         }
//         Ok(None)
//     }

//     /// Clear the current search
//     fn clear_search(&mut self) -> Result<Option<Action>> {
//         self.help_search = None;
//         self.help_input = Input::default();
//         self.help_prompt_active = false;
//         self.help_table_state.select(Some(0));
//         Ok(None)
//     }

//     /// Navigate the help table
//     fn navigate_table(&mut self, direction: TableNavigation) -> Result<Option<Action>> {
//         if self.help_table_len == 0 {
//             return Ok(None);
//         }

//         let new_index = match direction {
//             TableNavigation::Up => {
//                 match self.help_table_state.selected() {
//                     Some(i) if i > 0 => Some(i - 1),
//                     Some(_) => Some(self.help_table_len - 1), // Wrap to bottom
//                     None => Some(0),
//                 }
//             }
//             TableNavigation::Down => {
//                 match self.help_table_state.selected() {
//                     Some(i) if i + 1 < self.help_table_len => Some(i + 1),
//                     Some(_) => Some(0), // Wrap to top
//                     None => Some(0),
//                 }
//             }
//             TableNavigation::PageUp => match self.help_table_state.selected() {
//                 Some(i) => Some(i.saturating_sub(10)),
//                 None => Some(0),
//             },
//             TableNavigation::PageDown => match self.help_table_state.selected() {
//                 Some(i) => Some((i + 10).min(self.help_table_len - 1)),
//                 None => Some(0),
//             },
//             TableNavigation::Home => Some(0),
//             TableNavigation::End => Some(self.help_table_len.saturating_sub(1)),
//         };

//         if let Some(index) = new_index {
//             self.help_table_state.select(Some(index));
//         }

//         Ok(None)
//     }

//     /// Build the keymap table data from settings
//     fn build_keymap_table_data(
//         &self,
//         active_contexts: &[String],
//     ) -> Vec<(String, String, String, String)> {
//         use settings::DeviceFilter;

//         let mut table_data = Vec::new();
//         let filter = self.help_search.as_ref().map(|s| s.to_ascii_lowercase());

//         if let Some(settings) = &self.settings {
//             let mut seen_actions = std::collections::HashSet::new();

//             // Iterate through ALL active contexts
//             for context_name in active_contexts {
//                 let ctx_map = settings.export_keymap_for(DeviceFilter::Keyboard, context_name);

//                 // Add context-specific bindings
//                 for (action_name, chords) in ctx_map.iter() {
//                     let keys_str = chords.join(", ");

//                     // Generate a simple detail description
//                     let details = match action_name.as_str() {
//                         "Help" => "Show/hide this help dialog".to_string(),
//                         "Quit" => "Exit the application".to_string(),
//                         "FocusNext" | "NextField" => "Move focus to next element".to_string(),
//                         "FocusPrev" | "PrevField" => "Move focus to previous element".to_string(),
//                         "NavigateUp" => "Navigate up".to_string(),
//                         "NavigateDown" => "Navigate down".to_string(),
//                         "NavigateLeft" => "Navigate left".to_string(),
//                         "NavigateRight" => "Navigate right".to_string(),
//                         "ActivateSelected" => "Activate selected item".to_string(),
//                         "ToggleEditMode" | "ModeCycle" => "Toggle edit mode".to_string(),
//                         "NextPage" => "Go to next page".to_string(),
//                         "PrevPage" | "PreviousPage" => "Go to previous page".to_string(),
//                         _ => format!("Execute {}", action_name.replace("_", " ").to_lowercase()),
//                     };

//                     // Apply search filter
//                     let include = match &filter {
//                         Some(query) => {
//                             let action_lower = action_name.to_ascii_lowercase();
//                             let keys_lower = keys_str.to_ascii_lowercase();
//                             let context_lower = context_name.to_ascii_lowercase();
//                             let details_lower = details.to_ascii_lowercase();

//                             action_lower.contains(query)
//                                 || keys_lower.contains(query)
//                                 || context_lower.contains(query)
//                                 || details_lower.contains(query)
//                         }
//                         None => true,
//                     };

//                     if include {
//                         // Use combination of action + context to avoid duplicates
//                         let key_id = (action_name.clone(), context_name.clone());
//                         if seen_actions.insert(key_id) {
//                             table_data.push((
//                                 action_name.clone(),
//                                 keys_str,
//                                 context_name.clone(),
//                                 details,
//                             ));
//                         }
//                     }
//                 }
//             }
//         } else {
//             // Fallback content when no settings available
//             let fallback_data = vec![
//                 (
//                     "Help".to_string(),
//                     "ctrl+h".to_string(),
//                     "global".to_string(),
//                     "Show this help dialog".to_string(),
//                 ),
//                 (
//                     "Quit".to_string(),
//                     "ctrl+c".to_string(),
//                     "global".to_string(),
//                     "Exit the application".to_string(),
//                 ),
//             ];

//             for (action, keys, context, details) in fallback_data {
//                 let include = match &filter {
//                     Some(query) => {
//                         let action_lower = action.to_ascii_lowercase();
//                         let keys_lower = keys.to_ascii_lowercase();
//                         let context_lower = context.to_ascii_lowercase();
//                         let details_lower = details.to_ascii_lowercase();

//                         action_lower.contains(query)
//                             || keys_lower.contains(query)
//                             || context_lower.contains(query)
//                             || details_lower.contains(query)
//                     }
//                     None => true,
//                 };

//                 if include {
//                     table_data.push((action, keys, context, details));
//                 }
//             }
//         }

//         // Sort by context first, then by action name for better organization
//         table_data.sort_by(|a, b| match a.2.cmp(&b.2) {
//             std::cmp::Ordering::Equal => a.0.cmp(&b.0),
//             other => other,
//         });

//         table_data
//     }
// }

// /// Navigation directions for the help table
// enum TableNavigation {
//     Up,
//     Down,
//     PageUp,
//     PageDown,
//     Home,
//     End,
// }

// impl Popup for HelpPopup {
//     fn as_any(&self) -> &dyn std::any::Any {
//         self
//     }

//     fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
//         self
//     }
//     fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
//         self.action_tx = Some(tx);
//         Ok(())
//     }

//     fn init(&mut self) -> Result<()> {
//         // Initialize table selection
//         self.help_table_state.select(Some(0));
//         Ok(())
//     }

//     fn keymap_context(&self) -> &'static str {
//         "help"
//     }

//     fn id(&self) -> &'static str {
//         "help"
//     }

//     fn config(&self) -> PopupConfig {
//         PopupConfig {
//             size: PopupSize::Percentage {
//                 width: 85,
//                 height: 85,
//             },
//             position: PopupPosition::Center,
//             modal: true,
//             closable: true,
//             resizable: false,
//         }
//     }

//     fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
//         // Handle different key events based on current mode
//         if self.help_prompt_active {
//             // Search mode - handle input
//             match key.code {
//                 KeyCode::Char(c) => {
//                     self.help_input
//                         .handle(tui_input::InputRequest::InsertChar(c));
//                 }
//                 KeyCode::Backspace => {
//                     self.help_input
//                         .handle(tui_input::InputRequest::DeletePrevChar);
//                 }
//                 KeyCode::Delete => {
//                     self.help_input
//                         .handle(tui_input::InputRequest::DeleteNextChar);
//                 }
//                 KeyCode::Left => {
//                     self.help_input
//                         .handle(tui_input::InputRequest::GoToPrevChar);
//                 }
//                 KeyCode::Right => {
//                     self.help_input
//                         .handle(tui_input::InputRequest::GoToNextChar);
//                 }
//                 KeyCode::Home => {
//                     self.help_input.handle(tui_input::InputRequest::GoToStart);
//                 }
//                 KeyCode::End => {
//                     self.help_input.handle(tui_input::InputRequest::GoToEnd);
//                 }
//                 KeyCode::Enter => {
//                     return self.toggle_search();
//                 }
//                 KeyCode::Esc => {
//                     self.help_prompt_active = false;
//                     return Ok(None);
//                 }
//                 _ => {}
//             }
//         } else {
//             // Normal mode - handle navigation and commands
//             match (key.code, key.modifiers) {
//                 (KeyCode::Esc, _) => {
//                     return Ok(Some(Action::Ui(UiAction::ClosePopup {
//                         id: "help".to_string(),
//                     })));
//                 }
//                 (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
//                     return self.toggle_search();
//                 }
//                 (KeyCode::Char('/'), _) => {
//                     return self.toggle_search();
//                 }
//                 (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
//                     return self.clear_search();
//                 }
//                 (KeyCode::Up, _) => {
//                     return self.navigate_table(TableNavigation::Up);
//                 }
//                 (KeyCode::Down, _) => {
//                     return self.navigate_table(TableNavigation::Down);
//                 }
//                 (KeyCode::PageUp, _) => {
//                     return self.navigate_table(TableNavigation::PageUp);
//                 }
//                 (KeyCode::PageDown, _) => {
//                     return self.navigate_table(TableNavigation::PageDown);
//                 }
//                 (KeyCode::Home, _) => {
//                     return self.navigate_table(TableNavigation::Home);
//                 }
//                 (KeyCode::End, _) => {
//                     return self.navigate_table(TableNavigation::End);
//                 }
//                 _ => {}
//             }
//         }

//         Ok(None)
//     }

//     fn update(&mut self, action: Action) -> Result<Option<Action>> {
//         match action {
//             Action::Ui(UiAction::ComponentCommand { command, data: _ }) => match command.as_str() {
//                 "toggle_search" => return self.toggle_search(),
//                 "clear_search" => return self.clear_search(),
//                 _ => {}
//             },
//             _ => {}
//         }
//         Ok(None)
//     }

//     fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
//         // Clear the area first for modal effect
//         f.render_widget(Clear, area);

//         let collapsed_border_set = symbols::border::Set {
//             top_left: symbols::line::NORMAL.vertical_right,
//             top_right: symbols::line::NORMAL.vertical_left,
//             bottom_left: symbols::line::ROUNDED.bottom_left,
//             bottom_right: symbols::line::ROUNDED.bottom_right,
//             ..symbols::border::PLAIN
//         };

//         // Layout: search area at top, table at bottom
//         let [search_area, key_table_area] =
//             Layout::vertical([Constraint::Length(4), Constraint::Fill(1)]).areas(area);

//         // Render search input area
//         let search_block = Block::new()
//             .border_type(BorderType::Rounded)
//             .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
//             .title("─ Search ");
//         f.render_widget(search_block, search_area);

//         // Render search input or status
//         if self.help_prompt_active {
//             let input_area = Rect::new(
//                 search_area.x + 2,
//                 search_area.y + 2,
//                 search_area.width.saturating_sub(4),
//                 1,
//             );
//             let prompt_value = self.help_input.value();
//             let prompt_text = format!("{}", prompt_value);
//             let prompt = Paragraph::new(prompt_text);
//             f.render_widget(prompt, input_area);

//             // Place cursor
//             let cursor = self.help_input.visual_cursor() as u16;
//             let cx = input_area.x + cursor;
//             let cy = input_area.y;
//             f.set_cursor_position((cx, cy));
//         } else {
//             // Show current search query or placeholder
//             let input_area = Rect::new(
//                 search_area.x + 2,
//                 search_area.y + 2,
//                 search_area.width.saturating_sub(4),
//                 1,
//             );
//             let search_text = match &self.help_search {
//                 Some(query) => format!("Filter: {} [Press Ctrl+F to search]", query),
//                 None => "Press Ctrl+F to search keybindings".to_string(),
//             };
//             let search_style = Style::default().fg(Color::Gray);
//             let search_para = Paragraph::new(search_text).style(search_style);
//             f.render_widget(search_para, input_area);
//         }

//         // Build table data - use current context for now
//         let contexts = vec![self.keymap_context.clone()];
//         let table_data = self.build_keymap_table_data(&contexts);

//         // Update stored table length for navigation
//         self.help_table_len = table_data.len();

//         // Ensure selection is within bounds
//         if let Some(selected) = self.help_table_state.selected() {
//             if selected >= table_data.len() && !table_data.is_empty() {
//                 self.help_table_state.select(Some(0));
//             }
//         } else if !table_data.is_empty() {
//             self.help_table_state.select(Some(0));
//         }

//         let table_block = Block::new()
//             .borders(Borders::ALL)
//             .border_set(collapsed_border_set)
//             .title("─ Key Bindings ");

//         // Calculate dynamic column widths based on content
//         let max_action_len = table_data
//             .iter()
//             .map(|(a, _, _, _)| a.len())
//             .max()
//             .unwrap_or(10)
//             .max(6)
//             + 2;
//         let max_keys_len = table_data
//             .iter()
//             .map(|(_, k, _, _)| k.len())
//             .max()
//             .unwrap_or(10)
//             .max(4)
//             + 2;
//         let max_context_len = table_data
//             .iter()
//             .map(|(_, _, c, _)| c.len())
//             .max()
//             .unwrap_or(10)
//             .max(7)
//             + 2;

//         let header = Row::new(vec!["", "Action", "Keys", "Context", "Details"])
//             .style(
//                 Style::default()
//                     .fg(Color::Blue)
//                     .add_modifier(Modifier::BOLD),
//             )
//             .height(1);

//         let rows: Vec<Row> = table_data
//             .into_iter()
//             .enumerate()
//             .map(|(i, (action, keys, context, details))| {
//                 let style = if i % 2 == 0 {
//                     Style::default().bg(Color::Rgb(40, 40, 60))
//                 } else {
//                     Style::default().bg(Color::Rgb(30, 30, 40))
//                 };

//                 // Add visual marker for selected row
//                 let marker = if self.help_table_state.selected() == Some(i) {
//                     "►"
//                 } else {
//                     " "
//                 };

//                 Row::new(vec![marker.to_string(), action, keys, context, details])
//                     .style(style)
//                     .height(1)
//             })
//             .collect();

//         let table = Table::new(
//             rows,
//             [
//                 Constraint::Length(3),
//                 Constraint::Length(max_action_len as u16),
//                 Constraint::Length(max_keys_len as u16),
//                 Constraint::Length(max_context_len as u16),
//                 Constraint::Fill(1),
//             ],
//         )
//         .header(header)
//         .block(table_block)
//         .row_highlight_style(
//             Style::default()
//                 .bg(Color::Blue)
//                 .add_modifier(Modifier::BOLD),
//         );

//         f.render_stateful_widget(table, key_table_area, &mut self.help_table_state);

//         Ok(())
//     }
// }
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/layers/notify.rs

```rs

```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/layers/page.rs

```rs
use crate::{action::Action, tui::Event, ui::components::Component};
use color_eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect};

use tokio::sync::mpsc::UnboundedSender;

mod dashboard;
mod welcome;

pub(crate) use dashboard::DashboardPage;
pub(crate) use welcome::WelcomePage;

/// A top-level screen composed of zero or more components.
/// Pages own focus state among their components and expose high-level behaviors to the app.
pub trait Page {
    /// Provide the page with an action sender so it can emit `Action`s.
    #[allow(unused_variables)]
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        Ok(())
    }

    /// Initialize the page once on creation/activation.
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    /// Provide components owned by this page for App registration.
    /// Return a vector of (stable_id, component). The App will assign indices and manage focus by index or id.
    fn provide_components(&mut self) -> Vec<(String, Box<dyn Component>)> {
        Vec::new()
    }

    /// Called when the page becomes focused/active.
    fn focus(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when the page is no longer focused/active.
    fn unfocus(&mut self) -> Result<()> {
        Ok(())
    }

    /// Active keymap context for this page (e.g. "global", "setup", "dashboard").
    fn keymap_context(&self) -> &'static str {
        "global"
    }

    /// Stable identifier for this page used for navigation (must be unique across pages).
    /// Override in each concrete page implementation.
    fn id(&self) -> &'static str {
        "unknown"
    }

    /// Ordered list of component IDs for focus traversal (FocusNext / FocusPrev).
    /// First entry is considered the initial logical focus. Return an empty slice
    /// if the page has no focusable components or manages focus manually.
    fn focus_order(&self) -> &'static [&'static str] {
        &[]
    }

    /// The currently focused component id within the page for status/tooling.
    /// Pages should emit UiAction::ReportFocusedComponent to update App focus;
    /// this method is for read-only status (e.g., status bar).
    fn focused_component_id(&self) -> Option<&str> {
        None
    }

    /// Compute the layout for this page: mapping component IDs to sub-rectangles.
    /// Default: empty layout (App falls back to drawing components in the full area).
    fn layout(&self, area: Rect) -> PageLayout {
        let _ = area;
        PageLayout::empty()
    }

    /// Route an optional event to the page. Return an action to send back to the app.
    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let action = match event {
            Some(Event::Key(key)) => self.handle_key_events(key)?,
            Some(Event::Mouse(mouse)) => self.handle_mouse_events(mouse)?,
            _ => None,
        };
        Ok(action)
    }

    /// Handle key events within the page. Return an action to send back to the app.
    #[allow(unused_variables)]
    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }

    /// Handle mouse events within the page. Return an action to send back to the app.
    #[allow(unused_variables)]
    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        Ok(None)
    }

    /// Update this page in response to an action broadcast by the app or other components.
    #[allow(unused_variables)]
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }

    /// Draw the page to the provided area.
    #[allow(unused_variables)]
    fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        Ok(())
    }
}

/// Layout description for a Page.
/// The App will consult this mapping when positioning components.
#[derive(Default)]
pub struct PageLayout {
    pub regions: std::collections::HashMap<String, Rect>,
}

impl PageLayout {
    pub fn empty() -> Self {
        Self {
            regions: std::collections::HashMap::new(),
        }
    }

    pub fn with(mut self, id: &str, rect: Rect) -> Self {
        self.regions.insert(id.to_string(), rect);
        self
    }
}
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/layers/popup.rs

```rs
//! Popup system for modal dialogs and overlays
//!
//! Popups work similar to Pages but are modal and overlay the current content.
//! They support flexible sizing and positioning.

use crate::{action::Action, tui::Event, ui::components::Component};
use color_eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect};
use tokio::sync::mpsc::UnboundedSender;

/// Popup sizing and positioning configuration
#[derive(Debug, Clone)]
pub enum PopupSize {
    /// Fixed pixel/character size
    Fixed { width: u16, height: u16 },
    /// Percentage of the parent area
    Percentage { width: u8, height: u8 }, // 0-100
    /// Fullscreen popup
    Fullscreen,
    /// Custom calculation based on content
    Custom,
}

/// Popup positioning configuration
#[derive(Debug, Clone)]
pub enum PopupPosition {
    /// Center the popup
    Center,
    /// Fixed position from top-left
    Fixed { x: u16, y: u16 },
    /// Relative position (percentage)
    Relative { x: u8, y: u8 }, // 0-100
    /// Custom positioning logic
    Custom,
}

/// Configuration for popup appearance and behavior
#[derive(Debug, Clone)]
pub struct PopupConfig {
    pub size: PopupSize,
    pub position: PopupPosition,
    pub modal: bool,     // If true, dims background
    pub closable: bool,  // If true, can be closed with Esc
    pub resizable: bool, // If true, popup can be resized
}

impl Default for PopupConfig {
    fn default() -> Self {
        Self {
            size: PopupSize::Percentage {
                width: 80,
                height: 80,
            },
            position: PopupPosition::Center,
            modal: true,
            closable: true,
            resizable: false,
        }
    }
}

/// A modal popup that can contain components and handle events.
/// Similar to Page trait but designed for overlays.
pub trait Popup {
    /// Downcast to Any for type-specific operations
    fn as_any(&self) -> &dyn std::any::Any;

    /// Downcast to Any for mutable type-specific operations
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    /// Provide the popup with an action sender so it can emit `Action`s.
    #[allow(unused_variables)]
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        Ok(())
    }

    /// Initialize the popup once on creation/activation.
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    /// Provide components owned by this popup for App registration.
    /// Return a vector of (stable_id, component).
    fn provide_components(&mut self) -> Vec<(String, Box<dyn Component>)> {
        Vec::new()
    }

    /// Called when the popup becomes active/visible.
    fn focus(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when the popup is no longer active/visible.
    fn unfocus(&mut self) -> Result<()> {
        Ok(())
    }

    /// Active keymap context for this popup (e.g. "help", "confirm", "settings").
    fn keymap_context(&self) -> &'static str {
        "popup"
    }

    /// Stable identifier for this popup used for management (must be unique across popups).
    fn id(&self) -> &'static str {
        "unknown"
    }

    /// Ordered list of component IDs for focus traversal within the popup.
    fn focus_order(&self) -> &'static [&'static str] {
        &[]
    }

    /// The currently focused component id within the popup.
    fn focused_component_id(&self) -> Option<&str> {
        None
    }

    /// Configuration for popup size, position, and behavior.
    fn config(&self) -> PopupConfig {
        PopupConfig::default()
    }

    /// Calculate the actual popup area based on the parent area and config.
    /// Override this for custom sizing logic.
    fn calculate_area(&self, parent_area: Rect) -> Rect {
        let config = self.config();

        let (width, height) = match config.size {
            PopupSize::Fixed { width, height } => (width, height),
            PopupSize::Percentage { width, height } => {
                let w = (parent_area.width * width as u16) / 100;
                let h = (parent_area.height * height as u16) / 100;
                (w, h)
            }
            PopupSize::Fullscreen => (parent_area.width, parent_area.height),
            PopupSize::Custom => {
                // Default to 80% if not overridden
                let w = (parent_area.width * 80) / 100;
                let h = (parent_area.height * 80) / 100;
                (w, h)
            }
        };

        let (x, y) = match config.position {
            PopupPosition::Center => {
                let x = parent_area.x + (parent_area.width.saturating_sub(width)) / 2;
                let y = parent_area.y + (parent_area.height.saturating_sub(height)) / 2;
                (x, y)
            }
            PopupPosition::Fixed { x, y } => (parent_area.x + x, parent_area.y + y),
            PopupPosition::Relative { x, y } => {
                let px = parent_area.x + (parent_area.width * x as u16) / 100;
                let py = parent_area.y + (parent_area.height * y as u16) / 100;
                (px, py)
            }
            PopupPosition::Custom => {
                // Default to center if not overridden
                let x = parent_area.x + (parent_area.width.saturating_sub(width)) / 2;
                let y = parent_area.y + (parent_area.height.saturating_sub(height)) / 2;
                (x, y)
            }
        };

        // Ensure popup fits within parent area
        let max_x = parent_area.x + parent_area.width;
        let max_y = parent_area.y + parent_area.height;
        let final_width = width.min(max_x.saturating_sub(x));
        let final_height = height.min(max_y.saturating_sub(y));

        Rect::new(x, y, final_width, final_height)
    }

    /// Compute the layout for this popup: mapping component IDs to sub-rectangles.
    /// Default: empty layout (App falls back to drawing components in the full area).
    fn layout(&self, area: Rect) -> PopupLayout {
        let _ = area;
        PopupLayout::empty()
    }

    /// Route an optional event to the popup. Return an action to send back to the app.
    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let action = match event {
            Some(Event::Key(key)) => self.handle_key_events(key)?,
            Some(Event::Mouse(mouse)) => self.handle_mouse_events(mouse)?,
            _ => None,
        };
        Ok(action)
    }

    /// Handle key events within the popup. Return an action to send back to the app.
    #[allow(unused_variables)]
    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }

    /// Handle mouse events within the popup. Return an action to send back to the app.
    #[allow(unused_variables)]
    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        Ok(None)
    }

    /// Update this popup in response to an action broadcast by the app or other components.
    #[allow(unused_variables)]
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }

    /// Draw the popup to the provided area.
    /// The area provided here is already calculated based on config().
    #[allow(unused_variables)]
    fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        Ok(())
    }
}

/// Layout description for a Popup.
/// The App will consult this mapping when positioning components within the popup.
#[derive(Default)]
pub struct PopupLayout {
    pub regions: std::collections::HashMap<String, Rect>,
}

impl PopupLayout {
    pub fn empty() -> Self {
        Self {
            regions: std::collections::HashMap::new(),
        }
    }

    pub fn with(mut self, id: &str, rect: Rect) -> Self {
        self.regions.insert(id.to_string(), rect);
        self
    }
}
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/layers/page/dashboard.rs

```rs
use crate::{
    action::{Action, UiAction},
    layers::page::Page,
    ui::components::{Component, TaskList},
};
use color_eyre::Result;
use tokio::sync::mpsc::UnboundedSender;

/// DashboardPage: registers task list component (StatusBar now rendered globally by App).
pub struct DashboardPage {
    tx: Option<UnboundedSender<Action>>,
    focused: Option<String>,
    // focusables: [&'static str; 2],
}

impl DashboardPage {
    pub fn new() -> Self {
        Self {
            tx: None,
            focused: None,
            // focusables: ["tasks", "fps_panel"],
        }
    }
}

impl Page for DashboardPage {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn provide_components(&mut self) -> Vec<(String, Box<dyn Component>)> {
        vec![
            (
                "tasks".to_string(),
                Box::new(TaskList::new()) as Box<dyn Component>,
            ),
            // StatusBar removed; rendered globally by App
            // Placeholder lines to keep line count stable
            // (additional components may be added here later)
        ]
    }

    fn focus(&mut self) -> Result<()> {
        if let Some(tx) = &self.tx {
            let _ = tx.send(Action::Ui(UiAction::ReportFocusedComponent(
                "tasks".to_string(),
            )));
        }
        Ok(())
    }

    fn keymap_context(&self) -> &'static str {
        "dashboard"
    }

    fn id(&self) -> &'static str {
        "dashboard"
    }

    fn focus_order(&self) -> &'static [&'static str] {
        &["tasks"]
    }

    fn focused_component_id(&self) -> Option<&str> {
        self.focused.as_deref()
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::Ui(UiAction::ReportFocusedComponent(id)) = action {
            self.focused = Some(id);
        }
        Ok(None)
    }
}
```

# /Users/tim-jonaswechler/GitHub-Projekte/forge_of_stories/crates/ui/wizard/src/layers/page/welcome.rs

```rs
use crate::{
    action::{Action, UiAction},
    layers::page::{Page, PageLayout},
    ui::components::{AetherStatusListComponent, Component, WizardLogoComponent},
};
use color_eyre::Result;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// WelcomePage: shows logo + a minimal Aether status list (settings, cert, uds)
pub struct WelcomePage {
    tx: Option<tokio::sync::mpsc::UnboundedSender<Action>>,
    focused: Option<String>,
    focusables: [&'static str; 1],
    status_component: Option<AetherStatusListComponent>,
}

impl WelcomePage {
    pub fn new() -> Self {
        Self {
            tx: None,
            focused: Some("aether_status".to_string()),
            focusables: ["aether_status"],
            status_component: Some(AetherStatusListComponent::new()),
        }
    }
}

impl Page for WelcomePage {
    fn register_action_handler(
        &mut self,
        tx: tokio::sync::mpsc::UnboundedSender<Action>,
    ) -> Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn provide_components(&mut self) -> Vec<(String, Box<dyn Component>)> {
        let mut out: Vec<(String, Box<dyn Component>)> = vec![(
            "wizard_logo".to_string(),
            Box::new(WizardLogoComponent::new()) as Box<dyn Component>,
        )];
        if let Some(c) = self.status_component.take() {
            out.push(("aether_status".to_string(), Box::new(c)));
        }
        out
    }

    fn focus(&mut self) -> Result<()> {
        if let Some(tx) = &self.tx {
            if let Some(first) = self.focusables.first() {
                let _ = tx.send(Action::Ui(UiAction::ReportFocusedComponent(
                    (*first).to_string(),
                )));
            }
        }
        Ok(())
    }

    fn keymap_context(&self) -> &'static str {
        "welcome"
    }

    fn id(&self) -> &'static str {
        "welcome"
    }

    fn focus_order(&self) -> &'static [&'static str] {
        &["aether_status"]
    }

    fn focused_component_id(&self) -> Option<&str> {
        self.focused.as_deref()
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::Ui(UiAction::ReportFocusedComponent(id)) = &action {
            self.focused = Some(id.clone());
        }
        Ok(None)
    }

    fn layout(&self, area: Rect) -> PageLayout {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(9),
                Constraint::Length(6),
                Constraint::Length(3),
            ])
            .split(area);
        PageLayout::empty()
            .with("wizard_logo", vertical[0])
            .with("aether_status", vertical[1])
            .with("welcome_message", vertical[2])
    }
}
```

