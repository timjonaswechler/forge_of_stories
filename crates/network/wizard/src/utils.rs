use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub(crate) fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let cw = width.min(r.width).max(1);
    let ch = height.min(r.height).max(1);
    let x = r.x + (r.width.saturating_sub(cw)) / 2;
    let y = r.y + (r.height.saturating_sub(ch)) / 2;
    Rect::new(x, y, cw, ch)
}
