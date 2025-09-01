use crate::{action::Action, components::Component, tui::Frame};
use color_eyre::eyre::Result;
use ratatui::{prelude::*, style::Color, widgets::Paragraph};

#[derive(Default)]
pub struct WelcomeComponent {}

impl WelcomeComponent {
    pub fn new() -> Self {
        Self {
            ..Default::default()
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
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> Result<()> {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(2),
                Constraint::Length(5),
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Fill(2),
            ])
            .split(area);

        frame.render_widget(welcome_text(), vertical[1]);
        frame.render_widget(start_text(), vertical[3]);
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

fn start_text() -> Paragraph<'static> {
    Paragraph::new(Text::from(vec![
        Line::from(""),
        Line::from("Press Enter to start"),
        Line::from(""),
    ]))
    .centered()
}

fn detect_server_settings() -> Result<bool> {
    // search for server settings
    Ok(true)
}
fn detect_server_installation() -> Result<bool> {
    // search for server installation
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
fn detect_certs() -> Result<bool> {
    // search for certs
    Ok(true)
}
fn detect_uds() -> Result<bool> {
    // search for cert
    Ok(true)
}
