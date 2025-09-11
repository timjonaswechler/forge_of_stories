use crate::{
    action::{Action, UiAction},
    components::{Component, TaskList},
    pages::Page,
};
use color_eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders},
};
use tokio::sync::mpsc::UnboundedSender;

/// DashboardPage: registers task list component (StatusBar now rendered globally by App).
pub struct DashboardPage {
    tx: Option<UnboundedSender<Action>>,
    focused: Option<String>,
    focusables: [&'static str; 2],
}

impl DashboardPage {
    pub fn new() -> Self {
        Self {
            tx: None,
            focused: None,
            focusables: ["tasks", "fps_panel"],
        }
    }
}

impl Page for DashboardPage {
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

    fn unfocus(&mut self) -> Result<()> {
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

    fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        let block = Block::default().borders(Borders::ALL).title("Dashboard");
        f.render_widget(block, area);
        Ok(())
    }
}
