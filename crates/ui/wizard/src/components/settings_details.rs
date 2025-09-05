use aether_config::{
    GeneralCfg, MonitoringCfg, NetworkCfg, SecurityCfg, ServerSettingField, UdsCfg,
    apply_server_setting,
};
use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    symbols::line::ROUNDED,
    widgets::{Block, Borders, List, ListItem, ListState},
};
use settings::{Settings, SettingsStore};
use std::sync::Arc;
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use crate::components::settings_categories::Category;
use crate::{action::Action, components::Component};

pub struct SettingsDetailsComponent {
    title: String,
    entries: Vec<(String, String)>,
    state: ListState,
    // Inline-Editing
    store: Option<Arc<SettingsStore>>,
    current_category: Category,
    editing: Option<usize>,
    input: Input,
}

impl SettingsDetailsComponent {
    pub fn new() -> Self {
        Self {
            title: "Settings".to_string(),
            entries: Vec::new(),
            state: ListState::default(),
            store: None,
            current_category: Category::General,
            editing: None,
            input: Input::default(),
        }
    }

    pub fn set_store(&mut self, store: Arc<SettingsStore>) {
        self.store = Some(store);
    }

    pub fn selected_field(&self) -> Option<ServerSettingField> {
        let idx = self.state.selected()?;
        self.field_for(self.current_category, idx)
    }

    pub fn current_category(&self) -> Category {
        self.current_category
    }
    fn select_up(&mut self) {
        let i = self.state.selected().unwrap_or(0);
        self.state.select(Some(i.saturating_sub(1)));
    }
    fn select_down(&mut self) {
        let i = self.state.selected().unwrap_or(0);
        let max = self.entries.len().saturating_sub(1);
        self.state.select(Some((i + 1).min(max)));
    }

    fn start_editing(&mut self) {
        if let Some(i) = self.state.selected() {
            self.editing = Some(i);
            self.input = Input::default();
            if let Some((_k, v)) = self.entries.get(i) {
                self.input = self.input.clone().with_value(v.clone());
            }
        }
    }

    pub fn selected_entry_label(&self) -> Option<&str> {
        let idx = self.state.selected()?;
        self.entries.get(idx).map(|(k, _)| k.as_str())
    }
    pub fn set_from_server(&mut self, cat: Category, store: &SettingsStore) -> Result<()> {
        self.current_category = cat;
        self.title = cat.title().to_string();
        // Reset editing state when switching categories
        self.editing = None;
        self.input = Input::default();
        self.entries = match cat {
            Category::General => {
                let g = store.get::<aether_config::General>()?;
                vec![
                    ("Tick Rate".into(), g.tick_rate.to_string()),
                    ("Autostart".into(), g.autostart.to_string()),
                ]
            }
            Category::Network => {
                let n = store.get::<aether_config::Network>()?;
                vec![
                    ("IP Address".into(), n.ip_address.clone()),
                    ("UDP Port".into(), n.udp_port.to_string()),
                    (
                        "Max Concurrent Bidi Streams".into(),
                        n.max_concurrent_bidi_streams.to_string(),
                    ),
                ]
            }
            Category::Security => {
                let s = store.get::<aether_config::Security>()?;
                vec![
                    ("tls_cert_path".into(), s.tls_cert_path.clone()),
                    ("create a self-signed certificate".into(), "".to_string()),
                ]
            }
            Category::Monitoring => {
                let m = store.get::<aether_config::Monitoring>()?;
                vec![("metrics_enabled".into(), m.metrics_enabled.to_string())]
            }
            Category::Uds => {
                let u = store.get::<aether_config::Uds>()?;
                vec![("path".into(), u.path.clone())]
            }
        };
        if !self.entries.is_empty() && self.state.selected().is_none() {
            self.state.select(Some(0));
        }
        Ok(())
    }

    fn commit_edit(&mut self) -> Result<()> {
        if let Some(idx) = self.editing {
            if let Some(store) = self.store.clone() {
                if let Some(field) = self.field_for(self.current_category, idx) {
                    apply_server_setting(&store, field, self.input.value())?;
                }
                self.set_from_server(self.current_category, &store)?;
            }
            self.editing = None;
        }
        Ok(())
    }

    fn field_for(&self, cat: Category, index: usize) -> Option<ServerSettingField> {
        match cat {
            Category::General => match index {
                0 => Some(ServerSettingField::GeneralTickRate),
                1 => Some(ServerSettingField::GeneralAutostart),
                _ => None,
            },
            Category::Network => match index {
                0 => Some(ServerSettingField::NetworkIpAddress),
                1 => Some(ServerSettingField::NetworkUdpPort),
                _ => None,
            },
            Category::Security => match index {
                0 => Some(ServerSettingField::SecurityTlsCertPath),
                _ => None,
            },
            Category::Monitoring => match index {
                0 => Some(ServerSettingField::MonitoringMetricsEnabled),
                _ => None,
            },
            Category::Uds => match index {
                0 => Some(ServerSettingField::UdsPath),
                _ => None,
            },
        }
    }
}

impl Component for SettingsDetailsComponent {
    fn height_constraint(&self) -> Constraint {
        Constraint::Fill(1)
    }

    fn name(&self) -> &'static str {
        "settings_right"
    }

    fn handle_key_events(
        &mut self,
        key: KeyEvent,
    ) -> Result<Option<crate::tui::EventResponse<Action>>> {
        if self.editing.is_some() {
            // Route alle Tastendrücke an das Input-Feld
            self.input.handle_event(&crossterm::event::Event::Key(key));
            return Ok(Some(crate::tui::EventResponse::Continue(Action::Update)));
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Submit | Action::SwitchInputMode => {
                if self.editing.is_some() {
                    if let Err(e) = self.commit_edit() {
                        return Ok(Some(Action::Error(format!(
                            "Failed to apply setting: {}",
                            e
                        ))));
                    }
                } else {
                    self.start_editing();
                }
            }
            Action::Up => {
                if self.editing.is_none() {
                    self.select_up()
                }
            }
            Action::Down => {
                if self.editing.is_none() {
                    self.select_down()
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let items: Vec<ListItem> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, (k, v))| {
                let display_v = if self.editing == Some(i) {
                    self.input.value().to_string()
                } else {
                    v.clone()
                };
                ListItem::new(format!("{k}: {display_v}"))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(self.title.clone())
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Thick),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED));

        f.render_stateful_widget(list, area, &mut self.state);
        Ok(())
    }
}
