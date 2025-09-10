use crate::{action::Action, components::Component, tui::Event};
use color_eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect};
use tokio::sync::mpsc::UnboundedSender;

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

    /// The currently focused component id within the page for status/tooling.
    /// Pages should emit UiAction::ReportFocusedComponent to update App focus;
    /// this method is for read-only status (e.g., status bar).
    fn focused_component_id(&self) -> Option<&str> {
        None
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
    fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<()>;
}
