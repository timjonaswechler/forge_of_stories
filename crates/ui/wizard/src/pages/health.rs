use color_eyre::Result;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::Action,
    components::{Component, logo::LogoComponent},
};

use super::Page;

/// LoginPage shows the authentication screen:
/// - Left side: username/password form (create-admin on first run, otherwise login)
/// - Right side: ASCII logo
pub struct HealthPage {
    command_tx: Option<UnboundedSender<Action>>,
    components: Vec<Box<dyn Component>>,
    focused_component_index: usize,
}

impl HealthPage {
    pub fn new() -> Result<Self> {
        Ok(Self {
            command_tx: None,
            components: vec![Box::new(LogoComponent::new())],
            focused_component_index: 0,
        })
    }
}

impl Page for HealthPage {
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
        // Only show top info when we actually timed out
        if let Action::IdleTimeout = action {}
        for component in self.components.iter_mut() {
            component.update(action.clone())?;
        }
        Ok(None)
    }

    fn keymap_context(&self) -> &'static str {
        "health"
    }

    fn focused_component_name(&self) -> &'static str {
        "logo"
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

        self.components[0].draw(frame, part[1])?;

        Ok(())
    }
}
