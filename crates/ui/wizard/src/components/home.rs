use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{prelude::*, widgets::*};
use std::collections::VecDeque;
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{action::Action, config::Config};

#[derive(Default)]
pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,

    // Dashboard state
    server_running: bool,
    uptime_secs: u64,
    players: usize,
    last_error: Option<String>,
    players_history: VecDeque<u64>,
}

impl Home {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for Home {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        // Use scoped keybindings for home page/component
        let scoped = self
            .config
            .keybindings
            .get_scoped(Some("home"), Some("home"));
        if let Some(action) = scoped.get(&vec![key]) {
            return Ok(Some(action.clone()));
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Tick => {
                // no-op for now
            }
            Action::Render => {
                // no-op for now
            }
            // Action::ServerStarted => {
            //     self.server_running = true;
            //     self.last_error = None;
            // }
            // Action::ServerStopped => {
            //     self.server_running = false;
            // }
            Action::Error(msg) => {
                self.last_error = Some(msg);
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let status = if self.server_running {
            "[RUNNING]"
        } else {
            "[STOPPED]"
        };
        let body = format!(
            "Status: {status}\nUptime: {}s\nPlayers: {}\nLast error: {}",
            self.uptime_secs,
            self.players,
            self.last_error.as_deref().unwrap_or("-"),
        );

        // Split area: top for status, bottom for sparkline
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(4),
                ratatui::layout::Constraint::Min(0),
            ])
            .split(area);

        // Status box
        let status_widget =
            Paragraph::new(body).block(Block::default().title("Server").borders(Borders::ALL));
        frame.render_widget(status_widget, chunks[0]);

        // Players sparkline from history
        let data: Vec<u64> = self.players_history.iter().copied().collect();
        let max_val = data.iter().copied().max().unwrap_or(1);
        let spark = Sparkline::default()
            .block(
                Block::default()
                    .title("Players (history)")
                    .borders(Borders::ALL),
            )
            .data(&data)
            .max(max_val)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(spark, chunks[1]);

        Ok(())
    }
}
