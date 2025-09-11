//! Wizard application shell
//!
//! Responsibilities:
//! - Initialize TUI, settings, pages, and components
//! - Store current page and currently focused component at app level
//! - Layer system scaffolding (popups, notifications)
//! - Background task scaffolding
//! - Split actions: UI, App, Logic, plus legacy core actions
//! - Normal vs Edit mode toggling
//!
//! Notes:
//! - This is a minimal, compiling scaffolding. Pages/components can be added incrementally.
//! - Keybindings are not wired yet; events are routed to pages/components which may emit actions.

pub(crate) mod settings;
pub(crate) mod task_manager;

use self::task_manager::{TaskManager, TaskManagerHandle};
use crate::layers::LayerRegistry;
use crate::{
    action::{
        Action, AppAction, LayerKind, LogicAction, TaskId, TaskKind, TaskProgress, TaskResult,
        TaskSpec, UiAction, UiMode,
    },
    cli::{Cli, Cmd, RunMode},
    components::Component,
    pages::{DashboardPage, Page, SetupPage},
    tui::{Event, Tui},
};
use app::AppBase;
pub use app::init;
use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::layout::{Rect, Size};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Home,
}

/// Layer entry tracked by the application shell.
/// Rendering is page/component specific and not implemented here.
#[derive(Debug, Clone)]
pub struct LayerEntry {
    pub kind: LayerKind,
    pub id: Option<String>,
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

    // Navigation and focus
    active_page_index: Option<usize>,
    focused_component_index: Option<usize>,
    /// Ordered cycle of focusable component indices (derived from page.focus_order()).
    focus_cycle: Vec<usize>,
    /// When a popup layer is active, normal component focus traversal is locked.
    popup_focus_lock: bool,
    keymap_context: String,
    comp_id_to_index: HashMap<String, usize>,

    // Layers
    layers: Vec<LayerEntry>,

    // Background tasks
    tasks: HashMap<TaskId, TaskState>,
    next_task_seq: u64,

    // App state
    should_quit: bool,
    should_suspend: bool,
    mode: Mode,
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
}

impl App {
    pub fn new(base: AppBase, cli: Cli) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        // Settings stores (wizard + aether). We can split/merge later if needed.
        let settings = Arc::new(settings::build_wizard_settings_store()?);
        let aether_settings = Arc::new(settings::build_wizard_settings_store()?);

