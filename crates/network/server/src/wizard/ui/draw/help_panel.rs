use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use super::super::colors::*;
use super::utils::get_category_icon;
use crate::wizard::app::{CategoryListItem, WizardApp};

pub(in crate::wizard::ui) fn draw_help_panel(
    f: &mut Frame,
    app: &WizardApp,
    area: Rect,
    active: bool,
) {
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

    // Show subcategories for this category
    let subcategories_in_category: Vec<&str> = app
        .category_list_items
        .iter()
        .filter_map(|item| match item {
            CategoryListItem::Subcategory(cat, name, _, _, _) if cat == category => {
                Some(name.as_str())
            }
            _ => None,
        })
        .collect();

    if !subcategories_in_category.is_empty() {
        help_lines.extend([
            Line::default(),
            Line::from(Span::styled(
                "Subcategories:",
                Style::default()
                    .fg(c_text_dim())
                    .add_modifier(Modifier::BOLD),
            )),
        ]);

        for (i, subcategory_name) in subcategories_in_category.iter().enumerate() {
            let is_last = i == subcategories_in_category.len() - 1;
            let is_selected = match &app.category_list_items[app.selected_category_item] {
                CategoryListItem::Subcategory(_, selected_name, _, _, _) => {
                    selected_name == *subcategory_name
                }
                _ => false,
            };

            let (prefix, style) = if is_selected {
                (
                    if is_last { "  └─ " } else { "  ├─ " },
                    Style::default().fg(c_accent()).add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    if is_last { "  └─ " } else { "  ├─ " },
                    Style::default().fg(c_text()),
                )
            };

            help_lines.push(Line::from(vec![
                Span::raw(prefix),
                Span::styled(*subcategory_name, style),
            ]));
        }
    } else if let Some(subcategory_name) = subcategory_name {
        // Show current subcategory if we're in one (fallback for single subcategory view)
        help_lines.extend([
            Line::default(),
            Line::from(Span::styled(
                "Current Subcategory:",
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

    // Show settings for selected subcategory
    if let Some(subcategory_name) = subcategory_name {
        let subcategory_settings: Vec<&str> = app
            .settings_items
            .iter()
            .filter(|setting| {
                setting.category == *category
                    && setting.subcategory.as_ref() == Some(subcategory_name)
            })
            .map(|setting| setting.name.as_str())
            .collect();

        if !subcategory_settings.is_empty() {
            help_lines.extend([
                Line::default(),
                Line::from(Span::styled(
                    "Settings in this subcategory:",
                    Style::default()
                        .fg(c_text_dim())
                        .add_modifier(Modifier::BOLD),
                )),
            ]);

            for (i, setting_name) in subcategory_settings.iter().enumerate() {
                let is_last = i == subcategory_settings.len() - 1;
                let prefix = if is_last { "  └─ " } else { "  ├─ " };

                help_lines.push(Line::from(vec![
                    Span::raw(prefix),
                    Span::styled(*setting_name, Style::default().fg(c_text())),
                ]));
            }
        }
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
