use color_eyre::eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    tui::{Event, EventResponse, Frame},
};

mod dashboard;
mod health;
mod settings;
mod setup;

pub use dashboard::DashboardPage;
pub use health::HealthPage;
pub use settings::SettingsPage;
pub use setup::SetupPage;

pub trait Page {
    #[allow(unused_variables)]
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        Ok(())
    }

    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn focus(&mut self) -> Result<()> {
        Ok(())
    }

    fn unfocus(&mut self) -> Result<()> {
        Ok(())
    }

    /// Return the active keymap context name used to export shortcuts for this page.
    /// Default is "global"; pages can override with e.g. "setup", "dashboard", "health".
    fn keymap_context(&self) -> &'static str {
        "global"
    }

    /// Return the focused component's name for this page (if any).
    /// Default is "root". Pages with sub-components should override and return the
    /// currently focused component's human-readable name.
    fn focused_component_name(&self) -> &'static str {
        "root"
    }

    fn handle_events(&mut self, event: Event) -> Result<Option<EventResponse<Action>>> {
        let r = match event {
            Event::Key(key_event) => self.handle_key_events(key_event)?,
            Event::Mouse(mouse_event) => self.handle_mouse_events(mouse_event)?,
            _ => None,
        };
        Ok(r)
    }

    #[allow(unused_variables)]
    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<EventResponse<Action>>> {
        Ok(None)
    }

    #[allow(unused_variables)]
    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<EventResponse<Action>>> {
        Ok(None)
    }

    #[allow(unused_variables)]
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()>;
}
