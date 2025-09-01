use color_eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
};

use crate::{action::Action, tui::Event, tui::EventResponse};

pub mod logo;
pub mod welcome;

/// `Component` is a trait that represents a visual and interactive element of the user interface.
///
/// Implementors of this trait can be registered with the main application loop and will be able to
/// receive events, update state, and be rendered on the screen.
pub trait Component {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn height_constraint(&self) -> Constraint;

    /// Human-readable component name (for diagnostics/logs)
    fn name(&self) -> &'static str {
        "component"
    }

    /// Keymap context used to look up shortcuts for this component
    fn keymap_context(&self) -> &'static str {
        "global"
    }

    fn handle_events(&mut self, event: Event) -> Result<Option<EventResponse<Action>>> {
        let r = match event {
            Event::Key(key_event) => self.handle_key_events(key_event)?,
            Event::Mouse(mouse_event) => self.handle_mouse_events(mouse_event)?,
            _ => None,
        };
        Ok(r)
    }

    fn handle_key_events(&mut self, _key: KeyEvent) -> Result<Option<EventResponse<Action>>> {
        Ok(None)
    }

    fn handle_mouse_events(&mut self, _mouse: MouseEvent) -> Result<Option<EventResponse<Action>>> {
        Ok(None)
    }

    fn update(&mut self, _action: Action) -> Result<Option<Action>> {
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()>;
}
