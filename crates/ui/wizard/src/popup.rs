//! Popup system for modal dialogs and overlays
//!
//! Popups work similar to Pages but are modal and overlay the current content.
//! They support flexible sizing and positioning.

use crate::{action::Action, components::Component, tui::Event};
use color_eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{Frame, layout::Rect};
use tokio::sync::mpsc::UnboundedSender;

pub mod help;
pub use help::HelpPopup;

/// Popup sizing and positioning configuration
#[derive(Debug, Clone)]
pub enum PopupSize {
    /// Fixed pixel/character size
    Fixed { width: u16, height: u16 },
    /// Percentage of the parent area
    Percentage { width: u8, height: u8 }, // 0-100
    /// Fullscreen popup
    Fullscreen,
    /// Custom calculation based on content
    Custom,
}

/// Popup positioning configuration
#[derive(Debug, Clone)]
pub enum PopupPosition {
    /// Center the popup
    Center,
    /// Fixed position from top-left
    Fixed { x: u16, y: u16 },
    /// Relative position (percentage)
    Relative { x: u8, y: u8 }, // 0-100
    /// Custom positioning logic
    Custom,
}

/// Configuration for popup appearance and behavior
#[derive(Debug, Clone)]
pub struct PopupConfig {
    pub size: PopupSize,
    pub position: PopupPosition,
    pub modal: bool,     // If true, dims background
    pub closable: bool,  // If true, can be closed with Esc
    pub resizable: bool, // If true, popup can be resized
}

impl Default for PopupConfig {
    fn default() -> Self {
        Self {
            size: PopupSize::Percentage {
                width: 80,
                height: 80,
            },
            position: PopupPosition::Center,
            modal: true,
            closable: true,
            resizable: false,
        }
    }
}

/// A modal popup that can contain components and handle events.
/// Similar to Page trait but designed for overlays.
pub trait Popup {
    /// Downcast to Any for type-specific operations
    fn as_any(&self) -> &dyn std::any::Any;

    /// Downcast to Any for mutable type-specific operations
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    /// Provide the popup with an action sender so it can emit `Action`s.
    #[allow(unused_variables)]
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        Ok(())
    }

    /// Initialize the popup once on creation/activation.
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    /// Provide components owned by this popup for App registration.
    /// Return a vector of (stable_id, component).
    fn provide_components(&mut self) -> Vec<(String, Box<dyn Component>)> {
        Vec::new()
    }

    /// Called when the popup becomes active/visible.
    fn focus(&mut self) -> Result<()> {
        Ok(())
    }

    /// Called when the popup is no longer active/visible.
    fn unfocus(&mut self) -> Result<()> {
        Ok(())
    }

    /// Active keymap context for this popup (e.g. "help", "confirm", "settings").
    fn keymap_context(&self) -> &'static str {
        "popup"
    }

    /// Stable identifier for this popup used for management (must be unique across popups).
    fn id(&self) -> &'static str {
        "unknown"
    }

    /// Ordered list of component IDs for focus traversal within the popup.
    fn focus_order(&self) -> &'static [&'static str] {
        &[]
    }

    /// The currently focused component id within the popup.
    fn focused_component_id(&self) -> Option<&str> {
        None
    }

    /// Configuration for popup size, position, and behavior.
    fn config(&self) -> PopupConfig {
        PopupConfig::default()
    }

    /// Calculate the actual popup area based on the parent area and config.
    /// Override this for custom sizing logic.
    fn calculate_area(&self, parent_area: Rect) -> Rect {
        let config = self.config();

        let (width, height) = match config.size {
            PopupSize::Fixed { width, height } => (width, height),
            PopupSize::Percentage { width, height } => {
                let w = (parent_area.width * width as u16) / 100;
                let h = (parent_area.height * height as u16) / 100;
                (w, h)
            }
            PopupSize::Fullscreen => (parent_area.width, parent_area.height),
            PopupSize::Custom => {
                // Default to 80% if not overridden
                let w = (parent_area.width * 80) / 100;
                let h = (parent_area.height * 80) / 100;
                (w, h)
            }
        };

        let (x, y) = match config.position {
            PopupPosition::Center => {
                let x = parent_area.x + (parent_area.width.saturating_sub(width)) / 2;
                let y = parent_area.y + (parent_area.height.saturating_sub(height)) / 2;
                (x, y)
            }
            PopupPosition::Fixed { x, y } => (parent_area.x + x, parent_area.y + y),
            PopupPosition::Relative { x, y } => {
                let px = parent_area.x + (parent_area.width * x as u16) / 100;
                let py = parent_area.y + (parent_area.height * y as u16) / 100;
                (px, py)
            }
            PopupPosition::Custom => {
                // Default to center if not overridden
                let x = parent_area.x + (parent_area.width.saturating_sub(width)) / 2;
                let y = parent_area.y + (parent_area.height.saturating_sub(height)) / 2;
                (x, y)
            }
        };

        // Ensure popup fits within parent area
        let max_x = parent_area.x + parent_area.width;
        let max_y = parent_area.y + parent_area.height;
        let final_width = width.min(max_x.saturating_sub(x));
        let final_height = height.min(max_y.saturating_sub(y));

        Rect::new(x, y, final_width, final_height)
    }

    /// Compute the layout for this popup: mapping component IDs to sub-rectangles.
    /// Default: empty layout (App falls back to drawing components in the full area).
    fn layout(&self, area: Rect) -> PopupLayout {
        let _ = area;
        PopupLayout::empty()
    }

    /// Route an optional event to the popup. Return an action to send back to the app.
    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
        let action = match event {
            Some(Event::Key(key)) => self.handle_key_events(key)?,
            Some(Event::Mouse(mouse)) => self.handle_mouse_events(mouse)?,
            _ => None,
        };
        Ok(action)
    }

    /// Handle key events within the popup. Return an action to send back to the app.
    #[allow(unused_variables)]
    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        Ok(None)
    }

    /// Handle mouse events within the popup. Return an action to send back to the app.
    #[allow(unused_variables)]
    fn handle_mouse_events(&mut self, mouse: MouseEvent) -> Result<Option<Action>> {
        Ok(None)
    }

    /// Update this popup in response to an action broadcast by the app or other components.
    #[allow(unused_variables)]
    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        Ok(None)
    }

    /// Draw the popup to the provided area.
    /// The area provided here is already calculated based on config().
    #[allow(unused_variables)]
    fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        Ok(())
    }
}

/// Layout description for a Popup.
/// The App will consult this mapping when positioning components within the popup.
#[derive(Default)]
pub struct PopupLayout {
    pub regions: std::collections::HashMap<String, Rect>,
}

impl PopupLayout {
    pub fn empty() -> Self {
        Self {
            regions: std::collections::HashMap::new(),
        }
    }

    pub fn with(mut self, id: &str, rect: Rect) -> Self {
        self.regions.insert(id.to_string(), rect);
        self
    }
}
