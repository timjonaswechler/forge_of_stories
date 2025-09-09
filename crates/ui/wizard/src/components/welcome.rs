use crate::{
    action::{Action, PreflightItem, PreflightStatus},
    components::Component,
    tui::Frame,
};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::{Paragraph, Block},style::Modifier};
use crate::theme::{Theme, UiGroup, Mode};

pub struct WelcomeComponent {
    items: Vec<PreflightItem>,
    mode: Mode,
}

impl Default for WelcomeComponent {
    fn default() -> Self {
        Self { items: Vec::new(), mode: Mode::Normal }
    }
}

impl WelcomeComponent {
    pub fn with_items(items: Vec<PreflightItem>) -> Self {
        Self { items, mode: Mode::Normal }
    }
}

impl WelcomeComponent {
    pub fn new() -> Self {
        Self {
            items: run_preflight(),
            mode: Mode::Normal,
        }
    }
}

impl Component for WelcomeComponent {
    fn name(&self) -> &'static str {
        "start"
    }
    fn init(&mut self) -> Result<()> {
        Ok(())
    }
    fn height_constraint(&self) -> Constraint {
        Constraint::Length(3)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Submit => Ok(Some(Action::Navigate(1))),
            Action::SetMode(m) => { self.mode = m; Ok(None) }
            Action::CycleMode => { self.mode = self.mode.next(); Ok(None) }
            Action::PreflightResults(items) => {
                self.items = items;
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn handle_key_events(
        &mut self,
        key: KeyEvent,
    ) -> Result<Option<crate::tui::EventResponse<Action>>> {
        if let KeyCode::Enter = key.code {
            return Ok(Some(crate::tui::EventResponse::Stop(Action::Submit)));
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> Result<()> {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(2),
                Constraint::Length(6),
                Constraint::Length(2),
                Constraint::Length(10),
                Constraint::Fill(2),
            ])
            .split(area);

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(35),
                Constraint::Fill(1),
            ])
            .split(vertical[3]);

        frame.render_widget(self.welcome_text_with_meta(), vertical[1]);
        frame.render_widget(preflight_paragraph(&self.items), layout[1]);
        Ok(())
    }
}

impl WelcomeComponent {
    fn welcome_text_with_meta(&self) -> Paragraph<'static> {
        let theme = Theme::from_env_auto();
        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from(vec![
            Span::styled("Wizard ", theme.style(UiGroup::Info).add_modifier(Modifier::BOLD)),
            Span::raw("weaves "),
            Span::styled("Aether", theme.style(UiGroup::ModeVisual)),
            Span::raw("’s"),
        ]));
        lines.push(Line::from("configuration into existence."));
        // meta line: colors + mode
        lines.push(Line::from(vec![
            Span::styled("[", theme.style(UiGroup::Dimmed)),
            Span::raw("colors: "),
            Span::styled(theme.mode_label(), theme.style(UiGroup::Info)),
            Span::styled("]  [", theme.style(UiGroup::Dimmed)),
            Span::raw("mode: "),
            Span::styled(self.mode.label(), self.mode.style(&theme)),
            Span::styled("]", theme.style(UiGroup::Dimmed)),
        ]));
        lines.push(Line::from(
            ratatui::symbols::line::HORIZONTAL
                .repeat(33)
                .as_str()
                .to_string(),
        ).style(theme.style(UiGroup::Border)));
        lines.push(Line::from(vec![
            Span::raw(env!("CARGO_PKG_NAME")),
            Span::raw(" v"),
            Span::raw(env!("CARGO_PKG_VERSION")),
        ]));
        lines.push(Line::from(vec![
            Span::styled("[", theme.style(UiGroup::Dimmed)),
            Span::raw("with ♥ by "),
            Span::styled("@chicken105", theme.style(UiGroup::Info)),
            Span::styled("]", theme.style(UiGroup::Dimmed)),
        ]));

        Paragraph::new(Text::from(lines)).centered()
    }
}

fn preflight_paragraph(checks: &[PreflightItem]) -> Paragraph<'static> {
    let theme = Theme::from_env_auto();
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled("Preflight checks:", theme.style(UiGroup::Title))));

    for item in checks {
        let status = match item.status {
            PreflightStatus::Present => Span::styled("✔", theme.style(UiGroup::Success)),
            PreflightStatus::Disabled => Span::styled("×", theme.style(UiGroup::Warn)),
            PreflightStatus::Missing | PreflightStatus::Error => Span::styled("✖", theme.style(UiGroup::Error)),
        };

        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::raw(" • "));
        spans.push(Span::raw(item.label.clone()));
        spans.push(Span::raw(" : "));
        spans.push(status);

        if let Some(note) = &item.message {
            spans.push(Span::raw(" ("));
            spans.push(Span::styled(note.clone(), theme.style(UiGroup::Dimmed)));
            spans.push(Span::raw(")"));
        }

        lines.push(Line::from(spans));
    }
    lines.push(Line::from(""));
    if !checks
        .iter()
        .all(|item| item.status == PreflightStatus::Present)
    {
        lines.push(Line::from(vec![
            Span::raw("Press "),
            Span::styled("Enter", theme.style(UiGroup::ModeNormal)),
            Span::raw(" to start"),
        ]).centered());
    } else {
        lines.push(Line::from(Span::styled("All is setup for running the server", theme.style(UiGroup::Success))).centered());
        lines.push(Line::from(vec![
            Span::raw("Press "),
            Span::styled("Enter", theme.style(UiGroup::ModeNormal)),
            Span::raw(" to check settings"),
        ]).centered());
    }

    Paragraph::new(Text::from(lines))
        .block(
            Block::bordered()
                .border_set(ratatui::symbols::border::ROUNDED)
                .border_style(theme.style(UiGroup::Border))
                .title(Span::styled("System", theme.style(UiGroup::Dimmed))),
        )
}

fn detect_server_installation() -> Result<bool> {
    // search for server installation
    Ok(true)
}

fn detect_server_settings() -> Result<bool> {
    // search for server settings
    Ok(true)
}

fn detect_certs() -> Result<bool> {
    // search for certs
    Ok(true)
}
fn detect_server_user_group() -> Result<bool> {
    // search for server user group
    Ok(true)
}
fn detect_server_user() -> Result<bool> {
    // search for server user
    Ok(true)
}

fn detect_uds() -> Result<bool> {
    // search for cert
    Ok(true)
}

pub fn run_preflight() -> Vec<PreflightItem> {
    let mut items = Vec::new();
    let mut push = |label: &str, res: Result<bool>| match res {
        Ok(true) => items.push(PreflightItem {
            label: label.to_string(),
            status: PreflightStatus::Present,
            message: None,
        }),
        Ok(false) => items.push(PreflightItem {
            label: label.to_string(),
            status: PreflightStatus::Missing,
            message: None,
        }),
        Err(e) => items.push(PreflightItem {
            label: label.to_string(),
            status: PreflightStatus::Error,
            message: Some(e.to_string()),
        }),
    };

    push("Server settings", detect_server_settings());
    push("Server installation", detect_server_installation());
    push("Certificates", detect_certs());
    push("Server user group", detect_server_user_group());
    push("Server user", detect_server_user());
    push("UDS / IPC socket", detect_uds());

    items
}
