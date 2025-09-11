use crate::components::Component;
use color_eyre::Result;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders},
};

/// A minimal overlay component (placeholder) for popups/notifications.
pub(crate) struct LayerOverlay {
    id: String,
    focused: bool,
}

impl LayerOverlay {
    pub(crate) fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            focused: false,
        }
    }
}

impl Component for LayerOverlay {
    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Overlay — {}", self.id));
        if self.focused {
            block = block.style(Style::default().fg(Color::Yellow));
        }
        f.render_widget(block, area);
        Ok(())
    }
}
