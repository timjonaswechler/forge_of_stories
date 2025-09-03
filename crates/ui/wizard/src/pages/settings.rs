use crate::{
    action::Action,
    components::{
        Component,
        settings_categories::{Category, SettingsCategoriesComponent},
        settings_details::SettingsDetailsComponent,
    },
};
use aether_config::build_server_settings_store;
use color_eyre::Result;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};
use settings::SettingsStore;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use super::Page;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Left,
    Right,
}

pub struct SettingsPage {
    command_tx: Option<UnboundedSender<Action>>,
    server_store: Arc<SettingsStore>,
    left: SettingsCategoriesComponent,
    right: SettingsDetailsComponent,
    focus: Focus,
}

impl SettingsPage {
    pub fn new() -> Result<Self> {
        let store = Arc::new(build_server_settings_store()?);

        let left = SettingsCategoriesComponent::new();
        let mut right = SettingsDetailsComponent::new();
        right.set_store(store.clone());
        right.set_from_server(Category::General, &store)?;

        Ok(Self {
            command_tx: None,
            server_store: store,

            left,
            right,
            focus: Focus::Left,
        })
    }
}

impl Page for SettingsPage {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn handle_events(
        &mut self,
        event: crate::tui::Event,
    ) -> Result<Option<crate::tui::EventResponse<Action>>> {
        match self.focus {
            Focus::Left => self.left.handle_events(event),
            Focus::Right => self.right.handle_events(event),
        }
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::FocusNext => {
                self.focus = match self.focus {
                    Focus::Right => Focus::Left,
                    Focus::Left => Focus::Right,
                };
            }
            Action::FocusPrev => {
                self.focus = match self.focus {
                    Focus::Right => Focus::Left,
                    Focus::Left => Focus::Right,
                };
            }
            // Forward input mode toggles and submits to the right component
            Action::Submit | Action::SwitchInputMode => {
                if let Some(a) = self.right.update(action.clone())? {
                    return Ok(Some(a));
                }
            }
            Action::Up | Action::Down => {
                match self.focus {
                    Focus::Left => {
                        // navigiere links und aktualisiere rechts
                        self.left.update(action.clone())?;
                        let cat = self.left.selected();
                        self.right.set_store(self.server_store.clone());
                        self.right.set_from_server(cat, &self.server_store)?;
                    }
                    Focus::Right => {
                        if let Some(a) = self.right.update(action.clone())? {
                            return Ok(Some(a));
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        self.left.draw(frame, chunks[0])?;
        self.right.draw(frame, chunks[1])?;
        Ok(())
    }

    fn keymap_context(&self) -> &'static str {
        "settings"
    }

    fn focused_component_name(&self) -> &'static str {
        // wichtig: Wizard verwendet diesen Namen als Keymap-Kontext
        "settings"
    }
}
