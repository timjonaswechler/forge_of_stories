use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use super::super::colors::{c_accent, c_accent2, c_bg_panel, c_text, c_text_dim};

pub(in crate::wizard::ui) fn draw_header(f: &mut Frame, area: Rect) {
    let title = Line::from(vec![
        Span::styled(
            " Forge of Stories ",
            Style::default().fg(c_accent()).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled("Setup Wizard ", Style::default().fg(c_accent2())),
    ]);

    let bar = Block::default()
        .borders(Borders::BOTTOM)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(c_text_dim()))
        .style(Style::default().bg(c_bg_panel()));
    f.render_widget(bar, area);

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    let title_para = Paragraph::new(title).style(Style::default().fg(c_text()));
    f.render_widget(title_para, inner[0]);

    let version_info = Paragraph::new(Line::from(vec![
        Span::styled("v", Style::default().fg(c_text_dim())),
        Span::styled(
            env!("CARGO_PKG_VERSION"),
            Style::default().fg(c_accent()).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" © 2025", Style::default().fg(c_text_dim())),
    ]))
    .style(Style::default().fg(c_text()))
    .alignment(Alignment::Right);
    f.render_widget(version_info, inner[1]);
}
