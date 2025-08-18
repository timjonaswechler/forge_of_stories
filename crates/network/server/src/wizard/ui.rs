mod colors;
mod draw;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::Block,
};

use super::ui::{
    colors::*,
    draw::{
        centered_rect, draw_categories_panel, draw_header, draw_help_panel, draw_overview,
        draw_settings_panel, draw_status_bar,
    },
};
use crate::wizard::app::{ActivePanel, Screen, SettingsCategory, WizardApp};

pub(super) fn draw(f: &mut Frame, app: &mut WizardApp) {
    let size = f.area();
    let bg_block = Block::default().style(Style::default().bg(c_bg()));
    f.render_widget(bg_block, size);

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(0),    // Body
            Constraint::Length(1), // Status bar
        ])
        .split(size);

    draw_header(f, root[0]);
    draw_status_bar(f, app, root[2]);

    match app.screen {
        Screen::Setup => draw_setup(f, app, root[1]),
        Screen::Overview => draw_overview(f, app, root[1]),
    }
}

fn draw_setup(f: &mut Frame, app: &WizardApp, body: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(centered_rect(95, 90, body));

    match app.active_panel {
        ActivePanel::Categories => {
            draw_categories_panel(f, app, chunks[0], true);
            draw_help_panel(f, app, chunks[1], false);
        }
        ActivePanel::Settings => {
            draw_categories_panel(f, app, chunks[0], false);
            draw_settings_panel(f, app, chunks[1], true);
        }
        _ => {}
    }
}
