use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};

use super::super::colors::{c_accent, c_accent2, c_bg_panel, c_border, c_ok, c_text, c_text_dim};
use crate::wizard::app::{CategoryListItem, WizardApp};
use tui_textarea::TextArea;

#[derive(Debug)]
struct CursorPosition {
    row: u16,
    col: u16,
}

pub(in crate::wizard::ui) fn draw_settings_panel(
    f: &mut Frame,
    app: &WizardApp,
    area: Rect,
    active: bool,
) {
    let (list_area, input_area) = if app.editing_setting {
        let input_height = calculate_input_height(&app.edit_input, area.height);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),                   // Settings list
                Constraint::Length(input_height + 2), // Input field + borders
            ])
            .split(area);
        (chunks[0], Some(chunks[1]))
    } else {
        (area, None)
    };

    let settings = app.get_current_category_settings();
    let mut list_items: Vec<ListItem> = Vec::new();

    if settings.is_empty() {
        list_items.push(ListItem::new(Line::from(Span::styled(
            "No settings for this category",
            Style::default().fg(c_text_dim()),
        ))));
    } else {
        for (index, setting) in settings.iter().enumerate() {
            let status_icon = if setting.completed { "✔" } else { "○" };

            let status_style = if setting.completed {
                Style::default().fg(c_ok()).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(c_text_dim())
            };

            let value_text: String = if setting.completed {
                setting
                    .value
                    .clone()
                    .unwrap_or_else(|| setting.default_value.clone())
            } else {
                setting.default_value.clone()
            };

            let current_value = if setting.completed {
                setting.value.as_ref().unwrap_or(&setting.default_value)
            } else {
                &setting.default_value
            };

            let is_default_value = current_value == &setting.default_value;

            let mut lines = vec![
                Line::from(vec![
                    Span::styled(format!("{status_icon} "), status_style),
                    Span::styled(
                        &setting.name,
                        Style::default().fg(c_text()).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("    "),
                    Span::styled(&setting.description, Style::default().fg(c_text_dim())),
                ]),
            ];

            // Add value lines (handle comma-separated values as separate lines)
            let value_style = if setting.completed {
                if is_default_value {
                    Style::default()
                        .fg(c_text_dim())
                        .add_modifier(Modifier::ITALIC)
                } else {
                    Style::default().fg(c_ok())
                }
            } else {
                Style::default()
                    .fg(c_text_dim())
                    .add_modifier(Modifier::ITALIC)
            };

            // Split into parts, but do NOT insert extra blank separator lines.
            let parts: Vec<&str> = value_text.split(',').collect();
            if parts.len() > 1 {
                lines.push(Line::from(Span::raw("    Value:")));
                for part in parts.iter() {
                    lines.push(Line::from(vec![
                        Span::raw("      • "),
                        Span::styled(part.trim().to_string(), value_style),
                    ]));
                }

                if !is_default_value && setting.completed {
                    lines.push(Line::from(vec![
                        Span::raw("    Default: "),
                        Span::styled(
                            &setting.default_value,
                            Style::default()
                                .fg(c_text_dim())
                                .add_modifier(Modifier::ITALIC),
                        ),
                    ]));
                }
            } else {
                lines.push(Line::from(vec![
                    Span::raw("    Value: "),
                    Span::styled(value_text, value_style),
                    if !is_default_value && setting.completed {
                        Span::styled(
                            format!(" ({})", setting.default_value),
                            Style::default()
                                .fg(c_text_dim())
                                .add_modifier(Modifier::ITALIC),
                        )
                    } else {
                        Span::raw("")
                    },
                ]));
            }

            // Add spacing between items except for the last one
            if index < settings.len() - 1 {
                lines.push(Line::from(""));
            }

            let item_style = if active && app.selected_setting == index {
                Style::default()
                    .bg(Color::Rgb(40, 46, 60))
                    .fg(c_accent())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            for line in lines {
                list_items.push(ListItem::new(line).style(item_style));
            }
        }
    }

    let title = if app.selected_category_item < app.category_list_items.len() {
        match &app.category_list_items[app.selected_category_item] {
            CategoryListItem::Category(category, _, _, _) => {
                format!(" {} ", category.display_name())
            }
            CategoryListItem::Subcategory(_, subcategory_name, _, _, _) => {
                format!(" {} ", subcategory_name)
            }
        }
    } else {
        " Settings ".to_string()
    };

    let border_style = if active {
        Style::default().fg(c_accent()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(c_border())
    };

    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .style(Style::default().bg(c_bg_panel()))
            .title(Span::styled(
                &title,
                Style::default()
                    .fg(c_accent2())
                    .add_modifier(Modifier::BOLD),
            )),
    );
    f.render_widget(list, list_area);

    // Render input field if editing
    if let Some(input_area) = input_area {
        // If we have an active TextArea in the app state, render it and use its cursor.
        if let Some(ta) = &app.edit_textarea {
            // Render the textarea widget directly; TextArea implements the Widget trait.
            f.render_widget(ta, input_area);

            // Get cursor from the textarea and position it within the input area.
            // TextArea::cursor() returns (usize, usize) as (row, col).
            let (row, col) = ta.cursor();
            #[allow(clippy::cast_possible_truncation)]
            f.set_cursor_position(Position::new(
                input_area.x + 1 + col as u16,
                input_area.y + 1 + row as u16,
            ));
        } else {
            // Fallback: render single-line/multi-line paragraph as before
            // Available content height (minus borders)
            let visible_lines = input_area.height.saturating_sub(2);
            let (display_text, cursor_pos) = format_comma_separated_input_with_scroll(
                &app.edit_input,
                app.edit_cursor_position,
                app.edit_scroll_offset,
                visible_lines,
            );

            let input = Paragraph::new(display_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(c_accent()).add_modifier(Modifier::BOLD))
                        .title(Span::styled(
                            " Edit Value ",
                            Style::default().fg(c_accent()).add_modifier(Modifier::BOLD),
                        )),
                )
                .style(Style::default().fg(c_text()));
            f.render_widget(input, input_area);

            // Set cursor position for input field
            #[allow(clippy::cast_possible_truncation)]
            f.set_cursor_position(Position::new(
                input_area.x + 1 + cursor_pos.col,
                input_area.y + 1 + cursor_pos.row,
            ));
        }
    }
}

