use crate::{
    action::{Action, PopupResult},
    components::popups::certificate_wizard_popup,
    components::{
        Component,
        popups::bool_choice::BoolChoicePopup,
        settings_categories::{Category, SettingsCategoriesComponent},
        settings_details::SettingsDetailsComponent,
    },
};
use aether_config::{ServerSettingField, apply_server_setting, build_server_settings_store};
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
                // Security: "create a self-signed certificate" → Form-Popup öffnen
                if self.right.current_category() == Category::Security {
                    if let Some(label) = self.right.selected_entry_label() {
                        if label == "create a self-signed certificate" {
                            let popup = certificate_wizard_popup();
                            return Ok(Some(Action::OpenPopup(Box::new(popup))));
                        }
                    }
                }

                // General/Autostart → Bool-Choice-Popup (wie gehabt)
                if self.right.current_category() == Category::General {
                    if let Some(field) = self.right.selected_field() {
                        if matches!(field, ServerSettingField::GeneralAutostart) {
                            let popup = BoolChoicePopup::new("Autostart")
                                .question("Enable autostart for the server?")
                                .true_label("On")
                                .false_label("Off");
                            return Ok(Some(Action::OpenPopup(Box::new(popup))));
                        }
                    }
                }

                if let Some(a) = self.right.update(action)? {
                    return Ok(Some(a));
                }
            }
            Action::Up | Action::Down => {
                match self.focus {
                    Focus::Left => {
                        // navigiere links und aktualisiere rechts
                        self.left.update(action)?;

                        let cat = self.left.selected();
                        self.right.set_store(self.server_store.clone());
                        self.right.set_from_server(cat, &self.server_store)?;
                    }
                    Focus::Right => {
                        if let Some(a) = self.right.update(action)? {
                            return Ok(Some(a));
                        }
                    }
                }
            }
            Action::PopupResult(PopupResult::InputSubmitted(val)) => {
                // Apply Autostart only if the current selection is the Autostart field
                if self.right.current_category() == Category::General {
                    if let Some(field) = self.right.selected_field() {
                        if matches!(field, ServerSettingField::GeneralAutostart) {
                            let s = if val == "true" { "true" } else { "false" }.to_string();
                            apply_server_setting(
                                &self.server_store,
                                ServerSettingField::GeneralAutostart,
                                &s,
                            )?;
                            // Refresh the right pane from store
                            let cat = self.left.selected();
                            self.right.set_store(self.server_store.clone());
                            self.right.set_from_server(cat, &self.server_store)?;
                        }
                    }
                }
            }
            Action::PopupResult(PopupResult::Cancelled) => {
                // No-op for cancel
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