        // Initial UI collections; pages/components can be registered here based on CLI.
        let (pages, components, active_page_index): (
            Vec<Box<dyn Page>>,
            Vec<Box<dyn Component>>,
            Option<usize>,
        ) = match cli.cmd {
            Cmd::Run { mode } => match mode {
                RunMode::Setup => (vec![Box::new(SetupPage::new())], vec![], Some(0)),
                RunMode::Dashboard => (vec![Box::new(DashboardPage::new())], vec![], Some(0)),
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
            active_page_index,
            focused_component_index: None,
            focus_cycle: Vec::new(),
            popup_focus_lock: false,
            keymap_context: "global".to_string(),
            comp_id_to_index: HashMap::new(),
            layers: Vec::new(),
            tasks: HashMap::new(),
            next_task_seq: 0,
            should_quit: false,
            should_suspend: false,
            mode: Mode::Home,
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
                // Initialize last help search from settings (if any)
                let help_last_search = settings_for_layer
                    .get::<settings::Wizard>()
                    .unwrap()
                    .help_last_search
                    .clone();
                if let Some(q) = help_last_search {
                    if !q.trim().is_empty() {
                        lr.set_help_search(Some(q));
                    }
                }
                lr
            },
            tick_rate,
            frame_rate,
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
            task_mgr,
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
        // Pages provide their owned components; App registers them so focus can be tracked by index or id
        {
            let rect = tui.size()?;
            let size = Size {
                width: rect.width,
                height: rect.height,
            };
            // Collect page-provided components first to avoid borrowing `self.pages`
            // while calling `self.register_components(...)`.
            let mut provided_all = Vec::new();
            for page in self.pages.iter_mut() {
                let provided = page.provide_components();
                if !provided.is_empty() {
                    provided_all.extend(provided);
                }
            }
            if !provided_all.is_empty() {
                // Change detection: only (re)register if the set of component IDs differs.
                let new_ids: std::collections::HashSet<String> =
                    provided_all.iter().map(|(id, _)| id.clone()).collect();
                let current_ids: std::collections::HashSet<String> =
                    self.comp_id_to_index.keys().cloned().collect();

                let changed = self.components.len() != provided_all.len() || new_ids != current_ids;

                if changed {
                    // Clear previous registry before re-registering.
                    self.components.clear();
                    self.comp_id_to_index.clear();
                    self.focused_component_index = None;
                    self.focus_cycle.clear();
                    let _ = self.register_components(provided_all, size)?;
                    // Rebuild focus cycle from active page's declared order (skip non-existent IDs).
                    if let Some(ix) = self.active_page_index {
                        if let Some(page) = self.pages.get(ix) {
                            let order = page.focus_order();
                            for id in order {
                                if let Some(&cix) = self.comp_id_to_index.get(*id) {
                                    self.focus_cycle.push(cix);
                                }
                            }
                        }
                    }
                } else if self.focus_cycle.is_empty() {
                    // Initial build (first run where list matched but cycle not yet built)
                    if let Some(ix) = self.active_page_index {
                        if let Some(page) = self.pages.get(ix) {
                            let order = page.focus_order();
                            for id in order {
                                if let Some(&cix) = self.comp_id_to_index.get(*id) {
                                    self.focus_cycle.push(cix);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Update keymap context from active page (scaffold) and trigger initial page focus reporting.
        if let Some(ix) = self.active_page_index {
            if let Some(page) = self.pages.get_mut(ix) {
                self.keymap_context = page.keymap_context().to_string();
                // Ask the page to emit its current focused component (UiAction::ReportFocusedComponent).
                let _ = page.focus();
                // Set initial focus to first in cycle if none reported yet.
                if self.focused_component_index.is_none() && !self.focus_cycle.is_empty() {
                    self.focused_component_index = Some(self.focus_cycle[0]);
                }
            }
        }

        // Register actions and settings for all components
        for component in self.components.iter_mut() {
            component.register_action_handler(self.action_tx.clone())?;
        }
        for component in self.components.iter_mut() {
            component.register_settings_handler(self.settings.clone())?;
        }
        // Initialize components with terminal size
        {
            let rect = tui.size()?;
            let size = Size {
                width: rect.width,
                height: rect.height,
            };
            for component in self.components.iter_mut() {
                component.init(size)?;
            }
        }

        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                // tui.mouse(true);
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

        // Route event to active page (if any)
        if let Some(ix) = self.active_page_index {
            if let Some(page) = self.pages.get_mut(ix) {
                if let Some(action) = page.handle_events(Some(event.clone()))? {
                    action_tx.send(action)?;
                }
            }
        }

        // Route event to components
        for component in self.components.iter_mut() {
            if let Some(action) = component.handle_events(Some(event.clone()))? {
                action_tx.send(action)?;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        use crossterm::event::{KeyCode, KeyModifiers};
        // When the Help search prompt is active, capture input here
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
                    // Forward key to help prompt widget
                    let _ = self
                        .action_tx
                        .send(Action::Ui(UiAction::HelpPromptKey(key)));
                }
                _ => {
                    // Forward other keys (arrows, ctrl shortcuts) to the prompt widget as well
                    let _ = self
                        .action_tx
                        .send(Action::Ui(UiAction::HelpPromptKey(key)));
                }
            }
            return Ok(());
        }

        // Build a chord string similar to the exported keymap format
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
                // Unhandled key; keep buffer for potential multi-key in the future
                self.last_tick_key_events.push(key);
                return Ok(());
            }
        };
        let chord = if mods.is_empty() {
            key_str
        } else {
            format!("{}+{}", mods.join("+"), key_str)
        };

        // Export keymap for current context and try to translate chord -> action
        let map = self
            .settings
            .export_keymap_for(::settings::DeviceFilter::Keyboard, &self.keymap_context);

        if let Some((action_name, _)) = map
            .iter()
            .find(|(_, chords)| chords.iter().any(|c| c.eq_ignore_ascii_case(&chord)))
        {
            let action = match action_name.as_str() {
                "Quit" | "quit" => Some(Action::Quit),
                "Help" | "help" => Some(Action::Help),
                "OpenPopup" | "popup" => {
                    Some(Action::Ui(UiAction::OpenPopup { id: "help".into() }))
                }
                "ModeInsert" | "insert" => Some(Action::Ui(UiAction::EnterEditMode)),
                "ModeNormal" | "normal" => Some(Action::Ui(UiAction::ExitEditMode)),
                "ModeCycle" | "modecycle" => Some(Action::Ui(UiAction::ToggleEditMode)),
                "NextField" | "next" => Some(Action::Ui(UiAction::FocusNext)),
                "PreviousField" | "prev" => Some(Action::Ui(UiAction::FocusPrev)),
                // Help controls
                "HelpToggleGlobal" | "helptoggleglobal" => {
                    Some(Action::Ui(UiAction::HelpToggleGlobal))
                }
                "HelpToggleWrap" | "helptogglewrap" => Some(Action::Ui(UiAction::HelpToggleWrap)),
                "HelpScrollUp" | "helpscrollup" => Some(Action::Ui(UiAction::HelpScrollUp)),
                "HelpScrollDown" | "helpscrolldown" => Some(Action::Ui(UiAction::HelpScrollDown)),
                "HelpPageUp" | "helppageup" => Some(Action::Ui(UiAction::HelpPageUp)),
                "HelpPageDown" | "helppagedown" => Some(Action::Ui(UiAction::HelpPageDown)),
                "HelpSearch" | "helpsearch" => Some(Action::Ui(UiAction::BeginHelpSearch)),
                _ => None,
            };
            if let Some(a) = action {
                self.action_tx.send(a)?;
            }
        } else {
            // Fallback: buffer key for potential multi-key support later
            self.last_tick_key_events.push(key);
        }

        Ok(())
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
            }
            match action.clone() {
                // Legacy/core actions
                Action::Tick => {
                    self.last_tick_key_events.clear();
                    // Refresh keymap context from the active page each tick and notify UI if it changed
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
                    // Toggle the help popup layer; report visibility accordingly
                    if let Some(pos) = self.layers.iter().rposition(|l| {
                        l.kind == LayerKind::Popup && l.id.as_deref() == Some("help")
                    }) {
                        self.layers.remove(pos);
                        let _ = self
                            .action_tx
                            .send(Action::Ui(UiAction::ReportHelpVisible(false)));
                    } else {
                        self.layers.push(LayerEntry {
                            kind: LayerKind::Popup,
                            id: Some("help".into()),
                        });
                        let _ = self
                            .action_tx
                            .send(Action::Ui(UiAction::ReportHelpVisible(true)));
                    }
                }

                // Structured actions: UI
                Action::Ui(ui) => self.handle_ui_action(tui, ui)?,

                // Structured actions: App
                Action::App(app) => self.handle_app_action(app)?,

                // Structured actions: Logic
                Action::Logic(logic) => self.handle_logic_action(logic)?,
            }

            self.layer_registry.update_from_action(&action);
            // Propagate updates into active page first
            if let Some(ix) = self.active_page_index {
                if let Some(page) = self.pages.get_mut(ix) {
                    if let Some(follow_up) = page.update(action.clone())? {
                        self.action_tx.send(follow_up)?
                    }
                }
            }
            // Then components
            for component in self.components.iter_mut() {
                if let Some(follow_up) = component.update(action.clone())? {
                    self.action_tx.send(follow_up)?
                }
            }
        }
        Ok(())
    }

    fn handle_ui_action(&mut self, _tui: &mut Tui, ui: UiAction) -> Result<()> {
        match ui {
            UiAction::FocusNext => {
                if self.popup_focus_lock {
                    // Ignore traversal while popup active.
                } else if self.focus_cycle.is_empty() {
                    self.focused_component_index = None;
                } else {
                    let pos = self
                        .focused_component_index
                        .and_then(|cur| self.focus_cycle.iter().position(|&c| c == cur))
                        .unwrap_or(0);
                    let next_pos = (pos + 1) % self.focus_cycle.len();
                    self.focused_component_index = Some(self.focus_cycle[next_pos]);
                    // Emit ReportFocusedComponent so status bar/colors can react.
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
                    // Ignore traversal while popup active.
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
                }
            }
            UiAction::ReportFocusedComponent(id) => {
                if let Some(ix) = self.comp_id_to_index.get(&id).cloned() {
                    self.focused_component_index = Some(ix);
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

            UiAction::OpenPopup { id } => {
                let is_help = id == "help";
                self.layers.push(LayerEntry {
                    kind: LayerKind::Popup,
                    id: Some(id.clone()),
                });
                self.popup_focus_lock = true;
                // While popup is open we conceptually "blur" component focus.
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
                // If no more popups remain, release focus lock and restore first focus in cycle.
                let any_popup = self.layers.iter().any(|l| l.kind == LayerKind::Popup);
                if !any_popup {
                    self.popup_focus_lock = false;
                    if !self.focus_cycle.is_empty() {
                        self.focused_component_index = Some(self.focus_cycle[0]);
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
                self.layers.push(LayerEntry { kind, id: None });
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

            UiAction::ShowNotification(_n) => {
                // ToastManager handles notifications; no-op here.
            }
            UiAction::ReportNotificationCount(_n) => {
                // App currently does not persist the visible count; it is forwarded to UI components.
            }
            UiAction::ReportNotificationSeverity(_sev) => {
                // App does not persist the highest severity; components (e.g., StatusBar) consume this.
            }
            UiAction::DismissNotification { .. } => {
                // ToastManager handles notifications; no-op here.
            }

            // Help controls: toggle global, scroll, paging, search
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
                // Approximate page size from terminal height (half height)
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
                // Persist last search (empty => clear)
                let val = if query.trim().is_empty() {
                    None
                } else {
                    Some(query)
                };
                let _ = self
                    .settings
                    .update::<settings::Wizard>(|w| w.help_last_search = val);
            }
            UiAction::HelpSearchClear => {
                self.layer_registry.clear_help_search();
                // Clear persisted last search
                let _ = self
                    .settings
                    .update::<settings::Wizard>(|w| w.help_last_search = None);
            }
            UiAction::BeginHelpSearch => {
                self.help_search_active = true;
                self.help_search_buffer.clear();
            }
            UiAction::ReportHelpVisible(_v) => {
                // Forward-only event for components (e.g., StatusBar).
            }
            UiAction::PersistHelpShowGlobal(value) => {
                // Persist the current show_global preference into settings.
                let _ = self
                    .settings
                    .update::<settings::Wizard>(|w| w.help_show_global = value);
            }
            UiAction::HelpToggleWrap => {
                // Toggle wrap flag locally and persist the new setting.
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
                // Persist the current wrap preference into settings.
                let _ = self
                    .settings
                    .update::<settings::Wizard>(|w| w.help_wrap_on = value);
            }
            UiAction::HelpPromptKey(_key) => {
                // No-op: prompt widget state is handled within the layer registry.
            }
            UiAction::ReportHelpSearchBuffer(_buf) => {
                // No-op: can be used by UI elements to show live search input.
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
                        // Gather new page data in a scoped borrow
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
                            let _ = page.focus(); // emits ReportFocusedComponent
                        }
                        // Register components after releasing mutable borrow of page
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
                // TODO: implement settings persistence
                info!("SaveSettings requested (not implemented)");
            }
            AppAction::LoadSettings => {
                // TODO: implement settings loading
                info!("LoadSettings requested (not implemented)");
            }
        }
        Ok(())
    }

    fn handle_logic_action(&mut self, logic: LogicAction) -> Result<()> {
        match logic {
            LogicAction::SpawnTask(spec) => {
                // Assign a task id and register it
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
                // Notify started
                let _ = self
                    .action_tx
                    .send(Action::Logic(LogicAction::TaskStarted { id: id.clone() }));

                // Delegate execution to TaskManager (simulated task for now)
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
                // Register task and delegate to TaskManager
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
                // Register task and delegate to TaskManager
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

            // Draw active page first (base layer)
            if let Some(ix) = self.active_page_index {
                if let Some(page) = self.pages.get_mut(ix) {
                    if let Err(err) = page.draw(frame, main_area) {
                        let _ = self
                            .action_tx
                            .send(Action::Error(format!("Failed to draw page: {:?}", err)));
                    }
                }
            }

            // Draw registered components (secondary base layer)
            let status_index = self.comp_id_to_index.get("status").cloned();
            for (i, component) in self.components.iter_mut().enumerate() {
                let target = if Some(i) == status_index {
                    status_area
                } else {
                    main_area
                };
                if let Err(err) = component.draw(frame, target) {
                    let _ = self.action_tx.send(Action::Error(format!(
                        "Failed to draw component: {:?}",
                        err
                    )));
                }
            }

            // Draw layers on top (popups, overlays, notifications)
            // This is only scaffolding; concrete rendering belongs to pages/components.
            for layer in self.layers.iter() {
                match layer.kind {
                    LayerKind::Popup | LayerKind::Overlay => {
                        if let Err(err) = self.layer_registry.render_layer(
                            frame,
                            area,
                            layer.kind,
                            layer.id.as_deref(),
                        ) {
                            let _ = self
                                .action_tx
                                .send(Action::Error(format!("Failed to render layer: {:?}", err)));
                        }
                    }
                    LayerKind::Notification => {
                        // Notifications are rendered by the ToastManager component; keeping sentinel for z-order.
                    }
                }
            }

            // Status bar is now rendered as its own component at the bottom area; removed temporary overlay panel.
        })?;
        Ok(())
    }

    /// Register a component provided by a page, returning its index.
    /// The page should pass a stable `id` to enable focus by id.
    pub fn register_component(
        &mut self,
        id: String,
        mut component: Box<dyn Component>,
        initial_size: Size,
    ) -> Result<usize> {
        // Wire action/settings and initialize with the current terminal size
        component.register_action_handler(self.action_tx.clone())?;
        component.register_settings_handler(self.settings.clone())?;
        component.init(initial_size)?;

        let index = self.components.len();
        self.components.push(component);
        self.comp_id_to_index.insert(id, index);

        // If nothing focused yet, focus the first registered component
        if self.focused_component_index.is_none() {
            self.focused_component_index = Some(index);
        }
        Ok(index)
    }

    /// Bulk register components from a page.
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
}
