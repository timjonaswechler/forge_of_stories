use crate::components::Component;
use color_eyre::Result;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders},
};

/// A minimal overlay component (placeholder) for popups/notifications.
pub(crate) struct LayerOverlay {
    id: String,
}

impl LayerOverlay {
    pub(crate) fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

impl Component for LayerOverlay {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Overlay — {}", self.id));
        f.render_widget(block, area);
        Ok(())
    }
}
