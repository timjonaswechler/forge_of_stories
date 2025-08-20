use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect, Size},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use tokio::sync::mpsc::UnboundedSender;
use tui_prompts::Prompt;
use tui_prompts::{
    State,
    prelude::{FocusState, TextPrompt, TextRenderStyle, TextState},
};

use super::Component;
use crate::{action::Action, auth, config::Config, tui::Event};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    CreateAdmin,
    Login,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Username,
    Password,
}

pub struct AuthComponent {
    tx: Option<UnboundedSender<Action>>,
    config: Config,
    mode: Mode,
    username: String,
    password: String,
    username_state: TextState<'static>,
    password_state: TextState<'static>,
    focus: Focus,
    error: Option<String>,
    info: Option<String>,

    last_input_at: Instant,
    idle_timeout: Duration,
}

impl Default for AuthComponent {
    fn default() -> Self {
        let mode = if auth::exists() {
            Mode::Login
        } else {
            Mode::CreateAdmin
        };
        Self {
            tx: None,
            config: Config::new().unwrap_or_default(),
            mode,
            username: String::new(),
            password: String::new(),
            username_state: TextState::default(),
            password_state: TextState::default(),
            focus: Focus::Username,
            error: None,
            info: match mode {
                Mode::CreateAdmin => Some("Erstelle einen neuen Admin-Benutzer".into()),
                Mode::Login => Some("Bitte einloggen".into()),
            },

            last_input_at: Instant::now(),
            idle_timeout: Duration::from_secs(5),
        }
    }
}

impl AuthComponent {
    pub fn new() -> Self {
        Self::default()
    }

    fn clear_inputs(&mut self) {
        self.username.clear();
        self.password.clear();
        self.username_state = TextState::default();
        self.password_state = TextState::default();
        self.focus = Focus::Username;
        *self.username_state.focus_state_mut() = FocusState::Focused;
        *self.password_state.focus_state_mut() = FocusState::Unfocused;
        self.error = None;

        // Hinweis: self.info hier absichtlich nicht ändern
    }

    fn reset_to_login(&mut self) {
        self.clear_inputs();
        self.mode = if auth::exists() {
            Mode::Login
        } else {
            Mode::CreateAdmin
        };
        // Hinweis: self.info hier absichtlich nicht setzen; LoginPage zeigt Grundzeile
    }

    fn focused_state_mut(&mut self) -> &mut TextState<'static> {
        match self.focus {
            Focus::Username => &mut self.username_state,
            Focus::Password => &mut self.password_state,
        }
    }

    fn sync_from_states(&mut self) {
        self.username = self.username_state.value().to_string();
        self.password = self.password_state.value().to_string();
    }

    fn validate_password_msg(pw: &str) -> Option<String> {
        // Beispiel: Live-Validierung
        if pw.len() < 8 {
            return Some("Passwort muss mind. 8 Zeichen haben".into());
        }
        if !pw.chars().any(|c| c.is_ascii_lowercase()) {
            return Some("Mind. 1 Kleinbuchstabe".into());
        }

        if !pw.chars().any(|c| c.is_ascii_uppercase()) {
            return Some("Mind. 1 Großbuchstabe".into());
        }
        if !pw.chars().any(|c| c.is_ascii_digit()) {
            return Some("Mind. 1 Ziffer".into());
        }
        None
    }

    fn submit(&mut self) -> Result<Option<Action>> {
        self.error = None;
        self.info = None;

        match self.mode {
            Mode::CreateAdmin => {
                if self.username.trim().len() < 3 {
                    self.error = Some("Username muss mind. 3 Zeichen haben".into());
                    return Ok(None);
                }
                if let Some(msg) = Self::validate_password_msg(&self.password) {
                    self.error = Some(msg);
                    return Ok(None);
                }
                auth::create_admin(self.username.trim(), &self.password)?;
                self.info = Some("Admin angelegt. Bitte jetzt einloggen.".into());
                self.username.clear();
                self.password.clear();
                self.username_state = TextState::default();
                self.password_state = TextState::default();
                self.mode = Mode::Login;
                self.focus = Focus::Username;
                *self.username_state.focus_state_mut() = FocusState::Focused;
                *self.password_state.focus_state_mut() = FocusState::Unfocused;
                Ok(None)
            }
            Mode::Login => {
                let ok = auth::verify(self.username.trim(), &self.password)?;
                if !ok {
                    self.error = Some("Ungültige Zugangsdaten".into());
                    return Ok(None);
                }
                // Anmeldedaten sicher aus dem Speicher entfernen
                self.clear_inputs();
                self.info = None;

                if let Some(tx) = &self.tx {
                    let _ = tx.send(Action::Navigate("home".into()));
                }
                Ok(None)
            }
        }
    }
}

impl Component for AuthComponent {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn init(&mut self, _area: Size) -> Result<()> {
        // Initialen Modus anhand vorhandener Credentials festlegen
        self.mode = if auth::exists() {
            Mode::Login
        } else {
            Mode::CreateAdmin
        };
        self.last_input_at = Instant::now();

