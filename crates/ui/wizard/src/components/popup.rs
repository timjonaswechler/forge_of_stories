use ratatui::{
    layout::Rect,
    style::{Color, Style},
    symbols,
    widgets::{Block, Borders, Clear},
};

use crate::{action::Action, components::Component, tui::Frame};

/// Popup components and helpers for Wizard
///
/// This module provides:
/// - `PopupComponent` trait: a small extension over `Component` with popup-specific lifecycle hooks
/// - Rendering helpers: `render_backdrop`, `centered_rect_fixed`, `draw_popup_frame`
///
/// Intended usage:
/// 1) Draw the active page as usual
/// 2) If a popup is active:
///    - call `render_backdrop(frame, area)`
///    - compute a centered rect with `centered_rect_fixed(area, width, height)`
///    - call `draw_popup_frame(frame, popup_area, "Title")`
///    - draw your popup content inside the same `popup_area`
pub trait PopupComponent: Component {
    /// Whether the popup is modal (blocks page interactions). Defaults to true.
    fn is_modal(&self) -> bool {
        true
    }

    /// Action to emit when the popup is confirmed/submitted (e.g., Enter).
    /// Default closes the popup; specific popups can override to return a richer action.
    fn submit_action(&mut self) -> Option<Action> {
        Some(Action::ClosePopup)
    }

    /// Action to emit when the popup is cancelled/closed (e.g., Esc).
    /// Default closes the popup.
    fn cancel_action(&mut self) -> Option<Action> {
        Some(Action::ClosePopup)
    }
}

/// Render a modal-style backdrop that visually separates a popup from the underlying page.
/// Since terminals don't support real transparency, we simulate a dim overlay via background color.
///
/// Call this after drawing the main page, and before drawing the popup dialog.
pub fn render_backdrop(frame: &mut Frame<'_>, area: Rect) {
    // A solid background color to "dim" the content. Choose a very dark color.
    let backdrop = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(backdrop, area);
}

/// Compute a centered rectangle with a fixed width/height clamped to the available `area`.
///
/// - Ensures the returned width/height never exceed the available area.
/// - Centers the popup within `area`.
pub fn centered_rect_fixed(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);

    let x = area.x.saturating_add((area.width.saturating_sub(w)) / 2);
    let y = area.y.saturating_add((area.height.saturating_sub(h)) / 2);

    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}

/// Draw a rounded, bordered popup shell (frame) with a title at `area`.
/// This also clears the area to ensure underlying content doesn't bleed through.
///
/// Return value:
/// - The same `area` passed in, for ergonomic chaining.
pub fn draw_popup_frame(frame: &mut Frame<'_>, area: Rect, title: impl Into<String>) -> Rect {
    // Clear the dialog area before drawing
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {} ", title.into()))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .style(Style::default().fg(Color::White).bg(Color::Black));

    frame.render_widget(block, area);
    area
}
