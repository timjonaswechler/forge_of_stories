use crate::{
    action::{Action, PreflightItem, PreflightStatus},
    components::Component,
    tui::Frame,
};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, style::Color, widgets::Paragraph};

#[derive(Default)]
pub struct WelcomeComponent {
    items: Vec<PreflightItem>,
}

impl WelcomeComponent {
    pub fn with_items(items: Vec<PreflightItem>) -> Self {
        Self { items }
    }
}

impl WelcomeComponent {
    pub fn new() -> Self {
        Self {
            items: run_preflight(),
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
                Constraint::Length(5),
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

        frame.render_widget(welcome_text(), vertical[1]);
        frame.render_widget(preflight_paragraph(&self.items), layout[1]);
        Ok(())
    }
}

fn welcome_text() -> Paragraph<'static> {
    Paragraph::new(Text::from(vec![
        Line::from(vec![
            "Wizard ".fg(Color::Blue).bold(),
            "weaves ".into(),
            "Aether".fg(Color::Magenta).bold(),
            "’s".into(),
        ]),
        Line::from("configuration into existence."),
        Line::from(
            ratatui::symbols::line::HORIZONTAL
                .repeat(33)
                .fg(Color::DarkGray),
        ),
        Line::from(vec![
            env!("CARGO_PKG_NAME").into(),
            " v".into(),
            env!("CARGO_PKG_VERSION").into(),
        ]),
        Line::from(vec![
            "[".fg(Color::DarkGray),
            "with ♥ by ".into(),
            "@chicken105".fg(Color::Red),
            "]".fg(Color::DarkGray),
        ]),
    ]))
    .centered()
}

fn preflight_paragraph(checks: &[PreflightItem]) -> Paragraph<'static> {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from("Preflight checks:").bold());

    for item in checks {
        let status = match item.status {
            PreflightStatus::Present => Span::styled("✔", Style::default().fg(Color::Green)),
            PreflightStatus::Disabled => Span::styled("×", Style::default().fg(Color::Yellow)),
            PreflightStatus::Missing | PreflightStatus::Error => {
                Span::styled("✖", Style::default().fg(Color::Red))
            }
        };

        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::raw(" • "));
        spans.push(Span::raw(item.label.clone()));
        spans.push(Span::raw(" : "));
        spans.push(status);

        if let Some(note) = &item.message {
            spans.push(Span::raw(" ("));
            spans.push(Span::styled(
                note.clone(),
                Style::default().fg(Color::DarkGray),
            ));
            spans.push(Span::raw(")"));
        }

        lines.push(Line::from(spans));
    }
    lines.push(Line::from(""));
    if !checks
        .iter()
        .all(|item| item.status == PreflightStatus::Present)
    {
        lines.push(
            Line::from(vec![
                Span::raw("Press "),
                Span::raw("Enter").bold(),
                Span::raw(" to start"),
            ])
            .centered(),
        );
    } else {
        lines.push(Line::from("All is setup for running the server").centered());
        lines.push(
            Line::from(vec![
                Span::raw("Press "),
                Span::raw("Enter").bold(),
                Span::raw(" to check settings"),
            ])
            .centered(),
        );
    }

    Paragraph::new(Text::from(lines))
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