        // Felder und Fokus initial zurücksetzen
        self.clear_inputs();

        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        match event {
            Some(Event::Key(key)) => {
                self.last_input_at = Instant::now();
                self.handle_key_event(key)
            }
            Some(Event::Mouse(mouse)) => self.handle_mouse_event(mouse),
            _ => Ok(None),
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::IdleTimeout => {
                // Sofort zurücksetzen und Grund anzeigen
                self.reset_to_login();
                self.info = Some("Zurück zum Login (Inaktivität)".into());
                self.last_input_at = Instant::now();
            }
            Action::Tick => {
                // Inaktivität prüfen (nur relevant, wenn wir bereits auf der Login-Seite sind)
                if self.last_input_at.elapsed() >= self.idle_timeout {
                    self.reset_to_login();
                    self.last_input_at = Instant::now();
                }
            }
            Action::Render => {}
            _ => {}
        }
        Ok(None)
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        self.last_input_at = Instant::now();

        // Respektiere page-spezifische Keybindings (z. B. Quit/Suspend), indem wir diese Keys hier nicht verbrauchen.
        if let Some(bindings) = self.config.keybindings.get_by_name("login") {
            if bindings.contains_key(&vec![key]) {
                return Ok(None);
            }
        }

        match key.code {
            KeyCode::Tab => {
                // Fokuswechsel: Username <-> Password
                match self.focus {
                    Focus::Username => {
                        *self.username_state.focus_state_mut() = FocusState::Unfocused;
                        self.focus = Focus::Password;
                        *self.password_state.focus_state_mut() = FocusState::Focused;
                    }
                    Focus::Password => {
                        *self.password_state.focus_state_mut() = FocusState::Unfocused;
                        self.focus = Focus::Username;
                        *self.username_state.focus_state_mut() = FocusState::Focused;
                    }
                }
                self.error = None;
                Ok(None)
            }
            KeyCode::BackTab => {
                // Fokuswechsel rückwärts: Password <-> Username
                match self.focus {
                    Focus::Username => {
                        *self.username_state.focus_state_mut() = FocusState::Unfocused;
                        self.focus = Focus::Password;
                        *self.password_state.focus_state_mut() = FocusState::Focused;
                    }
                    Focus::Password => {
                        *self.password_state.focus_state_mut() = FocusState::Unfocused;
                        self.focus = Focus::Username;
                        *self.username_state.focus_state_mut() = FocusState::Focused;
                    }
                }
                self.error = None;
                Ok(None)
            }
            KeyCode::Enter => {
                // Vor Submit syncen
                self.sync_from_states();
                self.submit()
            }
            KeyCode::Esc => {
                // Felder komplett zurücksetzen und Fokus auf Username (Info unverändert lassen)
                self.clear_inputs();
                Ok(None)
            }
            _ => {
                // Alle übrigen Tasten an den aktiven TextState delegieren
                self.focused_state_mut().handle_key_event(key);
                // Werte übernehmen
                self.sync_from_states();

                // Live-Validierung fürs Passwort während CreateAdmin
                if matches!(self.mode, Mode::CreateAdmin) && matches!(self.focus, Focus::Password) {
                    self.error = Self::validate_password_msg(&self.password);
                } else {
                    self.error = None;
                }
                Ok(None)
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame, body: Rect) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Header-Bereich
                Constraint::Length(1), // Username
                Constraint::Length(1), // Password
                Constraint::Length(2), // Validation/Info
                Constraint::Min(0),
            ])
            .split(body);

        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(
                        "Analyze ELF binaries ",
                        Style::default().fg(Color::Rgb(140, 145, 160)),
                    ),
                    Span::styled(
                        "like a boss.",
                        Style::default().fg(Color::Rgb(255, 255, 255)),
                    ),
                ]),
                Line::from(Span::styled(
                    ratatui::symbols::line::HORIZONTAL.repeat(31),
                    Style::default().fg(Color::Rgb(100, 100, 100)),
                )),
                Line::from(vec![
                    match self.mode {
                        Mode::CreateAdmin => "Create Admin",
                        Mode::Login => "Login",
                    }
                    .into(),
                ]),
                Line::from(Span::styled(
                    self.info.clone().unwrap_or_default(),
                    Style::default().fg(Color::Rgb(100, 255, 100)),
                )),
            ])
            .centered(),
            chunks[0],
        );
        // 2) Username Prompt
        TextPrompt::from("Username").draw(frame, chunks[1], &mut self.username_state);

        // 3) Password Prompt (maskiert)
        TextPrompt::from("Password")
            .with_render_style(TextRenderStyle::Password)
            .draw(frame, chunks[2], &mut self.password_state);

        // 4) Validation / Info / Error
        let validation = if let Some(err) = &self.error {
            Span::styled(err.as_str(), Style::default().fg(Color::Red))
        } else if self.password.len() >= 12 {
            Span::styled("Passwortstärke: gut", Style::default().fg(Color::Green))
        } else {
            Span::styled("", Style::default())
        };
        frame.render_widget(Paragraph::new(Line::from(validation)), chunks[3]);

        Ok(())
    }
}
