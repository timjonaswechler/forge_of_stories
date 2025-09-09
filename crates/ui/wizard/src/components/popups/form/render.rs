use color_eyre::Result;
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::tui::Frame;

use super::{FormField, FormFieldKind, FormPopup, FormState};
use crate::components::popups::{centered_rect_fixed, draw_popup_frame};

/// Diagnostic / introspection data produced during rendering.
/// (Not strictly required for functionality; useful for future tests or
/// logging / debugging.)
#[derive(Debug, Clone)]
pub struct FormRenderMetrics {
    pub total_fields: usize,
    pub visible_start: usize,
    pub visible_end: usize,
    pub focused_index: usize,
    pub visible_count: usize,
    pub scroll: usize,
    pub thumb_y: Option<usize>,
    pub track_height: Option<u16>,
}

/// Pure helper (Task 6.3 target) computing the vertical thumb position for a
/// scrollbar-like indicator.
///
/// Arguments:
/// - `total`        : total number of items (fields)
/// - `visible`      : number of items that can be shown simultaneously
/// - `scroll`       : current scroll offset (0 <= scroll < total)
/// - `track_height` : height in terminal cells of the track area
///
/// Returns:
/// - `Some(y)` where 0 <= y < track_height  (integral thumb row)
/// - `None` if no scrollbar is needed (e.g., total <= visible or degenerate sizes)
pub fn compute_scrollbar_thumb(
    total: usize,
    visible: usize,
    scroll: usize,
    track_height: u16,
) -> Option<usize> {
    if track_height == 0 {
        return None;
    }
    if total == 0 || visible == 0 || total <= visible {
        return None;
    }

    let max_thumb_y = track_height.saturating_sub(1) as usize;
    let denom = total.saturating_sub(visible).max(1);
    let ratio = (scroll as f32) / (denom as f32);
    let thumb_y = (ratio * (max_thumb_y as f32)).round() as usize;
    Some(thumb_y.min(max_thumb_y))
}

