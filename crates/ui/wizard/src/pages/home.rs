use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::{Rect, Size};
use std::{any::Any, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;

use crate::components::Component;
use crate::components::fps::FpsCounter;
use crate::components::home::Home;
use crate::{action::Action, config::Config, style::Theme, tui::Event};

use super::Page;

pub struct HomePage {
    home: Home,
    fps: FpsCounter,
    stats_history: Option<Arc<dyn Any + Send + Sync>>,
}

impl HomePage {
    pub fn new() -> Self {
        Self {
            home: Home::new(),
            fps: FpsCounter::default(),
            stats_history: None,
        }
    }
}

impl Default for HomePage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for HomePage {
    fn name(&self) -> &str {
        "home"
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.home.register_action_handler(tx.clone())?;
        self.fps.register_action_handler(tx)?;
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.home.register_config_handler(config.clone())?;
        self.fps.register_config_handler(config)?;
        Ok(())
    }

    fn register_theme(&mut self, theme: Theme) -> Result<()> {
        self.home.register_theme(theme.clone())?;
        self.fps.register_theme(theme)?;
        Ok(())
    }

    fn register_shared_state(&mut self, state: Arc<dyn Any + Send + Sync>) -> Result<()> {
        self.stats_history = Some(state);
        Ok(())
    }

    fn init(&mut self, area: Size) -> Result<()> {
        self.home.init(area)?;
        self.fps.init(area)?;
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        if let Some(action) = self.home.handle_events(event.clone())? {
            return Ok(Some(action));
        }
        if let Some(action) = self.fps.handle_events(event)? {
            return Ok(Some(action));
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Some(a) = self.home.update(action.clone())? {
            return Ok(Some(a));
        }
        if let Some(a) = self.fps.update(action)? {
            return Ok(Some(a));
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Layout: top row for fps (1 line), rest for home
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(1),
                ratatui::layout::Constraint::Min(0),
            ])
            .split(area);
        self.fps.draw(frame, chunks[0])?;
        self.home.draw(frame, chunks[1])?;
        Ok(())
    }
}
