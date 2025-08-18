use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use super::super::colors::{c_accent, c_bg_panel, c_ok, c_text, c_text_dim};
use super::utils::{centered_rect, get_category_icon};
use crate::wizard::app::WizardApp;

pub(in crate::wizard::ui) fn draw_overview(f: &mut Frame, app: &WizardApp, body: Rect) {
    let area = centered_rect(80, 80, body);

    let mut lines = vec![
        Line::from(Span::styled(
            "🎉 Setup Complete!",
            Style::default().fg(c_ok()).add_modifier(Modifier::BOLD),
        )),
        Line::default(),
        Line::from(Span::styled(
            "Configuration Summary:",
            Style::default().fg(c_text()).add_modifier(Modifier::BOLD),
        )),
        Line::default(),
    ];

    for category_progress in &app.categories {
        let icon = get_category_icon(&category_progress.category);

        lines.push(Line::from(vec![
            Span::styled("✔ ", Style::default().fg(c_ok())),
            Span::styled(format!("{icon} "), Style::default().fg(c_text())),
            Span::styled(
                category_progress.category.display_name(),
                Style::default().fg(c_text()).add_modifier(Modifier::BOLD),
            ),
        ]));

        // Show settings for this category
        let category_settings: Vec<_> = app
            .settings_items
            .iter()
            .filter(|s| s.category == category_progress.category && s.completed)
            .collect();

        for setting in category_settings {
            lines.push(Line::from(vec![
                Span::raw("    • "),
                Span::styled(&setting.name, Style::default().fg(c_text())),
                Span::raw(": "),
                Span::styled(
                    setting.value.as_ref().unwrap_or(&setting.default_value),
                    Style::default().fg(c_accent()),
                ),
            ]));
        }

        lines.push(Line::default());
    }

    lines.push(Line::from(Span::styled(
        "Press Enter to save and exit, Esc to go back",
        Style::default().fg(c_text_dim()),
    )));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(c_accent()))
                .style(Style::default().bg(c_bg_panel()))
                .title(Span::styled(
                    " Setup Complete ",
                    Style::default().fg(c_ok()).add_modifier(Modifier::BOLD),
                )),
        )
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(c_text()));
    f.render_widget(paragraph, area);
}
