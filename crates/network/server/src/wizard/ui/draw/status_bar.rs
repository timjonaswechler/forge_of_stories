use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use super::super::colors::{
    c_accent, c_bg_panel, c_border, c_err, c_ok, c_text, c_text_dim, c_warn,
};
use crate::wizard::app::{ActivePanel, Screen, StatusType, WizardApp};

pub(in crate::wizard::ui) fn draw_status_bar(f: &mut Frame, app: &WizardApp, area: Rect) {
    // Create the status bar background
    let status_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(c_border()))
        .style(Style::default().bg(c_bg_panel()));

    f.render_widget(status_block, area);

    // Split the area into left (status message) and right (key hints)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    // Left side: Status message or default context
    let (message, message_style) = get_status_message_and_style(app);
    let status_paragraph = Paragraph::new(Line::from(vec![
        Span::raw(" "), // Small padding
        Span::styled(message, message_style),
    ]))
    .style(Style::default().bg(c_bg_panel()))
    .wrap(Wrap { trim: true });

    f.render_widget(status_paragraph, chunks[0]);

    // Right side: Key hints
    let key_hints = get_key_hints_for_screen(app);
    let hints_paragraph = Paragraph::new(Line::from(key_hints))
        .style(Style::default().bg(c_bg_panel()).fg(c_border()))
        .alignment(Alignment::Right)
        .wrap(Wrap { trim: true });

    f.render_widget(hints_paragraph, chunks[1]);
}

fn get_status_message_and_style(app: &WizardApp) -> (String, Style) {
    if let Some(message) = &app.status_message {
        let style = match app.status_type {
            StatusType::Success => Style::default().fg(c_ok()).add_modifier(Modifier::BOLD),
            StatusType::Warning => Style::default().fg(c_warn()).add_modifier(Modifier::BOLD),
            StatusType::Error => Style::default().fg(c_err()).add_modifier(Modifier::BOLD),
            StatusType::Info => Style::default().fg(c_accent()),
        };
        (message.clone(), style)
    } else {
        // Default context message based on the current screen
        let context_message = match app.screen {
            Screen::Setup => {
                let (completed, total) = app.get_total_progress();
                format!(
                    "Setup Progress: {}/{} categories completed",
                    completed, total
                )
            }
            Screen::Overview => "Review your configuration and press Enter to save".to_string(),
        };
        (context_message, Style::default().fg(c_text_dim()))
    }
}

fn get_key_hints_for_screen(app: &WizardApp) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    let add_hint = |spans: &mut Vec<Span<'static>>,
                    key: &'static str,
                    action: &'static str,
                    emphasized: bool| {
        if !spans.is_empty() {
            spans.push(Span::raw(" "));
        }
        let key_style = if emphasized {
            Style::default().fg(c_accent()).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(c_text()).add_modifier(Modifier::BOLD)
        };
        spans.push(Span::styled(format!("[{key}]"), key_style));
        spans.push(Span::styled(
            format!(" {action}"),
            Style::default().fg(c_text_dim()),
        ));
    };

    match (app.screen, app.active_panel) {
        (Screen::Setup, ActivePanel::Categories) => {
            add_hint(&mut spans, "↑↓", "Navigate", false);
            add_hint(&mut spans, "→/Enter", "Configure", true);
            add_hint(&mut spans, "q", "Quit", false);
        }
        (Screen::Setup, ActivePanel::Settings) => {
            add_hint(&mut spans, "↑↓", "Navigate", false);
            add_hint(&mut spans, "Enter", "Confirm", true);
            add_hint(&mut spans, "e", "Edit", true);
            add_hint(&mut spans, "←", "Back", false);
            add_hint(&mut spans, "q", "Quit", false);
        }
        (Screen::Overview, _) => {
            add_hint(&mut spans, "Enter", "Save & Exit", true);
            add_hint(&mut spans, "Esc", "Back to Setup", false);
            add_hint(&mut spans, "q", "Quit", false);
        }
        _ => {}
    }

    // Add a trailing space for padding
    spans.push(Span::raw(" "));
    spans
}
