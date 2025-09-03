use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Constraint,
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
};

use crate::{
    action::{Action, PopupResult},
    components::{Component, PopupComponent},
    tui::{EventResponse, Frame},
};

use super::{centered_rect_fixed, draw_popup_frame};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    Ok,
    Cancel,
}

/// Modal confirmation popup with selectable OK/Cancel buttons.
///
/// Behavior:
/// - Arrow Left/Right or Tab/BackTab: switch selected button
/// - Enter: submit (emits Action::PopupResult with Confirmed/Cancelled depending on selection)
/// - Esc: cancel (emits Action::PopupResult(Cancelled))
///
/// Lifecycle:
/// - The popup emits Action::PopupResult(...).
/// - In `update`, the popup maps the PopupResult to Action::ClosePopup to keep lifecycle consistent.
///   The application should handle forwarding the result and then close the popup.
///
pub struct ConfirmPopup {
    title: String,
    question: String,
    ok_label: String,
    cancel_label: String,
    selected: Choice,
    min_width: u16,
    min_height: u16,
}

impl ConfirmPopup {
    pub fn new<T: Into<String>, Q: Into<String>>(title: T, question: Q) -> Self {
        Self {
            title: title.into(),
            question: question.into(),
            ok_label: "OK".into(),
            cancel_label: "Cancel".into(),
            selected: Choice::Ok,
            min_width: 60,
            min_height: 9,
        }
    }

    pub fn ok_label<S: Into<String>>(mut self, label: S) -> Self {
        self.ok_label = label.into();
        self
    }

    pub fn cancel_label<S: Into<String>>(mut self, label: S) -> Self {
        self.cancel_label = label.into();
        self
    }

    pub fn min_width(mut self, w: u16) -> Self {
        self.min_width = w.max(24);
        self
    }

    pub fn min_height(mut self, h: u16) -> Self {
        self.min_height = h.max(7);
        self
    }

    fn confirm_action(&self) -> Action {
        match self.selected {
            Choice::Ok => Action::PopupResult(PopupResult::Confirmed),
            Choice::Cancel => Action::PopupResult(PopupResult::Cancelled),
        }
    }

    fn cancel_action(&self) -> Action {
        Action::PopupResult(PopupResult::Cancelled)
    }

    fn toggle_selection(&mut self) {
        self.selected = match self.selected {
            Choice::Ok => Choice::Cancel,
            Choice::Cancel => Choice::Ok,
        };
    }

    fn inner_rect(area: ratatui::layout::Rect) -> ratatui::layout::Rect {
        let x = area.x.saturating_add(1);
        let y = area.y.saturating_add(1);
        let width = area.width.saturating_sub(2);
        let height = area.height.saturating_sub(2);
        ratatui::layout::Rect {
            x,
            y,
            width,
            height,
        }
    }
}

impl Component for ConfirmPopup {
    fn height_constraint(&self) -> Constraint {
        Constraint::Min(self.min_height)
    }

    fn keymap_context(&self) -> &'static str {
        "popup"
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<EventResponse<Action>>> {
        let action = match key.code {
            KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::BackTab => {
                self.toggle_selection();
                None
            }
            KeyCode::Enter => Some(self.confirm_action()),
            KeyCode::Esc => Some(ConfirmPopup::cancel_action(self)),
            _ => None,
        };
        Ok(action.map(EventResponse::Stop))
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            // Allow Enter via mapped Submit action too
            Action::Submit => Ok(Some(self.confirm_action())),
            // When the result gets re-injected into the action loop, close the popup.
            Action::PopupResult(PopupResult::Confirmed)
            | Action::PopupResult(PopupResult::Cancelled) => Ok(Some(Action::ClosePopup)),
            _ => Ok(None),
        }
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: ratatui::layout::Rect) -> Result<()> {
        if area.width < 5 || area.height < 5 {
            return Ok(());
        }

        let w = self.min_width.min(area.width);
        let h = self.min_height.min(area.height);
        let dialog = centered_rect_fixed(area, w, h);

        // Outer frame with title
        let _ = draw_popup_frame(f, dialog, &self.title);

        let inner = Self::inner_rect(dialog);

        // Compose message lines (wrapping)
        let mut lines: Vec<Line> = Vec::new();
        for l in self.question.lines() {
            lines.push(Line::from(Span::raw(l)));
        }

        // Spacer
        if inner.height >= 3 {
            lines.push(Line::raw(""));
        }

        // Buttons row (centered)
        let ok_style_selected = Style::default().fg(Color::Black).bg(Color::White).bold();
        let ok_style_unselected = Style::default().fg(Color::White).bg(Color::Black);
        let cancel_style_selected = ok_style_selected;
        let cancel_style_unselected = ok_style_unselected;

        let ok_span = if self.selected == Choice::Ok {
            Span::styled(format!("[ {} ]", self.ok_label), ok_style_selected)
        } else {
            Span::styled(format!("[ {} ]", self.ok_label), ok_style_unselected)
        };

        let cancel_span = if self.selected == Choice::Cancel {
            Span::styled(format!("[ {} ]", self.cancel_label), cancel_style_selected)
        } else {
            Span::styled(
                format!("[ {} ]", self.cancel_label),
                cancel_style_unselected,
            )
        };

        // Compute centered placement by padding with spaces
        let spacing = "   ";
        let buttons_len =
            (2 + self.ok_label.len() + 2) + spacing.len() + (2 + self.cancel_label.len() + 2);
        let total_width = inner.width as usize;
        let pad = total_width.saturating_sub(buttons_len) / 2;
        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::raw(" ".repeat(pad)));
        spans.push(ok_span);
        spans.push(Span::raw(spacing));
        spans.push(cancel_span);

        lines.push(Line::from(spans));

        // Footer hints (dimmed)
        if inner.height >= 4 {
            lines.push(Line::raw(""));
            let hints = Line::from(vec![
                Span::styled("←/→/Tab", Style::default().fg(Color::White)),
                Span::raw(": Select   "),
                Span::styled("Enter", Style::default().fg(Color::White)),
                Span::raw(": Confirm   "),
                Span::styled("Esc", Style::default().fg(Color::White)),
                Span::raw(": Cancel"),
            ])
            .fg(Color::DarkGray);
            lines.push(hints);
        }

        let text = Text::from(lines);
        let para = Paragraph::new(text).wrap(Wrap { trim: true });
        f.render_widget(para, inner);

        Ok(())
    }
}

impl PopupComponent for ConfirmPopup {
    fn submit_action(&mut self) -> Option<Action> {
        Some(self.confirm_action())
    }

    fn cancel_action(&mut self) -> Option<Action> {
        Some(ConfirmPopup::cancel_action(self))
    }
}
