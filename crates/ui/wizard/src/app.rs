use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Flex, Layout},
    prelude::Rect,
    symbols::border,
    text::Text,
    widgets::{Block, Paragraph, Widget},
};

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
use tracing::debug;
use util::AppContext;

use crate::messages::{AetherStatsSnapshot, AetherToWizard};
use crate::services::aether::{AetherSettingsSnapshot, AetherSupervisor};
use crate::services::shortcuts::create_shortcut_list;
use crate::{
    action::Action,
    config::Config,
    pages::{HomePage, LoginPage, Page, SetupPage},
    style,
    tui::{Event, Tui},
};

const STATS_HISTORY_LEN: usize = 300;

#[derive(Default, Debug)]
pub struct StatsHistory {
    buf: VecDeque<AetherStatsSnapshot>,
}

impl StatsHistory {
    fn with_capacity(cap: usize) -> Self {
        Self {
            buf: VecDeque::with_capacity(cap),
        }
    }

    fn push(&mut self, sample: AetherStatsSnapshot) {
        self.buf.push_back(sample);
        if self.buf.len() > STATS_HISTORY_LEN {
            self.buf.pop_front();
        }
    }

    fn as_iter(&self) -> impl Iterator<Item = &AetherStatsSnapshot> {
        self.buf.iter()
    }

    fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

pub struct App {
    context: AppContext,
    config: Config,
    tick_rate: f64,
    frame_rate: f64,
    pages: HashMap<String, Box<dyn Page>>,
    current_page: Option<String>,
    should_quit: bool,
    should_suspend: bool,
    last_tick_key_events: Vec<KeyEvent>,
    last_input_at: Instant,
    idle_timeout: Duration,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,

    aether_sup: Arc<Mutex<AetherSupervisor>>,
    aether_rx: Option<tokio::sync::mpsc::UnboundedReceiver<AetherToWizard>>,
    aether_settings: Option<AetherSettingsSnapshot>,
    settings_ready: bool,

    // Dashboard state
    server_running: bool,
    last_error: Option<String>,
    stats_history: Arc<Mutex<StatsHistory>>,

    autostart_on_launch: bool,
    autostart_on_settings_ready: bool,
    autostart_done: bool,

    // Theming
    theme: style::Theme,
    palette_guard: Option<style::TerminalPaletteGuard>,
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64, cx: AppContext) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        let pages: HashMap<String, Box<dyn Page>> = {
            let mut m = HashMap::new();
            // Register login and home pages
            m.insert(
                "login".to_string(),
                Box::new(LoginPage::new()) as Box<dyn Page>,
            );
            m.insert(
                "setup".to_string(),
                Box::new(SetupPage::new()) as Box<dyn Page>,
            );
            m.insert(
                "home".to_string(),
                Box::new(HomePage::new()) as Box<dyn Page>,
            );
            m
        };

        let mut sup = AetherSupervisor::new();
        let aether_rx = sup.take_event_receiver();

