use crate::{
    action::{Action, UiAction},
    components::Component,
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

/// SetupPage: base page without per-page StatusBar (now rendered globally in App).
/// Focus handling is trivial (no intrinsic components).
pub struct SetupPage {
    tx: Option<UnboundedSender<Action>>,
    focused: Option<String>,
}

impl SetupPage {
    pub fn new() -> Self {
        Self {
            tx: None,
            focused: None,
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
        Vec::new()
    }

    fn focus(&mut self) -> Result<()> {
        // No page-owned components to focus; status bar handled globally.
        // (Any future focusable components would emit ReportFocusedComponent here.)

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
        &[]
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
        Ok(())
    }
}
