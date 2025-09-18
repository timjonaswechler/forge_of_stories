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

use crate::{
    action::{Action, UiAction, UiMode},
    cli::{Cli, Cmd, RunMode},
    layers::{
        self, ActionOutcome, LayerAction, LayerSystem,
        page::{DashboardPage, WelcomePage},
    },
    tui::{Event, Tui},
    ui::components::{Component, ComponentKey, StatusBar},
};
use app::AppBase;
pub use app::init;
use color_eyre::Result;
use crossterm::event::KeyEvent;
use crossterm::event::{KeyCode, KeyModifiers};
use keymap_registry::WizardActionRegistry;
use ratatui::layout::Rect;
use settings::ActionRegistry;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, warn};

/// Main Wizard application.
pub struct App {
    // Platform
    base: AppBase,
    settings: Arc<settings::SettingsStore>,
    aether_settings: Arc<settings::SettingsStore>,
    status_bar: StatusBar,
    layers: LayerSystem,
    action_registry: WizardActionRegistry,
    should_quit: bool,
    should_suspend: bool,
    ui_mode: UiMode,
    tick_rate: f64,
    frame_rate: f64,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
}

impl App {
    pub fn new(base: AppBase, cli: Cli) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        // Settings stores (wizard + aether). We can split/merge later if needed.
        let settings = Arc::new(settings::build_wizard_settings_store()?);
        let aether_settings = Arc::new(settings::build_wizard_settings_store()?);

        // Default rates; in the future read from settings::GeneralCfg
        let tick_rate = settings.get::<settings::Wizard>().unwrap().tick_rate;
        let frame_rate = settings.get::<settings::Wizard>().unwrap().fps;

        let mut layers = LayerSystem::new();
        match cli.cmd {
            Cmd::Run { mode } => match mode {
                RunMode::Setup => {
                    let welcome_page = layers.create_page("Welcome", WelcomePage::new("Welcome"));
                    layers.activate_page(welcome_page);
                }
                RunMode::Dashboard => {
                    let dashboard_page =
                        layers.create_page("Dashboard", DashboardPage::new("Dashboard"));
                    layers.activate_page(dashboard_page);
                }
            },
        }

        let mut app = Self {
            base,
            settings,
            aether_settings,
            layers,
            status_bar: StatusBar::new(),
            action_registry: WizardActionRegistry::new(),
            should_quit: false,
            should_suspend: false,
            ui_mode: UiMode::Normal,
            tick_rate,
            frame_rate,
            action_tx,
            action_rx,
        };

