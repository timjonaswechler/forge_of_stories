use super::super::SettingsCategory;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub(in crate::wizard::ui::draw) fn get_category_icon(category: &SettingsCategory) -> &'static str {
    match category {
        SettingsCategory::Network => "🌐",
        SettingsCategory::Security => "🔒",
        SettingsCategory::World => "🌍",
        SettingsCategory::Features => "🎨",
        SettingsCategory::Finished => "✅",
    }
}

pub(crate) fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    let h = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(v[1]);
    h[1]
}
