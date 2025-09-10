use color_eyre::Result;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    components::{Component, logo::LogoComponent, welcome::WelcomeComponent},
};

use super::Page;

/// LoginPage shows the authentication screen:
/// - Left side: username/password form (create-admin on first run, otherwise login)
/// - Right side: ASCII logo
pub struct SetupPage {
    command_tx: Option<UnboundedSender<Action>>,
    components: Vec<Box<dyn Component>>,
    focused_component_index: usize,
}

impl SetupPage {
    pub fn new() -> Result<Self> {
        Ok(Self {
            command_tx: None,
            components: vec![
                Box::new(WelcomeComponent::new()),
                Box::new(LogoComponent::new()),
            ],
            focused_component_index: 0,
        })
    }
}

impl Page for SetupPage {
    fn init(&mut self) -> Result<()> {
        for pane in self.components.iter_mut() {
            pane.init()?;
        }
        Ok(())
    }

    fn handle_events(
        &mut self,
        event: crate::tui::Event,
    ) -> Result<Option<crate::tui::EventResponse<Action>>> {
        if let Some(r) = self.components[self.focused_component_index].handle_events(event)? {
            return Ok(Some(r));
        }
        Ok(None)
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::FocusNext | Action::Down => {
                if !self.components.is_empty() {
                    self.focused_component_index =
                        (self.focused_component_index + 1) % self.components.len();
                }
                Ok(None)
            }
            Action::FocusPrev | Action::Up => {
                if !self.components.is_empty() {
                    if self.focused_component_index == 0 {
                        self.focused_component_index = self.components.len() - 1;
                    } else {
                        self.focused_component_index -= 1;
                    }
                }
                Ok(None)
            }
            Action::PreflightResults(_) => {
                for component in self.components.iter_mut() {
                    component.update(Action::Update)?;
                }
                Ok(None)
            }
            Action::Submit => Ok(None),
            _ => Ok(None),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
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
        self.components[0].draw(frame, part[0])?;
        self.components[1].draw(frame, part[1])?;

        Ok(())
    }

    fn keymap_context(&self) -> &'static str {
        "setup"
    }

    fn focused_component_name(&self) -> &'static str {
        self.components
            .get(self.focused_component_index)
            .map(|c| c.name())
            .unwrap_or("unknown")
    }
}
