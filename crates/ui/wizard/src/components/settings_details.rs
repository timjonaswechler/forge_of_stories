use color_eyre::Result;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    symbols::line::{ROUNDED, THICK_VERTICAL_RIGHT},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::components::settings_categories::Category;
use crate::{action::Action, components::Component};
use aether_config::ServerSettings;

pub struct SettingsDetailsComponent {
    title: String,
    entries: Vec<(String, String)>,
    state: ListState,
}

impl SettingsDetailsComponent {
    pub fn new() -> Self {
        Self {
            title: "Einstellungen".to_string(),
            entries: Vec::new(),
            state: ListState::default(),
        }
    }

    pub fn set_from_server(&mut self, cat: Category, server: &ServerSettings) {
        self.title = cat.title().to_string();
        self.entries = match cat {
            Category::General => vec![
                ("Tick Rate".into(), server.general.tick_rate.to_string()),
                ("Autostart".into(), server.general.autostart.to_string()),
            ],
            Category::Network => vec![
                ("IP Address".into(), server.network.ip_address.clone()),
                ("UDP Port".into(), server.network.udp_port.to_string()),
            ],
            Category::Security => vec![(
                "tls_cert_path".into(),
                server.security.tls_cert_path.clone(),
            )],
            Category::Monitoring => vec![(
                "metrics_enabled".into(),
                server.monitoring.metrics_enabled.to_string(),
            )],
            Category::Uds => vec![("path".into(), server.uds.path.clone())],
        };
        if !self.entries.is_empty() && self.state.selected().is_none() {
            self.state.select(Some(0));
        }
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
}

impl Component for SettingsDetailsComponent {
    fn height_constraint(&self) -> Constraint {
        Constraint::Fill(1)
    }

    fn name(&self) -> &'static str {
        "settings_right"
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Up => self.select_up(),
            Action::Down => self.select_down(),
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let items: Vec<ListItem> = self
            .entries
            .iter()
            .map(|(k, v)| ListItem::new(format!("{k}: {v}")))
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
