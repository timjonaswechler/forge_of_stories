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
use crate::pages::WelcomePage;
use crate::{
    action::{
        Action, AppAction, LayerKind, LogicAction, NotificationLevel, TaskId, TaskKind,
        TaskProgress, TaskResult, TaskSpec, UiAction, UiMode,
    },
    cli::{Cli, Cmd, RunMode},
    components::{Component, StatusBar},
    pages::{DashboardPage, Page},
    tui::{Event, Tui},
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

/// Layer entry tracked by the application shell.
/// Rendering is page/component specific and not implemented here.
#[derive(Debug, Clone)]
pub struct LayerEntry {
    pub kind: LayerKind,
    pub id: Option<String>,
    /// Z-order / priority (higher draws later). Currently unused => default 0.
    pub priority: i32,
}

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

/// Main Wizard application.
pub struct App {
    // Platform
    base: AppBase,

    // Settings stores
    settings: Arc<settings::SettingsStore>,
    aether_settings: Arc<settings::SettingsStore>,

    // UI collections
    pages: Vec<Box<dyn Page>>,
    components: Vec<Box<dyn Component>>,
    /// Global status bar (no longer provided by each page).
    status_bar: StatusBar,

    // Navigation and focus
    active_page_index: Option<usize>,
    focused_component_index: Option<usize>,
    /// Ordered cycle of focusable component indices (derived from page.focus_order()).
    focus_cycle: Vec<usize>,
    /// When a popup layer is active, normal component focus traversal is locked.
    popup_focus_lock: bool,
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

    // Help search prompt state
    help_search_active: bool,
    help_search_buffer: String,

    // Layers registry
    layer_registry: crate::layers::BasicLayerRegistry,

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
    toast_position: crate::layers::ToastPosition,
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
        // Derive toast position from settings (wizard.toast_position) with graceful fallback.
        let toast_position_value = {
            use crate::layers::ToastPosition;
            let map = settings_for_layer.effective_settings();
            let tbl_opt = map.get("wizard").and_then(|v| v.as_table());
            let s_opt = tbl_opt
                .and_then(|t| t.get("toast_position"))
                .and_then(|v| v.as_str());
            match s_opt.map(|s| s.to_ascii_lowercase()) {
                Some(ref s) if s == "top_left" || s == "topleft" => ToastPosition::TopLeft,
                Some(ref s) if s == "bottom_left" || s == "bottomleft" => ToastPosition::BottomLeft,
                Some(ref s) if s == "bottom_right" || s == "bottomright" => {
                    ToastPosition::BottomRight
                }
                Some(ref s) if s == "top_right" || s == "topright" => ToastPosition::TopRight,
                _ => ToastPosition::TopRight,
            }
        };
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
            layers: Vec::new(),
            tasks: HashMap::new(),
            next_task_seq: 0,
            should_quit: false,
            should_suspend: false,
            ui_mode: UiMode::Normal,
            help_search_active: false,
            help_search_buffer: String::new(),
            layer_registry: {
                let mut lr = crate::layers::BasicLayerRegistry::new();
                lr.register_settings_handler(settings_for_layer.clone());
                // Initialize show_global from settings
                let help_show_global = settings_for_layer
                    .get::<settings::Wizard>()
                    .unwrap()
                    .help_show_global;
                if !help_show_global {
                    lr.toggle_show_global();
                }
                // Initialize wrap_on from settings
                let help_wrap_on = settings_for_layer
                    .get::<settings::Wizard>()
                    .unwrap()
                    .help_wrap_on;
                if !help_wrap_on {
                    lr.toggle_wrap();
                }
                lr
            },
            tick_rate,
            frame_rate,
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
            task_mgr,
            toast_notifications: Vec::new(),
            toast_last_visible: 0,
            toast_last_highest: None,
            toast_position: toast_position_value,
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
                info!("UI mode changed to {:?}", self.ui_mode);
            }
            UiAction::EnterEditMode => {
                self.ui_mode = UiMode::Edit;
                info!("UI mode changed to Edit");
            }
            UiAction::ExitEditMode => {
                self.ui_mode = UiMode::Normal;
                info!("UI mode changed to Normal");
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
            UiAction::HelpScrollUp => {
                self.layer_registry.scroll_help_lines(-1);
            }
            UiAction::HelpScrollDown => {
                self.layer_registry.scroll_help_lines(1);
            }
            UiAction::HelpPageUp => {
                if let Ok(size) = _tui.size() {
                    let page = (size.height as i16 / 2).max(1);
                    self.layer_registry.scroll_help_lines(-page);
                } else {
                    self.layer_registry.scroll_help_lines(-10);
                }
            }
            UiAction::HelpPageDown => {
                if let Ok(size) = _tui.size() {
                    let page = (size.height as i16 / 2).max(1);
                    self.layer_registry.scroll_help_lines(page);
                } else {
                    self.layer_registry.scroll_help_lines(10);
                }
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
                    crate::pages::PageLayout::empty()
                }
            } else {
                crate::pages::PageLayout::empty()
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

    // Converts a KeyEvent to a chord string (e.g., "ctrl+c", "tab", "esc")
    // fn key_event_to_chord_string(&self, key: &KeyEvent) -> String {
    //     use crossterm::event::{KeyCode, KeyModifiers};

    //     let mut mods: Vec<&'static str> = Vec::new();
    //     if key.modifiers.contains(KeyModifiers::CONTROL) {
    //         mods.push("ctrl");
    //     }
    //     if key.modifiers.contains(KeyModifiers::ALT) {
    //         mods.push("alt");
    //     }
    //     if key.modifiers.contains(KeyModifiers::SHIFT) {
    //         mods.push("shift");
    //     }

    //     let key_str = match key.code {
    //         KeyCode::Char(c) => c.to_ascii_lowercase().to_string(),
    //         KeyCode::Enter => "enter".into(),
    //         KeyCode::Esc => "esc".into(),
    //         KeyCode::Tab => "tab".into(),
    //         KeyCode::Backspace => "backspace".into(),
    //         KeyCode::Left => "left".into(),
    //         KeyCode::Right => "right".into(),
    //         KeyCode::Up => "up".into(),
    //         KeyCode::Down => "down".into(),
    //         KeyCode::Delete => "delete".into(),
    //         KeyCode::Home => "home".into(),
    //         KeyCode::End => "end".into(),
    //         KeyCode::PageUp => "pageup".into(),
    //         KeyCode::PageDown => "pagedown".into(),
    //         KeyCode::F(n) => format!("f{}", n),
    //         _ => return "unknown".to_string(),
    //     };

    //     if mods.is_empty() {
    //         key_str
    //     } else {
    //         format!("{}+{}", mods.join("+"), key_str)
    //     }
    // }
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
            "HelpScrollUp" => Some(Action::Ui(UiAction::HelpScrollUp)),
            "HelpScrollDown" => Some(Action::Ui(UiAction::HelpScrollDown)),
            "HelpPageUp" => Some(Action::Ui(UiAction::HelpPageUp)),
            "HelpPageDown" => Some(Action::Ui(UiAction::HelpPageDown)),
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
