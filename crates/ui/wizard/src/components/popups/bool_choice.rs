use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Constraint,
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Clear, Paragraph, Wrap},
};

use crate::{
    action::{Action, PopupResult},
    components::Component,
    components::popup::PopupComponent,
    tui::Frame,
};

use super::{centered_rect_fixed, draw_popup_frame};

/// BoolChoicePopup is a small modal dialog for selecting a boolean value.
///
/// - Navigation:
///   - Left/Right/Up/Down/Tab: toggle between the two choices
///   - Enter: submit the current choice (emits `PopupResult::InputSubmitted("true"|"false")`)
///   - Esc: cancel (emits `PopupResult::Cancelled`)
///
/// - Labels:
///   - `true_label` (default: "Yes")
///   - `false_label` (default: "No")
///
/// - Result:
///   - On submit: `Action::PopupResult(PopupResult::InputSubmitted("true" | "false"))`
///   - On cancel: `Action::PopupResult(PopupResult::Cancelled)`
pub struct BoolChoicePopup {
    title: String,
    question: Option<String>,
    true_label: String,
    false_label: String,
    value: bool, // true => left button highlighted is "true_label"
    min_width: u16,
    min_height: u16,
}

impl BoolChoicePopup {
    pub fn new<T: Into<String>>(title: T) -> Self {
        Self {
            title: title.into(),
            question: None,
            true_label: "Yes".into(),
            false_label: "No".into(),
            value: true,
            min_width: 50,
            min_height: 9,
        }
    }

    pub fn question<Q: Into<String>>(mut self, question: Q) -> Self {
        self.question = Some(question.into());
        self
    }

    pub fn true_label<S: Into<String>>(mut self, label: S) -> Self {
        self.true_label = label.into();
        self
    }

    pub fn false_label<S: Into<String>>(mut self, label: S) -> Self {
        self.false_label = label.into();
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

    fn inner_rect(area: ratatui::layout::Rect) -> ratatui::layout::Rect {
        ratatui::layout::Rect {
            x: area.x.saturating_add(1),
            y: area.y.saturating_add(1),
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        }
    }

    fn toggle(&mut self) {
        self.value = !self.value;
    }

    fn submit_action(&self) -> Option<Action> {
        let val = if self.value { "true" } else { "false" };
        Some(Action::PopupResult(PopupResult::InputSubmitted(
            val.to_string(),
        )))
    }

    fn cancel_action(&self) -> Option<Action> {
        Some(Action::PopupResult(PopupResult::Cancelled))
    }
}

impl Component for BoolChoicePopup {
    fn height_constraint(&self) -> Constraint {
        Constraint::Min(self.min_height)
    }

    fn keymap_context(&self) -> &'static str {
        "popup"
    }

    fn handle_key_events(
        &mut self,
        key: KeyEvent,
    ) -> Result<Option<crate::tui::EventResponse<Action>>> {
        match key.code {
            KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down | KeyCode::Tab => {
                self.toggle();
                Ok(Some(crate::tui::EventResponse::Stop(Action::Update)))
            }
            KeyCode::Enter => Ok(Some(crate::tui::EventResponse::Stop(
                self.submit_action().unwrap_or(Action::Update),
            ))),
            KeyCode::Esc => Ok(Some(crate::tui::EventResponse::Stop(
                self.cancel_action().unwrap_or(Action::Update),
            ))),
            _ => Ok(None),
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Submit => Ok(self.submit_action()),
            Action::PopupResult(PopupResult::InputSubmitted(_))
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

        // Compose lines (question, buttons, hints)
        let mut lines: Vec<Line> = Vec::new();

        if let Some(q) = &self.question {
            for l in q.lines() {
                lines.push(Line::from(Span::raw(l.to_string())));
            }
            if inner.height >= 3 {
                lines.push(Line::raw(""));
            }
        }

        // Buttons row (centered)
        let selected_style = Style::default().fg(Color::Black).bg(Color::White).bold();
        let unselected_style = Style::default().fg(Color::White).bg(Color::Black);

        let true_span = if self.value {
            Span::styled(format!("[ {} ]", self.true_label), selected_style)
        } else {
            Span::styled(format!("[ {} ]", self.true_label), unselected_style)
        };

        let false_span = if !self.value {
            Span::styled(format!("[ {} ]", self.false_label), selected_style)
        } else {
            Span::styled(format!("[ {} ]", self.false_label), unselected_style)
        };

        let spacing = "   ";
        let buttons_len =
            (2 + self.true_label.len() + 2) + spacing.len() + (2 + self.false_label.len() + 2);
        let total_width = inner.width as usize;
        let pad = total_width.saturating_sub(buttons_len) / 2;

        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::raw(" ".repeat(pad)));
        spans.push(true_span);
        spans.push(Span::raw(spacing));
        spans.push(false_span);

        lines.push(Line::from(spans));

        // Footer hints (dimmed)
        if inner.height >= 4 {
            lines.push(Line::raw(""));
            let hints = Line::from(vec![
                Span::styled("←/→/↑/↓/Tab", Style::default().fg(Color::White)),
                Span::raw(": Toggle   "),
                Span::styled("Enter", Style::default().fg(Color::White)),
                Span::raw(": Submit   "),
                Span::styled("Esc", Style::default().fg(Color::White)),
                Span::raw(": Cancel"),
            ])
            .fg(Color::DarkGray);
            lines.push(hints);
        }

        // Render text
        let text = Text::from(lines);
        let para = Paragraph::new(text).wrap(Wrap { trim: true });

        f.render_widget(Clear, inner);
        f.render_widget(para, inner);

        Ok(())
    }
}

impl PopupComponent for BoolChoicePopup {
    fn submit_action(&mut self) -> Option<Action> {
        Self::submit_action(self)
    }

    fn cancel_action(&mut self) -> Option<Action> {
        Self::cancel_action(self)
    }
}
