use crate::{
    action::{Action, UiAction},
    components::{Component, WizardLogoComponent},
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

/// WelcomePage: base page without per-page StatusBar (now rendered globally in App) and
/// demonstrates embedding the WizardLogoComponent.
pub struct WelcomePage {
    tx: Option<UnboundedSender<Action>>,
    focused: Option<String>,
    focusables: [&'static str; 2],
}

impl WelcomePage {
    pub fn new() -> Self {
        Self {
            tx: None,
            focused: Some("tasks".to_string()),
            focusables: ["tasks", "fps_panel"],
        }
    }
}

impl Page for WelcomePage {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn provide_components(&mut self) -> Vec<(String, Box<dyn Component>)> {
        vec![(
            "wizard_logo".to_string(),
            Box::new(WizardLogoComponent::new()) as Box<dyn Component>,
        )]
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
        "welcome"
    }

    fn id(&self) -> &'static str {
        "welcome"
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
        // let block = Block::default().borders(Borders::ALL).title("Affe");
        // f.render_widget(block, area);
        Ok(())
    }
}
