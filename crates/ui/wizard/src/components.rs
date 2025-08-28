use color_eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
};

use crate::{action::Action, state::State, tui::Event, tui::EventResponse};

pub mod auth;
pub mod footer;
pub mod logo;

/// `Component` is a trait that represents a visual and interactive element of the user interface.
///
/// Implementors of this trait can be registered with the main application loop and will be able to
/// receive events, update state, and be rendered on the screen.
pub trait Component {
    fn init(&mut self, _state: &State) -> Result<()> {
        Ok(())
    }

    fn height_constraint(&self) -> Constraint;

    fn handle_events(
        &mut self,
        event: Event,
        state: &mut State,
    ) -> Result<Option<EventResponse<Action>>> {
        let r = match event {
            Event::Key(key_event) => self.handle_key_events(key_event, state)?,
            Event::Mouse(mouse_event) => self.handle_mouse_events(mouse_event, state)?,
            _ => None,
        };
        Ok(r)
    }

    fn handle_key_events(
        &mut self,
        _key: KeyEvent,
        _state: &mut State,
    ) -> Result<Option<EventResponse<Action>>> {
        Ok(None)
    }

    fn handle_mouse_events(
        &mut self,
        _mouse: MouseEvent,
        _state: &mut State,
    ) -> Result<Option<EventResponse<Action>>> {
        Ok(None)
    }

    fn update(&mut self, _action: Action, _state: &mut State) -> Result<Option<Action>> {
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect, state: &State) -> Result<()>;
}