/// Render the form popup (extracted from the former monolithic `FormPopup::draw`).
///
/// This function performs all layout & drawing but mutates only:
/// - `popup.last_inner_height` (for page navigation heuristics)
/// - reads other internal fields (focused, scroll, editing, input, state)
///
/// NOTE: No behavioral changes vs. the original implementation (Phase 6.1).
pub fn render_form_popup(
    popup: &mut FormPopup,
    f: &mut Frame<'_>,
    area: Rect,
) -> Result<FormRenderMetrics> {
    if area.width < 5 || area.height < 5 {
        return Ok(FormRenderMetrics {
            total_fields: popup.field_count(),
            visible_start: 0,
            visible_end: 0,
            focused_index: popup.focused_index(),
            visible_count: 0,
            scroll: popup.scroll(),
            thumb_y: None,
            track_height: None,
        });
    }

    // Compute dialog rectangle & draw shell
    let w = popup.schema().min_width.min(area.width);
    let h = popup.schema().min_height.min(area.height);
    let dialog = centered_rect_fixed(area, w, h);
    let _ = draw_popup_frame(f, dialog, &popup.schema().title);

    // Inner drawable area (inside frame)
    let inner = Rect {
        x: dialog.x.saturating_add(1),
        y: dialog.y.saturating_add(1),
        width: dialog.width.saturating_sub(2),
        height: dialog.height.saturating_sub(2),
    };
    f.render_widget(Clear, inner);

    popup.set_last_inner_height(inner.height);

    // Collect lines
    let mut lines: Vec<Line> = Vec::new();

    // (Phase 6.1 fix) We intentionally delay binding schema/state to local variables
    // to avoid keeping immutable borrows alive across later mutable borrows
    // (e.g. popup.ensure_visible). We will access popup.schema()/popup.state()
    // inline below until after ensure_visible().

    // Description
    if let Some(desc) = &popup.schema().description {
        for l in desc.lines() {
            lines.push(Line::from(Span::styled(
                l.to_string(),
                Style::default().fg(Color::Gray),
            )));
        }
        lines.push(Line::raw(""));
    }

    // Global errors
    if !popup.state().global_errors.is_empty() {
        lines.push(
            Line::from("Errors:").style(
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ),
        );
        for e in &popup.state().global_errors {
            lines.push(Line::from(Span::styled(
                format!("• {}", e),
                Style::default().fg(Color::Red),
            )));
        }
        lines.push(Line::raw(""));
    }

    // Ensure focused field is visible; compute window
    popup.ensure_visible(inner.height);
    let (start, end) = popup.visible_bounds(inner.height);
    // Safe to bind schema/state after mutable borrows above.
    let schema = popup.schema();
    let state = popup.state();

    for (idx, field) in schema.fields[start..end].iter().enumerate() {
        let absolute_idx = start + idx;
        let focused = absolute_idx == popup.focused_index();

        // Field label
        let mut label_spans = vec![Span::styled(
            format!("{}:", field.label),
            Style::default().fg(Color::White).add_modifier(if focused {
                ratatui::style::Modifier::BOLD
            } else {
                ratatui::style::Modifier::empty()
            }),
        )];

        // Field value (or input editing)
        let value = if focused && popup.is_editing() && (field.is_textual() || field.is_list()) {
            popup.input_value().to_string()
        } else {
            popup.field_display_value(field)
        };

        label_spans.push(Span::raw(" "));
        let value_style = if focused {
            Style::default().fg(Color::Black).bg(Color::White)
        } else {
            Style::default().fg(Color::Cyan)
        };
        label_spans.push(Span::styled(value, value_style));

        lines.push(Line::from(label_spans));

        // Help
        if let Some(h) = &field.help {
            lines.push(Line::from(Span::styled(
                h,
                Style::default().fg(Color::DarkGray),
            )));
        }

        // Field error
        if let Some(err) = state.errors.get(&field.key) {
            lines.push(Line::from(Span::styled(
                err,
                Style::default().fg(Color::Red),
            )));
        }

        // Spacer
        lines.push(Line::raw(""));
    }

    // Footer hints
    lines.push(Line::raw(""));
    let footer = Line::from(vec![
        Span::styled("Up/Down", Style::default().fg(Color::White)),
        Span::raw(": Navigate   "),
        Span::styled("Enter", Style::default().fg(Color::White)),
        Span::raw(": "),
        if popup.is_editing() {
            Span::raw("Confirm edit   ")
        } else {
            Span::raw("Submit   ")
        },
        Span::styled("Esc", Style::default().fg(Color::White)),
        Span::raw(": Cancel   "),
        Span::styled("Left/Right", Style::default().fg(Color::White)),
        Span::raw(": Toggle/Select   "),
        Span::styled("Insert", Style::default().fg(Color::White)),
        Span::raw(": Add list item"),
    ])
    .fg(Color::DarkGray);
    lines.push(footer);

    let text = Text::from(lines);
    let para = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(Style::default()),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(para, inner);

    // Scrollbar / track
    let total = popup.field_count();
    let reserve = if inner.height > 8 { 4 } else { 2 };
    let visible = inner.height.saturating_sub(reserve).max(3) as usize;

    let (thumb_y, track_height_opt) = if total > 0 && total > visible && inner.width >= 1 {
        let track_rect = Rect {
            x: inner.x + inner.width.saturating_sub(1),
            y: inner.y,
            width: 1,
            height: inner.height,
        };

        if let Some(thumb) =
            compute_scrollbar_thumb(total, visible, popup.scroll(), track_rect.height)
        {
            // Build track lines
            let mut track_lines: Vec<Line> = Vec::new();
            for i in 0..track_rect.height {
                if i as usize == thumb {
                    track_lines.push(Line::from(Span::styled(
                        "█",
                        Style::default().fg(Color::Gray),
                    )));
                } else {
                    track_lines.push(Line::from(Span::styled(
                        "│",
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
            let track_para = Paragraph::new(Text::from(track_lines)).wrap(Wrap { trim: false });
            f.render_widget(track_para, track_rect);
            (Some(thumb), Some(track_rect.height))
        } else {
            (None, Some(track_rect.height))
        }
    } else {
        (None, None)
    };

    Ok(FormRenderMetrics {
        total_fields: total,
        visible_start: start,
        visible_end: end,
        focused_index: popup.focused_index(),
        visible_count: end.saturating_sub(start),
        scroll: popup.scroll(),
        thumb_y,
        track_height: track_height_opt,
    })
}

/// (Helper) Format a field's current display value.
/// Delegates to value logic that originally lived inside the popup implementation.
/// Kept here for situations where external code wants to do partial custom rendering
/// without re‑implementing the value formatting rules.
///
/// NOTE: This mirrors the original internal rules; consider consolidating with
/// `FormPopup::field_display_value` if duplication arises.
pub fn format_field_value(state: &FormState, field: &FormField) -> String {
    match &field.kind {
        FormFieldKind::Text | FormFieldKind::Path | FormFieldKind::Number => {
            state.get_value(&field.key).unwrap_or("").to_string()
        }
        FormFieldKind::Secret => {
            let len = state.get_value(&field.key).unwrap_or("").len();
            if len == 0 {
                "".to_string()
            } else {
                "•".repeat(len)
            }
        }
        FormFieldKind::Bool => {
            if state.get_value(&field.key).unwrap_or("false") == "true" {
                "true".into()
            } else {
                "false".into()
            }
        }
        FormFieldKind::Select { options } => {
            let v = state
                .get_value(&field.key)
                .unwrap_or_else(|| options.get(0).map(|s| s.as_str()).unwrap_or(""));
            v.to_string()
        }
        FormFieldKind::ListString => {
            let items = state.get_list(&field.key).unwrap_or(&[]);
            if items.is_empty() {
                "".to_string()
            } else {
                items.join(", ")
            }
        }
    }
}
