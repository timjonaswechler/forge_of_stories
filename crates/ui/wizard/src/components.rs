use std::sync::Arc;

use color_eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Rect, Size},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, app::settings::SettingsStore, tui::Event};

pub mod fps;
pub mod home;
mod logo;
mod status_bar;
mod task_list;

pub(crate) use logo::{LogoComponent, WizardLogoComponent};
pub(crate) use status_bar::StatusBar;
pub(crate) use task_list::TaskList;

/// `Component` is a trait that represents a visual and interactive element of the user interface.
///
/// Implementors of this trait can be registered with the main application loop and will be able to
/// receive events, update state, and be rendered on the screen.
pub trait Component {
    /// Register the action sender so the component can emit actions back to the app.
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        let _ = tx; // to appease clippy
        Ok(())
    }

    /// Register a shared settings store that components can read from and (optionally) write to.
    ///
    /// Note: This uses `Arc<SettingsStore>` to avoid cloning the store and to keep a single shared
    /// view across components.
    fn register_settings_handler(&mut self, settings: Arc<SettingsStore>) -> Result<()> {
        let _ = settings; // to appease clippy
        Ok(())
    }

    /// Called once when the component is created to provide initial terminal size.
    fn init(&mut self, area: Size) -> Result<()> {
        let _ = area; // to appease clippy
        Ok(())
    }

    /// Inform the component that its focus state changed (true = focused).
    /// Default: no-op. Override to store focus state for custom rendering.
    fn set_focused(&mut self, focused: bool) {
        let _ = focused; // default no-op
    }

    /// Route a high-level TUI event to this component.
    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let action = match event {
            Some(Event::Key(key_event)) => self.handle_key_event(key_event)?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_event(mouse_event)?,
            _ => None,
        };
        Ok(action)
    }

    /// Handle a key event. Return an Action to be dispatched if appropriate.
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        let _ = key; // to appease clippy
        Ok(None)
    }

    /// Handle a mouse event. Return an Action to be dispatched if appropriate.
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        let _ = mouse; // to appease clippy
        Ok(None)
    }

    /// Update this component in response to a dispatched action.
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let _ = action; // to appease clippy
        Ok(None)
    }

    /// Draw this component within the provided area.
    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> Result<()>;
}
