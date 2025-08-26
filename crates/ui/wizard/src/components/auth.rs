use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect, Size},
    style::{Modifier, Style},
    symbols::border,
    widgets::{Block, Paragraph},
};

use tokio::sync::mpsc::UnboundedSender;

use tui_input::Input;
use tui_input::backend::crossterm::EventHandler as _;

use super::Component;
use crate::services::keybind_symbols::{CONTROL_C, ENTER, ESC, SHIFT_TAB, TAB};
use crate::services::shortcuts::Shortcut;
use crate::{action::Action, config::Config, services::auth, style::Theme, tui::Event};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    CreateAdmin,
    Login,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Username,
    Password,
    Confirm,
}

pub struct AuthComponent {
    tx: Option<UnboundedSender<Action>>,
    config: Config,
    theme: Theme,
    mode: Mode,
    username: String,
    password: String,
    confirm_password: String,
    username_input: Input,
    password_input: Input,
    confirm_input: Input,
    focus: Focus,
    error: Option<String>,
    info: Option<String>,
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
            theme: crate::style::default_dark_theme(),
            mode,
            username: String::new(),
            password: String::new(),
            confirm_password: String::new(),
            username_input: Input::default(),
            password_input: Input::default(),
            confirm_input: Input::default(),
            focus: Focus::Username,
            error: None,
            info: match mode {
                Mode::CreateAdmin => Some("Erstelle einen neuen Admin-Benutzer".into()),
                Mode::Login => Some("Bitte einloggen".into()),
            },
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
        self.confirm_password.clear();
        self.username_input = Input::default();
        self.password_input = Input::default();
        self.confirm_input = Input::default();
        self.focus = Focus::Username;
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

