use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;
use std::{any::Any, sync::Arc};

use crate::pages::Page;

pub struct SetupPage {
    stats_history: Option<Arc<dyn Any + Send + Sync>>,
}

impl SetupPage {
    pub fn new() -> Self {
        Self {
            stats_history: None,
        }
    }
}

impl Default for SetupPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for SetupPage {
    fn name(&self) -> &str {
        "settup"
    }
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        Ok(())
    }
}
