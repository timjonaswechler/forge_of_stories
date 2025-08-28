use color_eyre::Result;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    components::{Component, auth::AuthComponent, logo::LogoComponent},
    config::Config,
    state::State,
};

use super::Page;

/// LoginPage shows the authentication screen:
/// - Left side: username/password form (create-admin on first run, otherwise login)
/// - Right side: ASCII logo
pub struct LoginPage {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    components: Vec<Box<dyn Component>>,
    focused_pane_index: usize,
}

impl LoginPage {
    pub fn new() -> Result<Self> {
        Ok(Self {
            command_tx: None,
            config: Config::default(),
            components: vec![
                Box::new(AuthComponent::new()),
                Box::new(LogoComponent::new()),
            ],
            focused_pane_index: 1,
        })
    }
}

impl Page for LoginPage {
    fn init(&mut self, state: &State) -> Result<()> {
        for pane in self.components.iter_mut() {
            pane.init(state)?;
        }
        Ok(())
    }

    fn on_enter(&mut self, state: &mut State) -> Result<()> {
        state.input_mode = InputMode::Insert;
        Ok(())
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action, state: &mut State) -> Result<Option<Action>> {
        // Only show top info when we actually timed out
        if let Action::IdleTimeout = action {}
        for component in self.components.iter_mut() {
            component.update(action.clone(), state)?;
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, state: &State) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Min(151),
                Constraint::Fill(1),
            ])
            .split(area);

        let part = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(60), Constraint::Min(71)])
            .split(chunks[1]);
        self.components[0].draw(frame, part[0], state)?;
        self.components[1].draw(frame, part[1], state)?;

        Ok(())
    }
}
