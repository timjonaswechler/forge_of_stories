use crate::{
    action::Action,
    components::{Component, logo::LogoComponent, welcome::WelcomeComponent},
};
use color_eyre::Result;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};
use tokio::sync::mpsc::UnboundedSender;

use super::Page;

pub struct SettingsPage {
    command_tx: Option<UnboundedSender<Action>>,
    components: Vec<Box<dyn Component>>,
    focused_component_index: usize,
}

impl SettingsPage {
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

impl Page for SettingsPage {
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
                    component.update(action.clone())?;
                }
                Ok(None)
            }
            Action::Submit => Ok(Some(Action::Navigate(1))),
            _ => Ok(None),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        Ok(())
    }

    fn keymap_context(&self) -> &'static str {
        "settings"
    }

    fn focused_component_name(&self) -> &'static str {
        self.components
            .get(self.focused_component_index)
            .map(|c| c.name())
            .unwrap_or("unknown")
    }
}
