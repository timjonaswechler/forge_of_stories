//! Dashboard2Page: Reference page demonstrating multi-component layout with Page::layout().
//!
//! Purpose:
//! - Show how to place several components (logo, task list, fps panel, spare panel)
//!   using the new Page::layout(PageLayout) API.
//! - Provide a richer focus cycle example (tasks -> fps).
//! - Serve as a template for future complex pages (split/grid composition).
//!
//! Integration:
//! This page is not yet wired into `App` (CLI route still selects `DashboardPage` / `SetupPage`).
//! To experiment, add it to the pages vector in `App::new` and adjust navigation logic.
//!
//! Layout strategy:
//! ┌─────────────────────────────────────────────┐
//! │ Header / Wizard Logo (fixed height ~9)      │  <- wizard_logo
//! ├─────────────────────────────────────────────┤
//! │ Tasks (flex)      │  FPS / Stats (fixed)    │  <- tasks | fps_panel
//! ├───────────────────┴─────────────────────────┤
//! │ (status bar global; not part of this page)  │
//! └─────────────────────────────────────────────┘
//!
//! Component IDs:
//! - "wizard_logo"
//! - "tasks"
//! - "fps_panel"
//!
//! Focus order:
//! - tasks
//! - fps_panel
//!
//! The logo is not focusable.
//!
use crate::{
    action::{Action, UiAction},
    components::{Component, TaskList, WizardLogoComponent},
    pages::{Page, PageLayout},
};
use color_eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
};
use tokio::sync::mpsc::UnboundedSender;

/// Optional lightweight FPS panel (inline implementation).
/// Demonstrates a second focusable component with custom rendering.
pub struct SimpleFpsPanel {
    fps: f32,
    focused: bool,
}
impl SimpleFpsPanel {
    pub fn new() -> Self {
        Self {
            fps: 0.0,
            focused: false,
        }
    }
    fn set_fps(&mut self, fps: f32) {
        self.fps = fps;
    }
}
impl Component for SimpleFpsPanel {
    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }
    fn update(&mut self, _action: Action) -> Result<Option<Action>> {
        // In a real implementation you would listen for a Tick action and
        // update internal rolling average FPS derived from settings or app state.
        Ok(None)
    }
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let title = if self.focused { "FPS (Focused)" } else { "FPS" };
        let block = Block::default().title(title).borders(Borders::ALL);
        // Just render placeholder value
        f.render_widget(block, area);
        Ok(())
    }
}

pub struct Dashboard2Page {
    tx: Option<UnboundedSender<Action>>,
    focused: Option<String>,
    // Local mirror of focusable component ids
    focusables: [&'static str; 2],
}

impl Dashboard2Page {
    pub fn new() -> Self {
        Self {
            tx: None,
            focused: Some("tasks".to_string()),
            focusables: ["tasks", "fps_panel"],
        }
    }
}

impl Page for Dashboard2Page {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn provide_components(&mut self) -> Vec<(String, Box<dyn Component>)> {
        vec![
            (
                "wizard_logo".to_string(),
                Box::new(WizardLogoComponent::new()) as Box<dyn Component>,
            ),
            (
                "tasks".to_string(),
                Box::new(TaskList::new()) as Box<dyn Component>,
            ),
            (
                "fps_panel".to_string(),
                Box::new(SimpleFpsPanel::new()) as Box<dyn Component>,
            ),
        ]
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

    fn unfocus(&mut self) -> Result<()> {
        Ok(())
    }

    fn keymap_context(&self) -> &'static str {
        "dashboard"
    }

    fn id(&self) -> &'static str {
        "dashboard2"
    }

    fn focus_order(&self) -> &'static [&'static str] {
        &["tasks", "fps_panel"]
    }

    fn focused_component_id(&self) -> Option<&str> {
        self.focused.as_deref()
    }

    fn handle_key_events(&mut self, _key: KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }

    fn handle_mouse_events(&mut self, _mouse: MouseEvent) -> Result<Option<Action>> {
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::Ui(UiAction::ReportFocusedComponent(id)) = action {
            self.focused = Some(id);
        }
        Ok(None)
    }

    fn layout(&self, area: Rect) -> PageLayout {
        // First vertical split: header (logo) vs main content
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(9), // wizard logo height (WizardLogoComponent::size().1 + padding)
                Constraint::Min(3),
            ])
            .split(area);
        let header = vertical[0];
        let body = vertical[1];

        // Body: two columns (tasks grows, fps panel fixed width)
        let body_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Min(20)])
            .split(body);
        let tasks_area = body_cols[0];
        let fps_area = body_cols[1];

        PageLayout::empty()
            .with("wizard_logo", header)
            .with("tasks", tasks_area)
            .with("fps_panel", fps_area)
    }

    fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        let block = Block::default()
            .title("Dashboard2 (reference multi-component layout)")
            .borders(Borders::ALL);
        f.render_widget(block, area);
        Ok(())
    }
}
