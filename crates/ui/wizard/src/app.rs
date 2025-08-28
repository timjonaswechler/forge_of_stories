use crate::{
    action::Action,
    components::{Component, footer::FooterComponent},
    config::Config,
    pages::{Page, login::LoginPage},
    services::aether::{AetherSettingsSnapshot, AetherSupervisor},
    services::shortcuts::create_shortcut_list,
    state::{InputMode, State},
    tui::{EventResponse, Tui},
};
use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
    prelude::Rect,
    symbols::border,
    text::Text,
    widgets::{Block, Paragraph},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tokio::sync::mpsc;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Home,
}
pub struct App {
    pub config: Config,
    pub pages: Vec<Box<dyn Page>>,
    pub history: HashMap<String, Box<dyn Page>>,
    pub active_page: usize,
    pub footer: FooterComponent,
    pub popup: Option<Box<dyn Component>>,
    pub should_quit: bool,
    pub should_suspend: bool,
    pub last_tick_key_events: Vec<KeyEvent>,
    pub last_input_at: Instant,
    pub idle_timeout: Duration,
    pub mode: Mode,
    pub state: State,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::new()?;
        let state = State::new()?;
        let home = LoginPage::new()?;
        let mode = Mode::Home;

        Ok(Self {
            config,
            pages: vec![Box::new(home)],
            history: HashMap::default(),
            footer: FooterComponent::new(),
            active_page: 0,
            popup: None,
            should_quit: false,
            should_suspend: false,
            last_tick_key_events: Vec::new(),
            last_input_at: std::time::Instant::now(),
            idle_timeout: std::time::Duration::from_secs(5 * 60),
            mode,
            state,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();

        let mut tui = Tui::new()?;
        tui.enter()?;
        for page in self.pages.iter_mut() {
            page.register_action_handler(action_tx.clone())?;
        }

        for page in self.pages.iter_mut() {
            page.register_config_handler(self.config.clone())?;
        }

        for page in self.pages.iter_mut() {
            page.init(&self.state)?;
            page.focus()?;
        }

        self.footer.init(&self.state)?;
        loop {
            if let Some(e) = tui.next().await {
                if matches!(
                    e,
                    crate::tui::Event::Key(_)
                        | crate::tui::Event::Mouse(_)
                        | crate::tui::Event::Paste(_)
                        | crate::tui::Event::FocusGained
                ) {
                    self.last_input_at = std::time::Instant::now();
                }
                let mut stop_event_propagation = self
                    .popup
                    .as_mut()
                    .and_then(|pane| pane.handle_events(e.clone(), &mut self.state).ok())
                    .map(|response| match response {
                        Some(EventResponse::Continue(action)) => {
                            action_tx.send(action).ok();
                            false
                        }
                        Some(EventResponse::Stop(action)) => {
                            action_tx.send(action).ok();
                            true
                        }
                        _ => false,
                    })
                    .unwrap_or(false);
                stop_event_propagation = stop_event_propagation
                    || self
                        .pages
                        .get_mut(self.active_page)
                        .and_then(|page| page.handle_events(e.clone(), &mut self.state).ok())
                        .map(|response| match response {
                            Some(crate::tui::EventResponse::Continue(action)) => {
                                action_tx.send(action).ok();
                                false
                            }
                            Some(crate::tui::EventResponse::Stop(action)) => {
                                action_tx.send(action).ok();
                                true
                            }
                            _ => false,
                        })
                        .unwrap_or(false);

                stop_event_propagation = stop_event_propagation
                    || self
                        .footer
                        .handle_events(e.clone(), &mut self.state)
                        .map(|response| match response {
                            Some(crate::tui::EventResponse::Continue(action)) => {
                                action_tx.send(action).ok();
                                false
                            }
                            Some(crate::tui::EventResponse::Stop(action)) => {
                                action_tx.send(action).ok();
                                true
                            }
                            _ => false,
                        })
                        .unwrap_or(false);

                if !stop_event_propagation {
                    match e {
                        crate::tui::Event::Quit if self.state.input_mode == InputMode::Normal => {
                            action_tx.send(Action::Quit)?
                        }
                        crate::tui::Event::Tick => action_tx.send(Action::Tick)?,
                        crate::tui::Event::Render => action_tx.send(Action::Render)?,
                        crate::tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
                        crate::tui::Event::Key(key) => {
                            if let Some(keymap) = self.config.keybindings.get(&self.mode) {
                                if let Some(action) = keymap.get(&vec![key]) {
                                    action_tx.send(action.clone())?;
                                } else {
                                    // If the key was not handled as a single key action,
                                    // then consider it for multi-key combinations.
                                    self.last_tick_key_events.push(key);

                                    // Check for multi-key combinations
                                    if let Some(action) = keymap.get(&self.last_tick_key_events) {
                                        action_tx.send(action.clone())?;
                                    }
                                }
                            };
                        }
                        _ => {}
                    }
                }
            }

            while let Ok(action) = action_rx.try_recv() {
                if action != Action::Tick && action != Action::Render {
                    log::debug!("{action:?}");
                }
                match action {
                    Action::Tick => {
                        self.last_tick_key_events.drain(..);
                        if self.last_input_at.elapsed() >= self.idle_timeout {
                            action_tx.send(Action::IdleTimeout)?;
                            self.last_input_at = std::time::Instant::now();
                        }
                    }
                    Action::Quit if self.state.input_mode == InputMode::Normal => {
                        self.should_quit = true
                    }
                    Action::Suspend => self.should_suspend = true,
                    Action::Resume => self.should_suspend = false,
                    Action::Resize(w, h) => {
                        tui.resize(Rect::new(0, 0, w, h))?;
                        tui.draw(|f| {
                            self.render(f).unwrap_or_else(|err| {
                                action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", err)))
                                    .unwrap();
                            })
                        })?;
                    }
                    Action::Render => {
                        tui.draw(|f| {
                            self.render(f).unwrap_or_else(|err| {
                                action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", err)))
                                    .unwrap();
                            })
                        })?;
                    }

                    Action::OpenPopup => {
                        // let operation_ids = self
                        //     .state
                        //     .openapi_operations
                        //     .iter()
                        //     .filter(|operation_item| {
                        //         let op_id = operation_item.operation.operation_id.clone();
                        //         self.history
                        //             .keys()
                        //             .any(|operation_id| op_id.eq(&Some(operation_id.clone())))
                        //     })
                        //     .collect::<Vec<_>>();
                        // let history_popup = HistoryPane::new(operation_ids);
                        // self.popup = Some(Box::new(history_popup));
                    }
                    Action::ClosePopup => {
                        if self.popup.is_some() {
                            self.popup = None;
                        }
                    }
                    _ => {}
                }

                if let Some(popup) = &mut self.popup {
                    if let Some(action) = popup.update(action.clone(), &mut self.state)? {
                        action_tx.send(action)?
                    };
                } else if let Some(page) = self.pages.get_mut(self.active_page) {
                    if let Some(action) = page.handle_events(action.clone(), &mut self.state)? {
                        action_tx.send(action)?
                    };
                }

                if let Some(action) = self.footer.update(action.clone(), &mut self.state)? {
                    action_tx.send(action)?
                };
            }

            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                tui = crate::tui::Tui::new()?;
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame<'_>) -> Result<()> {
        let vertical_layout =
            Layout::vertical(vec![Constraint::Fill(1), Constraint::Length(3)]).split(frame.area());

        if let Some(page) = self.pages.get_mut(self.active_page) {
            page.draw(frame, vertical_layout[0], &self.state)?;
        };

        self.footer.draw(frame, vertical_layout[1], &self.state)?;
        Ok(())
    }
}
