use color_eyre::Result;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect, Size},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    components::{Component, auth::AuthComponent, logo::LogoComponent},
    config::Config,
    tui::Event,
};

use super::Page;

/// LoginPage shows the authentication screen:
/// - Left side: username/password form (create-admin on first run, otherwise login)
/// - Right side: ASCII logo
pub struct LoginPage {
    auth: AuthComponent,
    logo: LogoComponent,
}

impl LoginPage {
    pub fn new() -> Self {
        Self {
            auth: AuthComponent::new(),
            logo: LogoComponent::new(),
        }
    }
}

impl Default for LoginPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for LoginPage {
    fn name(&self) -> &str {
        "login"
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.auth.register_action_handler(tx.clone())?;
        self.logo.register_action_handler(tx)?;
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.auth.register_config_handler(config.clone())?;
        self.logo.register_config_handler(config)?;
        Ok(())
    }
    fn register_shortcuts(
        &self,
    ) -> Option<(&'static str, Box<[crate::services::shortcuts::Shortcut]>)> {
        // Aktuell ist AuthComponent die interaktivste Komponente.
        self.auth.register_shortcuts()
    }

    fn init(&mut self, area: Size) -> Result<()> {
        self.auth.init(area)?;
        self.logo.init(area)?;
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        if let Some(a) = self.auth.handle_events(event.clone())? {
            return Ok(Some(a));
        }
        if let Some(a) = self.logo.handle_events(event)? {
            return Ok(Some(a));
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        // Only show top info when we actually timed out
        if let Action::IdleTimeout = action {}
        if let Some(a) = self.auth.update(action.clone())? {
            return Ok(Some(a));
        }
        if let Some(a) = self.logo.update(action)? {
            return Ok(Some(a));
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // Left: 55% input, Right: 45% ASCII logo
        let chunks = if area.width < 121 {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(16), Constraint::Length(16)])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Max(50),
                    Constraint::Min(LogoComponent::length()),
                ])
                .split(area)
        };
        if area.width < 121 {
            self.logo.draw(frame, chunks[0])?;
            self.auth.draw(frame, chunks[1])?;
        } else {
            self.auth.draw(frame, chunks[0])?;
            self.logo.draw(frame, chunks[1])?;
        }
        Ok(())
    }
}
