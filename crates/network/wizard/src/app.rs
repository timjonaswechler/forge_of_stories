use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use tokio::sync::mpsc;
use tracing::{debug, info};

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::{
    action::Action,
    config::Config,
    pages::{HomePage, LoginPage, Page},
    tui::{Event, Tui},
};

pub struct App {
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
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        let pages: HashMap<String, Box<dyn Page>> = {
            let mut m = HashMap::new();
            // Register login and home pages
            m.insert(
                "login".to_string(),
                Box::new(LoginPage::new()) as Box<dyn Page>,
            );
            m.insert(
                "home".to_string(),
                Box::new(HomePage::new()) as Box<dyn Page>,
            );
            m
        };

        Ok(Self {
            tick_rate,
            frame_rate,
            pages,
            current_page: Some("login".to_string()),
            should_quit: false,
            should_suspend: false,
            last_tick_key_events: Vec::new(),
            last_input_at: std::time::Instant::now(),
            idle_timeout: std::time::Duration::from_secs(5),
            config: Config::new()?,
            action_tx,
            action_rx,
        })
    }

    /// Register a page at runtime. This will also register the page's action & config handlers.
    pub fn register_page(&mut self, id: impl Into<String>, mut page: Box<dyn Page>) -> Result<()> {
        let id = id.into();
        // Register handlers so the page can send/receive actions immediately.
        page.register_action_handler(self.action_tx.clone())?;
        page.register_config_handler(self.config.clone())?;
        self.pages.insert(id, page);
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            // .mouse(true) // uncomment this line to enable mouse support
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        tui.enter()?;

        if let Some(current) = &self.current_page {
            if let Some(page) = self.pages.get_mut(current) {
                page.register_action_handler(self.action_tx.clone())?;
                page.register_config_handler(self.config.clone())?;
                page.init(tui.size()?)?;
                // Force an initial full redraw after first page init
                let _ = self.action_tx.send(Action::ClearScreen);
                let _ = self.action_tx.send(Action::Render);
            }
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
        let Some(keymap) = self.config.keybindings.get(self.current_page.as_deref()) else {
            return Ok(());
        };
        match keymap.get(&vec![key]) {
            Some(action) => {
                info!("Got action: {action:?}");
                action_tx.send(action.clone())?;
            }
            _ => {
                // If the key was not handled as a single key action,
                // then consider it for multi-key combinations.
                self.last_tick_key_events.push(key);

                // Check for multi-key combinations
                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                    info!("Got action: {action:?}");
                    action_tx.send(action.clone())?;
                }
            }
        }
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
                            let _ = page.init(tui.size()?);
                            let _ = page.on_enter();
                        }
                        // force a full redraw
                        let _ = self.action_tx.send(Action::ClearScreen);
                        let _ = self.action_tx.send(Action::Render);
                    }
                }
                Action::Resize(w, h) => self.handle_resize(tui, *w, *h)?,
                Action::Render => self.render(tui)?,
                _ => {}
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
            if let Some(current) = &self.current_page {
                if let Some(page) = self.pages.get_mut(current) {
                    if let Err(err) = page.draw(frame, frame.area()) {
                        let _ = action_tx.send(Action::Error(format!("Failed to draw: {:?}", err)));
                    }
                }
            }
        })?;
        Ok(())
    }
}
