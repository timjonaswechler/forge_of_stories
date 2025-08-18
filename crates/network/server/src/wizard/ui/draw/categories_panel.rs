use super::super::colors::{c_accent, c_bg_panel, c_border, c_ok, c_text, c_text_dim};
use super::utils::get_category_icon;
use crate::wizard::app::{CategoryListItem, SettingsCategory, WizardApp};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
};

pub(in crate::wizard::ui) fn draw_categories_panel(
    f: &mut Frame,
    app: &WizardApp,
    area: Rect,
    active: bool,
) {
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