        Ok(Self {
            context: cx,
            config: Config::new()?,
            tick_rate,
            frame_rate,
            pages,
            current_page: Some("login".to_string()),
            should_quit: false,
            should_suspend: false,
            last_tick_key_events: Vec::new(),
            last_input_at: std::time::Instant::now(),
            idle_timeout: std::time::Duration::from_secs(5 * 60),

            action_tx,
            action_rx,

            aether_sup: Arc::new(Mutex::new(sup)),
            aether_rx,
            aether_settings: None,
            settings_ready: false,

            server_running: false,
            last_error: None,
            stats_history: Arc::new(Mutex::new(StatsHistory::with_capacity(STATS_HISTORY_LEN))),

            autostart_on_launch: false, // oder true, wenn du direkt starten willst
            autostart_on_settings_ready: true, // z. B. automatisch starten, sobald Settings ok sind
            autostart_done: false,

            theme: style::default_dark_theme(),
            palette_guard: None,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            // .mouse(true) // uncomment this line to enable mouse support
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);

        // Apply terminal palette override for the selected theme; reset via RAII on drop
        self.palette_guard = Some(style::apply_theme_palette(&self.theme));

        tui.enter()?;

        if let Some(current) = &self.current_page {
            if let Some(page) = self.pages.get_mut(current) {
                page.register_action_handler(self.action_tx.clone())?;
                page.register_config_handler(self.config.clone())?;
                page.register_theme(self.theme.clone())?;
                page.init(tui.size()?)?;
                page.register_shared_state(self.stats_history.clone())?;
                // Force an initial full redraw after first page init
                let _ = self.action_tx.send(Action::ClearScreen);
                let _ = self.action_tx.send(Action::Render);
            }
        }
        if self.autostart_on_launch && !self.autostart_done {
            // Entweder Settings aus Store oder Default
            if self.aether_settings.is_none() {
                self.aether_settings = Some(AetherSettingsSnapshot::default());
                self.settings_ready = true;
            }
            let _ = self.action_tx.send(Action::StartServer);
            self.autostart_done = true;
        }

        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        // Drop the palette guard to reset terminal colors after exiting TUI
        self.palette_guard = None;
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx.send(Action::Quit)?,
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,
            Event::Mouse(_) => {
                self.last_input_at = Instant::now();
            }
            _ => {}
        }
        if let Some(current) = &self.current_page {
            if let Some(page) = self.pages.get_mut(current) {
                if let Some(action) = page.handle_events(Some(event.clone()))? {
                    action_tx.send(action)?;
                }
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        self.last_input_at = Instant::now();
        let action_tx = self.action_tx.clone();

        Ok(())
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
            }
            let action_clone = action.clone();
            match &action_clone {
                Action::Tick => {
                    self.last_tick_key_events.drain(..);
                    if self.current_page.as_deref() != Some("login")
                        && self.last_input_at.elapsed() >= self.idle_timeout
                    {
                        // Navigate first so login page is active, then notify about idle timeout
                        let _ = self.action_tx.send(Action::Navigate("login".to_string()));
                        let _ = self.action_tx.send(Action::IdleTimeout);
                        self.last_input_at = Instant::now();
                    }
                }
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Navigate(name) => {
                    // attempt to switch to the requested page
                    if !self.pages.contains_key(name.as_str()) {
                        debug!("Attempted to navigate to unknown page: {}", name);
                    } else {
                        // call on_exit on current page
                        if let Some(current) = &self.current_page {
                            if let Some(page) = self.pages.get_mut(current) {
                                let _ = page.on_exit();
                            }
                        }
                        // set new page
                        self.current_page = Some(name.clone());
                        // init/register new page
                        if let Some(page) = self.pages.get_mut(name.as_str()) {
                            let _ = page.register_action_handler(self.action_tx.clone());
                            let _ = page.register_config_handler(self.config.clone());
                            let _ = page.register_theme(self.theme.clone());
                            let _ = page.init(tui.size()?);
                            let _ = page.register_shared_state(self.stats_history.clone());
                            let _ = page.on_enter();
                        }
                        // force a full redraw
                        let _ = self.action_tx.send(Action::ClearScreen);
                        let _ = self.action_tx.send(Action::Render);
                    }
                }
                Action::Resize(w, h) => self.handle_resize(tui, *w, *h)?,
                Action::Render => self.render(tui)?,
                Action::SettingsReady => {
                    self.settings_ready = true;
                    // Build settings snapshot (placeholder; later from settings store)
                    self.aether_settings = Some(AetherSettingsSnapshot {
                        tick_hz: 60,
                        quic_bind_addr: "0.0.0.0:7777".into(),
                    });

                    if self.autostart_on_settings_ready && !self.autostart_done {
                        let _ = self.action_tx.send(Action::StartServer);
                        self.autostart_done = true;
                    }
                }

                Action::SettingsInvalid(msg) => {
                    self.settings_ready = false;
                    self.aether_settings = None;
                    let _ = self.action_tx.send(Action::Error(msg.clone()));
                }

                Action::ApplyRuntimeSetting { key, value } => {
                    let sup = self.aether_sup.lock().unwrap();
                    if let Err(e) =
                        sup.send_control(crate::messages::WizardToAether::ApplyRuntimeSetting {
                            key: key.clone(),
                            value: value.clone(),
                        })
                    {
                        let _ = self.action_tx.send(Action::Error(e.to_string()));
                    }
                }
                Action::StartServer => {
                    if !self.settings_ready {
                        let _ = self
                            .action_tx
                            .send(Action::Error("Settings not ready".into()));
                    } else if let Some(snap) = self.aether_settings.clone() {
                        let sup = self.aether_sup.clone();
                        tokio::spawn(async move {
                            // IMPORTANT: Keine .awaits hier drin; Guard nur kurz halten.
                            let mut guard = sup.lock().unwrap();
                            if guard.can_start() {
                                let _ = guard.start(snap);
                                true
                            } else {
                                false
                            }
                            // Guard fällt hier!
                        });
                    } else {
                        let _ = self
                            .action_tx
                            .send(Action::Error("No settings snapshot".into()));
                    }
                }

                Action::StopServer => {
                    self.aether_sup.lock().unwrap().stop();
                }

                Action::AetherEvent(evt) => {
                    match evt {
                        AetherToWizard::ServerStarted => {
                            self.server_running = true;
                            // Supervisor von Starting -> Running schalten (ohne neue Channels)
                            self.aether_sup.lock().unwrap().mark_started();
                            // Optional: Render triggern
                            let _ = self.action_tx.send(Action::Render);
                        }
                        AetherToWizard::ServerStopped => {
                            self.server_running = false;
                            let _ = self.action_tx.send(Action::Render);
                        }
                        AetherToWizard::Stats(snap) => {
                            self.stats_history.lock().unwrap().push(snap.clone());
                            // Optional: Render triggern
                            let _ = self.action_tx.send(Action::Render);
                        }
                        AetherToWizard::Error(e) => {
                            self.last_error = Some(e.clone());
                            tracing::error!("Error: {}", e);
                            let _ = self.action_tx.send(Action::Render);
                        }
                    }
                }

                Action::RestartServer => {
                    self.aether_sup
                        .lock()
                        .unwrap()
                        .restart(self.aether_settings.clone().unwrap_or_default());
                }

                Action::Error(msg) => {
                    self.last_error = Some(msg.clone());
                    tracing::error!("Error: {}", msg);
                }

                Action::Reload => {
                    // TODO: implement settings reload flow
                }
                Action::Save => {
                    // TODO: implement settings save flow
                }
                Action::IdleTimeout => {
                    // Handle idle timeout, e.g. navigate to login page
                    if self.current_page.as_deref() != Some("login") {
                        let _ = self.action_tx.send(Action::Navigate("login".to_string()));
                    }
                }
                Action::Help => {
                    // TODO: show help overlay
                }
            }

            if let Some(current) = &self.current_page {
                if let Some(page) = self.pages.get_mut(current) {
                    if let Some(action) = page.update(action_clone.clone())? {
                        self.action_tx.send(action)?
                    };
                }
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        let action_tx = self.action_tx.clone();
        tui.draw(|frame| {
            // Fill full-screen background with theme color
            // let bg = Block::default()
            //     .style(ratatui::style::Style::default().bg(self.theme.roles.background));
            // frame.render_widget(bg, frame.area());
            if let Some(current) = &self.current_page {
                if let Some(page) = self.pages.get_mut(current) {
                    let Some((keybind_scope, shortcuts)) = page.get_shortcuts() else {
                        // No shortcuts registered, just draw the page full area
                        if let Some(page_mut) = self.pages.get_mut(current) {
                            if let Err(err) = page_mut.draw(frame, frame.area()) {
                                let _ = action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", err)));
                            }
                        }
                        return;
                    };
                    let keybinds_block = Block::bordered()
                        .title(format!(" {keybind_scope} "))
                        .border_set(border::ROUNDED);
                    let keybind_render_width = keybinds_block.inner(frame.area()).width;
                    let keybinds = create_shortcut_list(shortcuts, keybind_render_width);
                    let keybind_len = keybinds.len() as u16;
                    let keybind_para =
                        Paragraph::new(Text::from_iter(keybinds)).block(keybinds_block);

                    let vertical = Layout::vertical([
                        Constraint::Percentage(100),
                        Constraint::Length(keybind_len + 2),
                    ])
                    .flex(Flex::Legacy)
                    .split(frame.area());

                    if let Err(err) = page.draw(frame, vertical[0]) {
                        let _ = action_tx.send(Action::Error(format!("Failed to draw: {:?}", err)));
                    }
                    frame.render_widget(keybind_para, vertical[1]);
                }
            }
        })?;
        Ok(())
    }

    pub fn theme(&self) -> &style::Theme {
        &self.theme
    }
}
