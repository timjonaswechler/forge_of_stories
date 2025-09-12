use crate::{
    action::{Action, UiAction},
    components::{Component, WizardLogoComponent},
    pages::{Page, PageLayout},
};
use color_eyre::Result;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph},
};

/// WelcomePage: base page showing logo and a simple aether settings status list.
pub struct WelcomePage {
    tx: Option<tokio::sync::mpsc::UnboundedSender<Action>>,
    focused: Option<String>,
    focusables: [&'static str; 1],
    settings: Option<AetherSettingsComponent>,
}

impl WelcomePage {
    pub fn new() -> Self {
        Self {
            tx: None,
            focused: Some("settings".to_string()),
            focusables: ["settings"],
            settings: Some(AetherSettingsComponent::new()),
        }
    }
}

impl Page for WelcomePage {
    fn register_action_handler(
        &mut self,
        tx: tokio::sync::mpsc::UnboundedSender<Action>,
    ) -> Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn provide_components(&mut self) -> Vec<(String, Box<dyn Component>)> {
        let mut out: Vec<(String, Box<dyn Component>)> = vec![(
            "wizard_logo".to_string(),
            Box::new(WizardLogoComponent::new()) as Box<dyn Component>,
        )];
        if let Some(settings) = self.settings.take() {
            out.push(("settings".to_string(), Box::new(settings)));
        }
        out
    }

    fn focus(&mut self) -> Result<()> {
        if let Some(tx) = &self.tx {
            if let Some(first) = self.focusables.first() {
                let _ = tx.send(Action::Ui(UiAction::ReportFocusedComponent(
                    (*first).to_string(),
                )));
            }
        }
        Ok(())
    }

    fn keymap_context(&self) -> &'static str {
        "welcome"
    }

    fn id(&self) -> &'static str {
        "welcome"
    }

    fn focus_order(&self) -> &'static [&'static str] {
        &["settings"]
    }

    fn focused_component_id(&self) -> Option<&str> {
        self.focused.as_deref()
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::Ui(UiAction::ReportFocusedComponent(id)) = action {
            self.focused = Some(id);
        }
        Ok(None)
    }

    fn layout(&self, area: Rect) -> PageLayout {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(9), Constraint::Min(3)])
            .split(area);
        let header = vertical[0];
        let body = vertical[1];

        PageLayout::empty()
            .with("wizard_logo", header)
            .with("settings", body)
    }
}

// -- Aether settings status list --------------------------------------------------------------

struct AetherSettingsComponent {
    list: AetherList,
    focused: bool,
}

struct AetherList {
    items: Vec<AetherItem>,
    state: ListState,
}

#[derive(Debug)]
struct AetherItem {
    info: String,
    status: bool,
}

impl AetherItem {
    fn new(status: bool, info: &str) -> Self {
        Self {
            status,
            info: info.to_string(),
        }
    }
}

impl AetherSettingsComponent {
    pub fn new() -> Self {
        let settings_found = aether_config::find_setting();
        // TODO: replace stubs with actual validations.
        let settings_valid = true;
        let cert_found = true;
        let uds_found = true;

        let settings_found_text = if settings_found {
            "Settings found"
        } else {
            "Settings not found"
        };

        let settings_valid_text = if settings_valid {
            "Settings valid"
        } else {
            "Settings invalid"
        };

        let cert_found_text = if cert_found {
            "Certificate found"
        } else {
            "Certificate not found"
        };

        let uds_found_text = if uds_found {
            "Unix Domain Socket found"
        } else {
            "Unix Domain Socket not found"
        };

        Self {
            focused: false,
            list: AetherList {
                items: vec![
                    AetherItem::new(
                        settings_found && settings_valid,
                        &format!("{}\n{}", settings_found_text, settings_valid_text),
                    ),
                    AetherItem::new(cert_found, cert_found_text),
                    AetherItem::new(uds_found, uds_found_text),
                ],
                state: ListState::default(),
            },
        }
    }

    fn render(&mut self, f: &mut Frame<'_>, area: Rect) {
        // Split area into list and info panel
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(30), Constraint::Min(10)])
            .split(area);
        let list_area = cols[0];
        let info_area = cols[1];

        // Build list items
        let items: Vec<ListItem> = self
            .list
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let color = alternate_colors(i);
                ListItem::new(render_item_line(item)).style(Style::default().bg(color))
            })
            .collect();

        let block = Block::default()
            .title(Line::raw("Aether Status"))
            .borders(Borders::ALL)
            .border_style(if self.focused {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            });

        let list_widget = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        f.render_stateful_widget(list_widget, list_area, &mut self.list.state);

        // Info panel
        let info = if let Some(i) = self.list.state.selected() {
            if self.list.items[i].status {
                format!("✓ {}", self.list.items[i].info)
            } else {
                format!("☐ {}", self.list.items[i].info)
            }
        } else {
            "No item selected".to_string()
        };

        let info_block = Block::default()
            .title(Line::raw("Details"))
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1));

        let info_widget = Paragraph::new(info).block(info_block);
        f.render_widget(info_widget, info_area);
    }
}

impl Component for AetherSettingsComponent {
    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        self.render(f, area);
        Ok(())
    }
}

// --- rendering helpers & styles --------------------------------------------------------------
const NORMAL_ROW_BG: Color = Color::Rgb(15, 23, 42); // slate-900ish
const ALT_ROW_BG_COLOR: Color = Color::Rgb(30, 41, 59); // slate-800ish

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}

fn render_item_line(item: &AetherItem) -> Line<'static> {
    if item.status {
        Line::raw(format!(" ✓ {}", item.info))
    } else {
        Line::raw(format!(" ☐ {}", item.info))
    }
}
