#![allow(clippy::new_without_default)]
// Minimal FPS component stub to satisfy module declarations.
// This implements the `Component` trait but draws nothing.
// Replace with a proper FPS widget when needed.

use color_eyre::Result;
use ratatui::{Frame, layout::Rect};

use crate::components::Component;

/// No-op FPS component stub.
pub struct Fps;

impl Fps {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Fps {
    fn default() -> Self {
        Self
    }
}

impl Component for Fps {
    fn draw(&mut self, _frame: &mut Frame<'_>, _area: Rect) -> Result<()> {
        // Intentionally draw nothing for now.
        Ok(())
    }
}