    fn sync_from_states(&mut self) {
        self.username = self.username_input.value().to_string();
        self.password = self.password_input.value().to_string();
        self.confirm_password = self.confirm_input.value().to_string();
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
                if self.password != self.confirm_password {
                    self.error = Some("Passwörter stimmen nicht überein".into());
                    return Ok(None);
                }
                auth::create_admin(self.username.trim(), &self.password)?;
                self.info = Some("Admin angelegt. Bitte jetzt einloggen.".into());
                self.username.clear();
                self.password.clear();
                self.username_input = Input::default();
                self.password_input = Input::default();
                self.mode = Mode::Login;
                self.focus = Focus::Username;
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

    fn register_theme(&mut self, theme: Theme) -> Result<()> {
        self.theme = theme;
        Ok(())
    }

    fn register_shortcuts(&self) -> Option<(&'static str, Box<[Shortcut]>)> {
        use crate::shortcuts;
        let scope = match self.mode {
            Mode::Login => "Login",
            Mode::CreateAdmin => "Create Admin",
        };

        let common = shortcuts!(
            ("Submit", [ENTER]),
            ("Cancle", [ESC]),
            ("Help", ["?"]),
            ("Quit", [CONTROL_C]),
        );

        // Fokus-basierte Navigation
        let nav = match self.mode {
            Mode::Login => match self.focus {
                Focus::Username => {
                    shortcuts!(("Continue", [TAB]), ("Back", [SHIFT_TAB]))
                }
                Focus::Password => shortcuts!(("Continue", [TAB]), ("Back", [SHIFT_TAB])),
                Focus::Confirm => shortcuts!(("Continue", [TAB]), ("Back", [SHIFT_TAB])),
            },
            Mode::CreateAdmin => match self.focus {
                Focus::Username => shortcuts!(("Continue", [TAB]), ("Back", [SHIFT_TAB])),
                Focus::Password => shortcuts!(("Continue", [TAB]), ("Back", [SHIFT_TAB])),
                Focus::Confirm => shortcuts!(("Continue", [TAB]), ("Back", [SHIFT_TAB])),
            },
        };

        // Zusammenführen: nav + common
        let mut v = Vec::new();
        v.extend_from_slice(&nav);
        v.extend_from_slice(&common);
        Some((scope, v.into_boxed_slice()))
    }

    fn init(&mut self, _area: Size) -> Result<()> {
        self.mode = if auth::exists() {
            Mode::Login
        } else {
            Mode::CreateAdmin
        };

        self.clear_inputs();

        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        match event {
            Some(Event::Key(key)) => {
                // self.last_input_at = Instant::now();
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
                // self.last_input_at = Instant::now();
            }
            Action::Tick => {}
            Action::Render => {}
            _ => {}
        }
        Ok(None)
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        // self.last_input_at = Instant::now();

        match key.code {
            KeyCode::Tab => {
                // Fokuswechsel abhängig vom Modus
                match (self.mode, self.focus) {
                    (Mode::CreateAdmin, Focus::Username) => {
                        self.focus = Focus::Password;
                    }
                    (Mode::CreateAdmin, Focus::Password) => {
                        self.focus = Focus::Confirm;
                    }
                    (Mode::CreateAdmin, Focus::Confirm) => {
                        self.focus = Focus::Username;
                    }
                    (Mode::Login, Focus::Username) => {
                        self.focus = Focus::Password;
                    }
                    (Mode::Login, Focus::Password) => {
                        self.focus = Focus::Username;
                    }
                    (Mode::Login, Focus::Confirm) => {
                        // Fallback: sollte im Login-Modus nicht auftreten
                        self.focus = Focus::Username;
                    }
                }
                self.error = None;
                Ok(None)
            }
            KeyCode::BackTab => {
                // Fokuswechsel rückwärts abhängig vom Modus
                match (self.mode, self.focus) {
                    (Mode::CreateAdmin, Focus::Username) => {
                        self.focus = Focus::Confirm;
                    }
                    (Mode::CreateAdmin, Focus::Confirm) => {
                        self.focus = Focus::Password;
                    }
                    (Mode::CreateAdmin, Focus::Password) => {
                        self.focus = Focus::Username;
                    }
                    (Mode::Login, Focus::Username) => {
                        self.focus = Focus::Password;
                    }
                    (Mode::Login, Focus::Password) => {
                        self.focus = Focus::Username;
                    }
                    (Mode::Login, Focus::Confirm) => {
                        // Fallback: sollte im Login-Modus nicht auftreten
                        self.focus = Focus::Password;
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
            KeyCode::Char('?') => {
                // Kontext-Hilfe abhängig von Fokus und Modus anzeigen
                self.error = None;
                self.info = Some(match (self.mode, self.focus) {
                    (Mode::CreateAdmin, Focus::Username) => {
                        "Wähle einen Admin-Benutzernamen (mind. 3 Zeichen)".into()
                    }
                    (Mode::CreateAdmin, Focus::Password) => {
                        "Passwortanforderungen: mind. 8 Zeichen, je 1 Klein-, 1 Großbuchstabe und 1 Ziffer".into()
                    }
                    (Mode::CreateAdmin, Focus::Confirm) => {
                        "Wiederhole das Passwort zur Bestätigung".into()
                    }
                    (Mode::Login, Focus::Username) => "Gib deinen Benutzernamen ein".into(),
                    (Mode::Login, Focus::Password) => "Gib dein Passwort ein".into(),
                    (Mode::Login, Focus::Confirm) => "Gib dein Passwort ein".into(),
                });
                Ok(None)
            }
            _ => {
                // Alle übrigen Tasten an das aktive Input-Feld delegieren
                let ev = crossterm::event::Event::Key(key);
                match self.focus {
                    Focus::Username => {
                        self.username_input.handle_event(&ev);
                    }
                    Focus::Password => {
                        self.password_input.handle_event(&ev);
                    }
                    Focus::Confirm => {
                        self.confirm_input.handle_event(&ev);
                    }
                }
                // Werte übernehmen
                self.sync_from_states();

                // Live-Validierung & Abgleich im CreateAdmin-Modus
                if matches!(self.mode, Mode::CreateAdmin) {
                    if !self.confirm_password.is_empty() && self.password != self.confirm_password {
                        self.error = Some("Passwörter stimmen nicht überein".into());
                    } else if matches!(self.focus, Focus::Password) {
                        self.error = Self::validate_password_msg(&self.password);
                    } else {
                        self.error = None;
                    }
                } else {
                    self.error = None;
                }
                Ok(None)
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame, body: Rect) -> Result<()> {
        let bg = Block::default()
            .style(ratatui::style::Style::default().bg(self.theme.roles.background));
        frame.render_widget(bg, body);
        let horizontal = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(50),
                Constraint::Fill(1),
            ])
            .split(body);
        let vertical = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(8), // Header
            Constraint::Length(1),
            Constraint::Length(2), // Info
            Constraint::Length(1),
            Constraint::Length(3), // Username
            Constraint::Length(1),
            Constraint::Length(3), // Password
            Constraint::Length(1),
            if matches!(self.mode, Mode::CreateAdmin) {
                Constraint::Length(3) // Confirm Password
            } else {
                Constraint::Length(0)
            },
            Constraint::Fill(1),
        ]);

        let [
            _,
            header,
            _,
            info,
            _,
            username_input,
            _,
            password_input,
            _,
            comfirm_password_input,
            _,
        ] = vertical.areas(horizontal[1]);
        self.render_header(frame, header);
        self.render_info(frame, info);
        self.render_username(frame, username_input);
        self.render_password(frame, password_input);
        if matches!(self.mode, Mode::CreateAdmin) {
            self.render_confirm(frame, comfirm_password_input);
        }

        Ok(())
    }
}

impl AuthComponent {
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        // frame.render_widget(Paragraph::new().centered(), area);
    }

    fn render_info(&self, frame: &mut Frame, area: Rect) {
        let (msg, style) = if let Some(err) = &self.error {
            (err.as_str(), Style::default().fg(self.theme.roles.danger))
        } else if let Some(info) = &self.info {
            (info.as_str(), Style::default().fg(self.theme.roles.info))
        } else {
            ("", Style::default().fg(self.theme.roles.subtle_text))
        };
        let info_paragraph = Paragraph::new(msg).centered().style(style);
        frame.render_widget(info_paragraph, area);
    }

    fn render_username(&self, frame: &mut Frame, area: Rect) {
        let horizontal = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(45),
                Constraint::Fill(1),
            ])
            .split(area);
        let width = horizontal[1].width.max(3) - 3;
        let scroll = self.username_input.visual_scroll(width as usize);

