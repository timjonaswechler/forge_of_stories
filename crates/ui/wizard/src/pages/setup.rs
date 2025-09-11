use crate::{
    action::{Action, UiAction},
    components::{Component, LayerOverlay, StatusBar},
    layers::ToastManager,
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

/// SetupPage: registers status bar, task list, and a basic overlay component.
/// It reports its focused component via UiAction::ReportFocusedComponent.
pub struct SetupPage {
    tx: Option<UnboundedSender<Action>>,
    focused: Option<String>,
}

impl SetupPage {
    pub fn new() -> Self {
        Self {
            tx: None,
            focused: Some("status".to_string()),
        }
    }
}

impl Page for SetupPage {
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
                "status".to_string(),
                Box::new(StatusBar::new("Setup")) as Box<dyn Component>,
            ),
            // Toasts: render notifications with severity styles; layered via LayerKind::Notification sentinel.
            (
                "toasts".to_string(),
                Box::new(ToastManager::new()) as Box<dyn Component>,
            ),
            (
                "overlay".to_string(),
                Box::new(LayerOverlay::new("setup_help")) as Box<dyn Component>,
            ),
        ]
    }

    fn focus(&mut self) -> Result<()> {
        if let Some(tx) = &self.tx {
            let _ = tx.send(Action::Ui(UiAction::ReportFocusedComponent(
                "status".to_string(),
            )));
        }
        Ok(())
    }

    fn unfocus(&mut self) -> Result<()> {
        Ok(())
    }

    fn keymap_context(&self) -> &'static str {
        "setup"
    }

    fn id(&self) -> &'static str {
        "setup"
    }

    fn focus_order(&self) -> &'static [&'static str] {
        // Order of focus traversal for this page (exclude non-focusable like toasts)
        &["status", "overlay"]
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
        let block = Block::default().borders(Borders::ALL).title("Setup");
        f.render_widget(block, area);
        Ok(())
    }
}
