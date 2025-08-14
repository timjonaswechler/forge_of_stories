use color_eyre::owo_colors::OwoColorize;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{self, Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, Paragraph, Row, Table, Tabs, Wrap,
    },
};

use crate::wizard::app::{
    ActivePanel, App, CategoryListItem, Screen, SettingsCategory, StatusType,
};

// --- Palette ---
const fn c_bg() -> Color {
    Color::Rgb(24, 26, 33)
}
const fn c_bg_panel() -> Color {
    Color::Rgb(24, 26, 33)
}
const fn c_border() -> Color {
    Color::Gray
}
const fn c_accent() -> Color {
    Color::Green
} // cyan-ish
const fn c_accent2() -> Color {
    Color::Yellow
} // purple
const fn c_ok() -> Color {
    Color::Rgb(120, 220, 120)
}
const fn c_warn() -> Color {
    Color::LightYellow
}
const fn c_err() -> Color {
    Color::Red
}
const fn c_text() -> Color {
    Color::Rgb(220, 224, 232)
}
const fn c_text_dim() -> Color {
    Color::Rgb(140, 145, 160)
}

fn truncate_text(text: &str, max_width: usize) -> String {
    if text.chars().count() <= max_width {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_width.saturating_sub(3)).collect();
        format!("{truncated}...")
    }
}

pub(super) fn draw(f: &mut Frame, app: &mut App) {
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

fn draw_header(f: &mut Frame, area: Rect) {
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

fn draw_setup(f: &mut Frame, app: &App, body: Rect) {
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

fn draw_categories_panel(f: &mut Frame, app: &App, area: Rect, active: bool) {
    let mut list_items: Vec<ListItem> = Vec::new();
    let (completed_categories, total_categories) = app.get_total_progress();

    for (index, category_list_item) in app.category_list_items.iter().enumerate() {
        match category_list_item {
            CategoryListItem::Category(category, completed_count, total_count, is_completed) => {
                let icon = get_category_icon(category);
                let status_icon = if *is_completed {
                    "✔" // Green checkmark
                } else {
                    "○" // Empty circle
                };

                let status_style = if *is_completed {
                    Style::default().fg(c_ok()).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(c_text_dim())
                };

                let item_line = Line::from(vec![
                    Span::styled(format!("{status_icon} "), status_style),
                    Span::styled(format!("{icon} "), Style::default().fg(c_text())),
                    Span::styled(
                        category.display_name(),
                        Style::default().fg(c_text()).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        format!("({completed_count}/{total_count})"),
                        Style::default().fg(c_text_dim()),
                    ),
                ]);

                let item_style = if active && app.selected_category_item == index {
                    Style::default()
                        .bg(Color::Rgb(40, 46, 60))
                        .fg(c_accent())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                list_items.push(ListItem::new(item_line).style(item_style));
            }
            CategoryListItem::Subcategory(
                parent_category,
                subcategory_name,
                completed_count,
                total_count,
                is_completed,
            ) => {
                let status_icon = if *is_completed {
                    "✔" // Green checkmark
                } else {
                    "○" // Empty circle
                };

                let status_style = if *is_completed {
                    Style::default().fg(c_ok()).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(c_text_dim())
                };

                // Check if this is the last subcategory of its parent category
                let is_last_subcategory = is_last_subcategory_of_category(
                    &app.category_list_items,
                    index,
                    *parent_category,
                );

                let tree_symbol = if is_last_subcategory {
                    "└─ " // Last subcategory: └─
                } else {
                    "├─ " // Middle subcategory: ├─
                };

                let item_line = Line::from(vec![
                    Span::raw("  "), // Indentation for subcategories
                    Span::styled(format!("{status_icon} "), status_style),
                    Span::styled(tree_symbol, Style::default().fg(c_text_dim())), // Tree-like connector
                    Span::styled(subcategory_name, Style::default().fg(c_text())),
                    Span::raw(" "),
                    Span::styled(
                        format!("({completed_count}/{total_count})"),
                        Style::default().fg(c_text_dim()),
                    ),
                ]);

                let item_style = if active && app.selected_category_item == index {
                    Style::default()
                        .bg(Color::Rgb(40, 46, 60))
                        .fg(c_accent())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                list_items.push(ListItem::new(item_line).style(item_style));
            }
        }
    }

    let title = format!(" Categories ({completed_categories}/{total_categories}) ");
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
                Style::default().fg(c_accent()).add_modifier(Modifier::BOLD),
            )),
    );
    f.render_widget(list, area);
}

fn draw_settings_panel(f: &mut Frame, app: &App, area: Rect, active: bool) {
    let (list_area, input_area) = if app.editing_setting {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Settings list
                Constraint::Length(3), // Input field
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
            let status_icon = if setting.completed {
                "✔" // Green checkmark
            } else {
                "○" // Empty circle
            };

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
                Line::from(vec![
                    Span::raw("    Value: "),
                    Span::styled(
                        value_text,
                        if setting.completed {
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
                        },
                    ),
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
                ]),
            ];

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
        let input = Paragraph::new(app.edit_input.as_str())
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
            input_area.x + app.edit_cursor_position as u16 + 1,
            input_area.y + 1,
        ));
    }
}

