use color_eyre::Result;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect, Size},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, components::Component, config::Config, tui::Event};

mod home;
mod login;

pub use home::HomePage;
pub use login::LoginPage;

/// A `Page` composes multiple `Component`s and exposes a lifecycle similar to the
/// existing `Component` trait but at the page level.
pub trait Page {
    fn name(&self) -> &str;

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        let _ = tx;
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        let _ = config;
        Ok(())
    }

    fn init(&mut self, area: Size) -> Result<()> {
        let _ = area;
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let _ = event;
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        let _ = action;
        Ok(None)
    }

    /// Draw the page using the provided `Frame` and `area`.
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()>;

    /// Called when the page becomes active.
    fn on_enter(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when the page is leaving / being replaced.
    fn on_exit(&mut self) -> Result<()> {
        Ok(())
    }
}

/// `SimplePage` is a convenience implementation of `Page` that delegates most
/// operations to an inner list of `Component`s. It's useful for pages that are
/// just a composition of components without additional page-level logic.
#[derive(Default)]
pub struct SimplePage {
    pub id: String,
    pub components: Vec<Box<dyn Component>>,
    initialized: bool,
}

impl SimplePage {
    pub fn new(id: impl Into<String>, components: Vec<Box<dyn Component>>) -> Self {
        Self {
            id: id.into(),
            components,
            initialized: false,
        }
    }
}

impl Page for SimplePage {
    fn name(&self) -> &str {
        &self.id
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        for c in self.components.iter_mut() {
            let _ = c.register_action_handler(tx.clone())?;
        }
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        for c in self.components.iter_mut() {
            let _ = c.register_config_handler(config.clone())?;
        }
        Ok(())
    }

    fn init(&mut self, area: Size) -> Result<()> {
        if !self.initialized {
            for c in self.components.iter_mut() {
                let _ = c.init(area)?;
            }
            self.initialized = true;
        }
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        // Propagate the event to each component. If one returns an action, return it.
        for c in self.components.iter_mut() {
            if let Some(action) = c.handle_events(event.clone())? {
                return Ok(Some(action));
            }
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        for c in self.components.iter_mut() {
            if let Some(action) = c.update(action.clone())? {
                return Ok(Some(action));
            }
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Split the area vertically among components evenly.
        let n = self.components.len();
        if n == 0 {
            return Ok(());
        }
        let base = 100u16 / (n as u16);
        let mut constraints: Vec<Constraint> = Vec::with_capacity(n);
        for i in 0..n {
            if i == n - 1 {
                // last one takes the remainder
                let used: u16 = base * (n as u16 - 1);
                let last = 100u16.saturating_sub(used);
                constraints.push(Constraint::Percentage(last));
            } else {
                constraints.push(Constraint::Percentage(base));
            }
        }

        let areas = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(constraints)
            .split(area);

        for (c, a) in self.components.iter_mut().zip(areas.iter()) {
            let rect = *a;
            c.draw(frame, rect)?;
        }

        Ok(())
    }
}