        app.sync_focus_change(None);
        Ok(app)
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            // .mouse(true) // enable when needed
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        tui.enter()?;

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
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
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
                return Ok(());
            }
        };
        let _chord = if mods.is_empty() {
            key_str
        } else {
            format!("{}+{}", mods.join("+"), key_str)
        };

        Ok(())
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            self.handle_single_action(tui, action)?;
        }
        Ok(())
    }

    fn handle_single_action(&mut self, tui: &mut Tui, action: Action) -> Result<()> {
        let status_outcome = self.status_bar.handle_action(&action);
        self.process_component_outcome(status_outcome);

        match action {
            Action::Tick => {
                self.broadcast_action_to_components(&Action::Tick);
                self.layers.tick_notifications();
                self.render(tui)?;
            }
            Action::Quit => self.should_quit = true,
            Action::Suspend => self.should_suspend = true,
            Action::Resume => self.should_suspend = false,
            Action::ClearScreen => tui.clear()?,
            Action::Resize(w, h) => {
                self.handle_resize(tui, w, h)?;
            }
            Action::Render => self.render(tui)?,
            Action::Error(msg) => {
                warn!("UI error: {msg}");
            }
            Action::Help => {
                self.apply_layer_action(LayerAction::ToggleHelp);
                self.render(tui)?;
            }
            Action::Ui(ui) => {
                self.handle_ui_action(ui)?;
                self.render(tui)?;
            }
        }

        Ok(())
    }

    fn handle_ui_action(&mut self, ui: UiAction) -> Result<()> {
        match ui {
            UiAction::FocusNext => {
                self.apply_layer_action(LayerAction::FocusNext);
                return Ok(());
            }
            UiAction::FocusPrev => {
                self.apply_layer_action(LayerAction::FocusPrev);
                return Ok(());
            }
            _ => {}
        }

        let consumed = self.dispatch_to_focused_component(&Action::Ui(ui.clone()));

        match ui {
            UiAction::ToggleEditMode => {
                self.ui_mode = match self.ui_mode {
                    UiMode::Normal => UiMode::Edit,
                    UiMode::Edit => UiMode::Normal,
                };
            }
            UiAction::EnterEditMode => {
                self.ui_mode = UiMode::Edit;
            }
            UiAction::ExitEditMode => {
                self.ui_mode = UiMode::Normal;
            }
            _ => {}
        }

        if consumed {
            return Ok(());
        }

        Ok(())
    }

    fn dispatch_to_focused_component(&mut self, action: &Action) -> bool {
        let Some(key) = self.layers.active.focus.component else {
            return false;
        };

        let outcome = {
            let component = self.layers.components.get_mut(key);
            component.handle_action(action)
        };

        self.process_component_outcome(outcome)
    }

    fn broadcast_action_to_components(&mut self, action: &Action) {
        let keys: Vec<ComponentKey> = self
            .layers
            .components
            .items
            .iter()
            .map(|(key, _)| key)
            .collect();

        for key in keys {
            let outcome = {
                let component = self.layers.components.get_mut(key);
                component.handle_action(action)
            };
            self.process_component_outcome(outcome);
        }
    }

    fn process_component_outcome(&mut self, outcome: ActionOutcome) -> bool {
        match outcome {
            ActionOutcome::Consumed => true,
            ActionOutcome::NotHandled => false,
            ActionOutcome::RequestFocus(key) => {
                self.focus_component(key);
                true
            }
            ActionOutcome::Emit() => true,
        }
    }

    fn apply_layer_action(&mut self, action: LayerAction) {
        let previous = self.layers.active.focus.component;
        self.layers.apply(action);
        self.layers.active.focus.surface = self
            .layers
            .active
            .popup
            .map(layers::Surface::Popup)
            .or_else(|| self.layers.active.page.map(layers::Surface::Page));
        self.sync_focus_change(previous);
    }

    fn focus_component(&mut self, key: ComponentKey) {
        let previous = self.layers.active.focus.component;
        self.layers.focus_component(key);
        self.layers.active.focus.surface = self
            .layers
            .active
            .popup
            .map(layers::Surface::Popup)
            .or_else(|| self.layers.active.page.map(layers::Surface::Page));
        self.sync_focus_change(previous);
    }

    fn sync_focus_change(&mut self, previous: Option<ComponentKey>) {
        let current = self.layers.active.focus.component;
        if current == previous {
            return;
        }

        if let Some(prev) = previous {
            if let Some(component) = self.layers.components.items.get_mut(prev) {
                component.on_focus(false);
            }
        }

        if let Some(cur) = current {
            if let Some(component) = self.layers.components.items.get_mut(cur) {
                component.on_focus(true);
            }
        }
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        let plan = self.layers.render_plan();
        tui.draw(|frame| {
            let area = frame.area();
            let status_h: u16 = 1;
            let (main_area, status_area) = if area.height > status_h {
                (
                    Rect::new(area.x, area.y, area.width, area.height - status_h),
                    Rect::new(
                        area.x,
                        area.y + area.height - status_h,
                        area.width,
                        status_h,
                    ),
                )
            } else {
                (area, area)
            };

            if let Some(page) = plan.page {
                self.render_page(frame, main_area, page);
            }

            if let Some(popup) = plan.popup {
                self.render_popup(frame, main_area, popup);
            }

            self.status_bar.render(frame, status_area);
        })?;
        Ok(())
    }

    fn render_page(&self, frame: &mut ratatui::Frame, area: Rect, page: &layers::page::Page) {
        let slots = (page.layout_any)(area);

        for &component in &page.components {
            let rect = self
                .find_component_rect(&slots, &page.slot_map, component)
                .unwrap_or(area);
            self.layers.components.get(component).render(frame, rect);
        }
    }

    fn render_popup(&self, frame: &mut ratatui::Frame, area: Rect, popup: &layers::popup::Popup) {
        use ratatui::widgets::{Block, Borders, Clear};

        let container = layers::default_popup_layout(area);
        frame.render_widget(Clear, container);
        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .title(popup.meta.title.as_str()),
            container,
        );

        let slots = (popup.layout_any)(container);
        for &component in &popup.components {
            let rect = self
                .find_component_rect(&slots, &popup.slot_map, component)
                .unwrap_or(container);
            self.layers.components.get(component).render(frame, rect);
        }
    }

    fn find_component_rect(
        &self,
        slots: &layers::SlotsAny,
        slot_map: &indexmap::IndexMap<u64, Vec<ComponentKey>>,
        component: ComponentKey,
    ) -> Option<Rect> {
        slot_map.iter().find_map(|(hash, keys)| {
            if keys.iter().any(|&key| key == component) {
                slots.map.get(hash).copied()
            } else {
                None
            }
        })
    }

    fn now_unix_ms(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

/// Implementation des ActionRegistry-Traits für die Wizard-App
impl ActionRegistry for App {
    type Action = Action;

    fn resolve_action(
        &self,
        action_name: &str,
        _action_data: Option<&toml::Value>,
    ) -> Option<Self::Action> {
        debug!("ActionRegistry: Resolving action '{}'", action_name);
        match action_name {
            // Core actions
            "Quit" => Some(Action::Quit),
            "Help" => Some(Action::Help),
            "Tick" => Some(Action::Tick),
            "Render" => Some(Action::Render),
            "ClearScreen" => Some(Action::ClearScreen),
            "Suspend" => Some(Action::Suspend),
            "Resume" => Some(Action::Resume),

            // UI focus & navigation
            "FocusNext" | "NextField" => Some(Action::Ui(UiAction::FocusNext)),
            "FocusPrev" | "PrevField" | "PreviousField" => Some(Action::Ui(UiAction::FocusPrev)),
            "NavigateUp" => Some(Action::Ui(UiAction::NavigateUp)),
            "NavigateDown" => Some(Action::Ui(UiAction::NavigateDown)),
            "NavigateLeft" => Some(Action::Ui(UiAction::NavigateLeft)),
            "NavigateRight" => Some(Action::Ui(UiAction::NavigateRight)),
            "ActivateSelected" => Some(Action::Ui(UiAction::ActivateSelected)),

            // Edit mode
            "ToggleEditMode" | "ModeCycle" => Some(Action::Ui(UiAction::ToggleEditMode)),
            "EnterEditMode" | "ModeInsert" => Some(Action::Ui(UiAction::EnterEditMode)),
            "ExitEditMode" | "ModeNormal" => Some(Action::Ui(UiAction::ExitEditMode)),

            _ => None,
        }
    }

    fn get_action_names(&self) -> Vec<String> {
        vec![
            // Core
            "Quit".into(),
            "Help".into(),
            "Tick".into(),
            "Render".into(),
            "ClearScreen".into(),
            "Suspend".into(),
            "Resume".into(),
            // UI Navigation
            "FocusNext".into(),
            "NextField".into(),
            "FocusPrev".into(),
            "PrevField".into(),
            "PreviousField".into(),
            // Component Navigation
            "NavigateUp".into(),
            "NavigateDown".into(),
            "NavigateLeft".into(),
            "NavigateRight".into(),
            "ActivateSelected".into(),
            // UI Mode
            "ToggleEditMode".into(),
            "ModeCycle".into(),
            "EnterEditMode".into(),
            "ModeInsert".into(),
            "ExitEditMode".into(),
            "ModeNormal".into(),
        ]
    }
}