fn calculate_input_height(input: &str, available_height: u16) -> u16 {
    // compute number of parts (at least 1)
    let parts = if input.is_empty() {
        1
    } else {
        input.split(',').count()
    };
    // user wanted the window to grow as entries are added; add one extra line so the user sees room
    let desired = (parts + 1) as u16;
    let max_height = (available_height / 2).max(2); // minimum 2 lines, max half of available
    desired.min(max_height)
}

// Format the input as multiple lines (one per comma-separated item) and compute cursor row/col.
// Important: mapping is done on character indices (not bytes). No extra blank separator lines are inserted.
fn format_comma_separated_input_with_scroll(
    input: &str,
    cursor_position: usize, // character index within input
    scroll_offset: u16,
    visible_lines: u16,
) -> (String, CursorPosition) {
    let mut display_lines: Vec<String> = Vec::new();
    let mut cursor_pos = CursorPosition { row: 0, col: 0 };

    // If empty, show a single empty line
    let parts: Vec<&str> = if input.is_empty() {
        vec![""]
    } else {
        input.split(',').collect()
    };

    // Build display lines and track char counts (characters, not bytes)
    let mut cum_chars = 0usize;
    for (i, part) in parts.iter().enumerate() {
        // Display trimmed part for nicer UI
        let trimmed = part.trim().to_string();
        display_lines.push(trimmed.clone());

        let part_len_chars = part.chars().count();
        // compute leading whitespace trimmed away so we can map cursor into visible col
        let leading_ws = part.chars().take_while(|c| c.is_whitespace()).count();

        // If cursor is within this part (based on character index)
        if cursor_position >= cum_chars && cursor_position <= cum_chars + part_len_chars {
            let relative = cursor_position.saturating_sub(cum_chars);
            // Map into trimmed: subtract leading whitespace that was trimmed for display
            let col = relative
                .saturating_sub(leading_ws)
                .min(trimmed.chars().count());
            cursor_pos.row = i as u16;
            cursor_pos.col = col as u16;
        }

        // advance cumulative count: part chars + possible comma
        cum_chars += part_len_chars;
        if i + 1 < parts.len() {
            cum_chars += 1; // comma counts as one character
        }
    }

    // Apply scrolling: select visible lines slice
    let start_line = scroll_offset as usize;
    let visible_slice: Vec<String> = display_lines
        .into_iter()
        .skip(start_line)
        .take(visible_lines as usize)
        .collect();

    // Adjust cursor row to visible coordinates
    cursor_pos.row = cursor_pos.row.saturating_sub(scroll_offset);

    (visible_slice.join("\n"), cursor_pos)
}