        let title_style = if self.focus == Focus::Username {
            Style::default()
                .fg(self.theme.roles.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.theme.roles.subtle_text)
        };
        let border_style = if self.focus == Focus::Username {
            Style::default().fg(self.theme.roles.primary)
        } else {
            Style::default().fg(self.theme.roles.muted)
        };
        let input_style = Style::default().fg(self.theme.roles.text);

        let input = Paragraph::new(self.username_input.value())
            .scroll((0, scroll as u16))
            .style(input_style)
            .block(
                Block::bordered()
                    .title("Username")
                    .title_style(title_style)
                    .border_set(border::ROUNDED)
                    .border_style(border_style),
            );
        frame.render_widget(input, horizontal[1]);

        if self.focus == Focus::Username {
            // Ratatui hides the cursor unless it's explicitly set. Position the  cursor past the
            // end of the input text and one line down from the border to the input line
            let x = self.username_input.visual_cursor().max(scroll) - scroll + 1;
            frame.set_cursor_position((horizontal[1].x + x as u16, horizontal[1].y + 1))
        }
    }

    fn render_password(&self, frame: &mut Frame, area: Rect) {
        let horizontal = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(45),
                Constraint::Fill(1),
            ])
            .split(area);
        // keep 2 for borders and 1 for cursor
        let width = horizontal[1].width.max(3) - 3;
        let scroll = self.password_input.visual_scroll(width as usize);
        let input_style = if self.focus == Focus::Password {
            Style::default().fg(self.theme.roles.text)
        } else {
            Style::default().fg(self.theme.roles.subtle_text)
        };
        let title_style = if self.focus == Focus::Password {
            Style::default()
                .fg(self.theme.roles.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.theme.roles.subtle_text)
        };
        let border_style = if self.focus == Focus::Password {
            Style::default().fg(self.theme.roles.primary)
        } else {
            Style::default().fg(self.theme.roles.muted)
        };
        let masked: String = self.password_input.value().chars().map(|_| '•').collect();
        let input = Paragraph::new(masked)
            .style(input_style)
            .scroll((0, scroll as u16))
            .block(
                Block::bordered()
                    .title("Password")
                    .title_style(title_style)
                    .border_set(border::ROUNDED)
                    .border_style(border_style),
            );
        frame.render_widget(input, horizontal[1]);

        if self.focus == Focus::Password {
            let x = self.password_input.visual_cursor().max(scroll) - scroll + 1;
            frame.set_cursor_position((horizontal[1].x + x as u16, horizontal[1].y + 1))
        }
    }

    fn render_confirm(&self, frame: &mut Frame, area: Rect) {
        let horizontal = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(45),
                Constraint::Fill(1),
            ])
            .split(area);
        // keep 2 for borders and 1 for cursor
        let width = horizontal[1].width.max(3) - 3;
        let scroll = self.confirm_input.visual_scroll(width as usize);
        let input_style = if self.focus == Focus::Confirm {
            Style::default().fg(self.theme.roles.text)
        } else {
            Style::default().fg(self.theme.roles.subtle_text)
        };
        let title_style = if self.focus == Focus::Confirm {
            Style::default()
                .fg(self.theme.roles.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.theme.roles.subtle_text)
        };
        let border_style = if self.focus == Focus::Confirm {
            Style::default().fg(self.theme.roles.primary)
        } else {
            Style::default().fg(self.theme.roles.muted)
        };
        let masked: String = self.confirm_input.value().chars().map(|_| '•').collect();
        let input = Paragraph::new(masked)
            .style(input_style)
            .scroll((0, scroll as u16))
            .block(
                Block::bordered()
                    .title("Confirm")
                    .title_style(title_style)
                    .border_set(border::ROUNDED)
                    .border_style(border_style),
            );
        frame.render_widget(input, horizontal[1]);

        if self.focus == Focus::Confirm {
            let x = self.confirm_input.visual_cursor().max(scroll) - scroll + 1;
            frame.set_cursor_position((horizontal[1].x + x as u16, horizontal[1].y + 1))
        }
    }
}
