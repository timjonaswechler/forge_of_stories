#![allow(clippy::new_without_default)]
// Minimal Home component stub to satisfy module declarations.
// This implements the `Component` trait and draws a simple placeholder.
// Replace or extend with real content and interactions as needed.

use color_eyre::Result;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::components::Component;

/// Basic "Home" component that draws a bordered, centered title.
pub struct Home {
    title: String,
    focused: bool,
}

impl Home {
    pub fn new() -> Self {
        Self {
            title: "Home".to_string(),
            focused: false,
        }
    }
}

impl Default for Home {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Home {
    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> Result<()> {
        let mut block = Block::default().borders(Borders::ALL).title(Span::styled(
            &self.title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
        if self.focused {
            block = block.style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        }

        let content = Paragraph::new(Line::from(vec![
            Span::raw("Welcome to "),
            Span::styled(
                "Wizard",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — this is a placeholder Home component."),
        ]))
        .alignment(Alignment::Center)
        .block(block);

        frame.render_widget(content, area);
        Ok(())
    }
}
