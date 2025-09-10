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

pub mod settings;

use crate::{
    action::{
        Action, AppAction, LayerKind, LogicAction, Notification, TaskId, TaskKind, TaskProgress,
        TaskResult, TaskSpec, UiAction, UiMode,
    },
    cli::{Cli, Cmd, RunMode},
    components::Component,
    pages::Page,
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
    keymap_context: String,
    comp_id_to_index: HashMap<String, usize>,

    // Layers & notifications
    layers: Vec<LayerEntry>,
    notifications: Vec<Notification>,

    // Background tasks
    tasks: HashMap<TaskId, TaskState>,
    next_task_seq: u64,

    // App state
    should_quit: bool,
    should_suspend: bool,
    mode: Mode,
    ui_mode: UiMode,

    // Timing / perf
    tick_rate: f64,
    frame_rate: f64,
    last_tick_key_events: Vec<KeyEvent>,

    // Action channel
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
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
                RunMode::Setup => {
                    // TODO: register setup page and components
                    (vec![], vec![], None)
                }
                RunMode::Dashboard => {
                    // TODO: register dashboard page and components
                    (vec![], vec![], None)
                }
            },
        };

        // Default rates; in the future read from settings::GeneralCfg
        let tick_rate = settings.get::<settings::Wizard>().unwrap().tick_rate;
        let frame_rate = settings.get::<settings::Wizard>().unwrap().fps;

        Ok(Self {
            base,
            settings,
            aether_settings,
            pages,
            components,
            active_page_index,
            focused_component_index: None,
            keymap_context: "global".to_string(),
            comp_id_to_index: HashMap::new(),
            layers: Vec::new(),
            notifications: Vec::new(),
            tasks: HashMap::new(),
            next_task_seq: 0,
            should_quit: false,
            should_suspend: false,
            mode: Mode::Home,
            ui_mode: UiMode::Normal,
            tick_rate,
            frame_rate,
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
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
                let _ = self.register_components(provided_all, size)?;
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
        // Without a centralized keymap, we let pages/components process keys.
        // Keep the multi-key buffer for future use.
        self.last_tick_key_events.push(key);
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
                    // TODO: open a help popup layer
                }

                // Structured actions: UI
                Action::Ui(ui) => self.handle_ui_action(tui, ui)?,

                // Structured actions: App
                Action::App(app) => self.handle_app_action(app)?,

                // Structured actions: Logic
                Action::Logic(logic) => self.handle_logic_action(logic)?,
            }

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
                if self.components.is_empty() {
                    self.focused_component_index = None;
                } else {
                    let next = self
                        .focused_component_index
                        .map(|i| (i + 1) % self.components.len())
                        .unwrap_or(0);
                    self.focused_component_index = Some(next);
                }
            }
            UiAction::FocusPrev => {
                if self.components.is_empty() {
                    self.focused_component_index = None;
                } else {
                    let prev = self
                        .focused_component_index
                        .map(|i| {
                            if i == 0 {
                                self.components.len() - 1
                            } else {
                                i - 1
                            }
                        })
                        .unwrap_or(self.components.len() - 1);
                    self.focused_component_index = Some(prev);
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
                self.layers.push(LayerEntry {
                    kind: LayerKind::Popup,
                    id: Some(id),
                });
            }
            UiAction::ClosePopup { id } => {
                if let Some(pos) = self
                    .layers
                    .iter()
                    .rposition(|l| l.kind == LayerKind::Popup && l.id.as_deref() == Some(&id))
                {
                    self.layers.remove(pos);
                }
            }
            UiAction::PushLayer(kind) => {
                self.layers.push(LayerEntry { kind, id: None });
            }
            UiAction::PopLayer => {
                self.layers.pop();
            }

            UiAction::ShowNotification(n) => {
                // Replace if same id exists
                if let Some(pos) = self.notifications.iter().position(|x| x.id == n.id) {
                    self.notifications[pos] = n;
                } else {
                    self.notifications.push(n);
                }
            }
            UiAction::DismissNotification { id } => {
                if let Some(pos) = self.notifications.iter().position(|x| x.id == id) {
                    self.notifications.remove(pos);
                }
            }
        }
        Ok(())
    }

    fn handle_app_action(&mut self, app: AppAction) -> Result<()> {
        match app {
            AppAction::SetActivePage { id } => {
                // For now we only store the name; when pages have stable IDs we can map them.
                info!("Request to activate page: {id} (not mapped yet)");
            }
            AppAction::SetKeymapContext { name } => {
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
                    spec,
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
                    .send(Action::Logic(LogicAction::TaskStarted { id }));
            }
            LogicAction::CancelTask { id } => {
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
                // TODO: load config from disk
                info!("LoadConfig requested (not implemented)");
            }
            LogicAction::SaveConfig => {
                // TODO: save config to disk
                info!("SaveConfig requested (not implemented)");
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

            // Draw active page first (base layer)
            if let Some(ix) = self.active_page_index {
                if let Some(page) = self.pages.get_mut(ix) {
                    if let Err(err) = page.draw(frame, area) {
                        let _ = self
                            .action_tx
                            .send(Action::Error(format!("Failed to draw page: {:?}", err)));
                    }
                }
            }

            // Draw registered components (secondary base layer)
            for component in self.components.iter_mut() {
                if let Err(err) = component.draw(frame, area) {
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
                        // TODO: Have a layer registry render popups/overlays as full-screen widgets
                        let _ = layer; // placeholder
                    }
                    LayerKind::Notification => {
                        // Notifications are drawn at the very top; actual rendering TBD.
                    }
                }
            }

            // Notifications would logically be drawn last; kept here as a reminder.
            for _n in self.notifications.iter() {
                // TODO: Draw notification banners/toasts
            }
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