fn draw_help_panel(f: &mut Frame, app: &App, area: Rect, active: bool) {
    if app.selected_category_item >= app.category_list_items.len() {
        return;
    }

    let (category, subcategory_name) = match &app.category_list_items[app.selected_category_item] {
        CategoryListItem::Category(category, _, _, _) => (category, None),
        CategoryListItem::Subcategory(category, subcategory_name, _, _, _) => {
            (category, Some(subcategory_name))
        }
    };

    let mut help_lines = vec![
        Line::from(Span::styled(
            "Current Selection",
            Style::default()
                .fg(c_text_dim())
                .add_modifier(Modifier::BOLD),
        )),
        Line::default(),
    ];

    // Show category info
    help_lines.extend([
        Line::from(vec![
            Span::styled(get_category_icon(category), Style::default().fg(c_text())),
            Span::raw(" "),
            Span::styled(
                category.display_name(),
                Style::default().fg(c_text()).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::default(),
        Line::from(Span::styled(
            category.description(),
            Style::default().fg(c_text()),
        )),
    ]);

    // Show subcategory info if applicable
    if let Some(subcategory_name) = subcategory_name {
        help_lines.extend([
            Line::default(),
            Line::from(Span::styled(
                "Subcategory:",
                Style::default()
                    .fg(c_text_dim())
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::raw("  └─ "),
                Span::styled(
                    subcategory_name,
                    Style::default().fg(c_accent()).add_modifier(Modifier::BOLD),
                ),
            ]),
        ]);
    }

    help_lines.extend([
        Line::default(),
        Line::default(),
        Line::from(Span::styled(
            "Navigation",
            Style::default()
                .fg(c_text_dim())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "↑↓ ",
                Style::default().fg(c_accent()).add_modifier(Modifier::BOLD),
            ),
            Span::raw("Navigate categories/subcategories"),
        ]),
        Line::from(vec![
            Span::styled(
                "→/Enter ",
                Style::default().fg(c_accent()).add_modifier(Modifier::BOLD),
            ),
            Span::raw("Configure settings"),
        ]),
        Line::from(vec![
            Span::styled(
                "q ",
                Style::default().fg(c_accent()).add_modifier(Modifier::BOLD),
            ),
            Span::raw("Quit setup"),
        ]),
    ]);

    let border_style = if active {
        Style::default().fg(c_accent()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(c_border())
    };

    let help = Paragraph::new(help_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style)
                .style(Style::default().bg(c_bg_panel()))
                .title(Span::styled(
                    " Help ",
                    Style::default()
                        .fg(c_accent2())
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(c_text()));
    f.render_widget(help, area);
}

fn draw_overview(f: &mut Frame, app: &App, body: Rect) {
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

fn get_category_icon(category: &SettingsCategory) -> &'static str {
    match category {
        SettingsCategory::Network => ("🌐"),
        SettingsCategory::Database => ("🗄"),
        SettingsCategory::Security => ("🔒"),
        SettingsCategory::Storage => ("💾"),
        SettingsCategory::Logging => ("📄"),
        SettingsCategory::Performance => ("⚡"),
        SettingsCategory::Features => ("🎨"),
    }
}

fn is_last_subcategory_of_category(
    category_list_items: &[CategoryListItem],
    current_index: usize,
    parent_category: SettingsCategory,
) -> bool {
    // Look ahead to see if there are more subcategories of the same parent category
    for i in (current_index + 1)..category_list_items.len() {
        match &category_list_items[i] {
            CategoryListItem::Subcategory(cat, _, _, _, _) if *cat == parent_category => {
                // Found another subcategory of the same parent, so current is not last
                return false;
            }
            CategoryListItem::Category(_, _, _, _) => {
                // Hit a new main category, so current subcategory is the last of its parent
                return true;
            }
            _ => continue,
        }
    }
    // Reached end of list, so current is the last subcategory
    true
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
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

fn get_status_message_and_style(app: &App) -> (String, Style) {
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

fn get_key_hints_for_screen(app: &App) -> Vec<Span<'static>> {
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

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
