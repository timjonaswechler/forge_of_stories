use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
};

use crate::{
    action::{Action, UiOutcome},
    components::Component,
    components::popup::PopupComponent,
    tui::{EventResponse, Frame},
};

use super::{centered_rect_fixed, draw_popup_frame};

/// Simple modal alert popup with a title, a message and standard controls:
/// - Enter / Esc: acknowledge (emit UiOutcome::RequestClose)
///
/// Emits:
/// - Action::UiOutcome(UiOutcome::RequestClose) on Enter/Esc
/// Lifecycle: central loop interprets UiOutcome and closes the popup.
pub struct AlertPopup {
    title: String,
    message: String,
    min_width: u16,
    min_height: u16,
}

impl AlertPopup {
    pub fn new<T: Into<String>, M: Into<String>>(title: T, message: M) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            min_width: 60,
            min_height: 7,
        }
    }

    /// Set a minimum width for the popup dialog (default: 60).
    pub fn min_width(mut self, w: u16) -> Self {
        self.min_width = w.max(20); // clamp to a sane minimum
        self
    }

    /// Set a minimum height for the popup dialog (default: 7).
    pub fn min_height(mut self, h: u16) -> Self {
        self.min_height = h.max(5); // clamp to a sane minimum
        self
    }

    fn ok_action(&self) -> Action {
        Action::UiOutcome(UiOutcome::RequestClose)
    }

    fn inner_rect(area: Rect) -> Rect {
        // Compute the inner rect of a bordered block: shrink by 1 on each side if possible.
        let x = area.x.saturating_add(1);
        let y = area.y.saturating_add(1);
        let width = area.width.saturating_sub(2);
        let height = area.height.saturating_sub(2);
        Rect {
            x,
            y,
            width,
            height,
        }
    }
}

impl Component for AlertPopup {
    fn height_constraint(&self) -> Constraint {
        Constraint::Min(self.min_height)
    }

    fn keymap_context(&self) -> &'static str {
        "popup"
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<EventResponse<Action>>> {
        let action = match key.code {
            KeyCode::Enter | KeyCode::Esc => Some(self.ok_action()),
            _ => None,
        };
        Ok(action.map(EventResponse::Stop))
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Submit => Ok(Some(self.ok_action())),
            _ => Ok(None),
        }
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        if area.width < 5 || area.height < 5 {
            // Not enough space to draw a dialog; do nothing
            return Ok(());
        }

        let w = self.min_width.min(area.width);
        let h = self.min_height.min(area.height);
        let dialog = centered_rect_fixed(area, w, h);

        // Draw outer framed dialog (rounded borders, title)
        let _ = draw_popup_frame(f, dialog, &self.title);

        let inner = Self::inner_rect(dialog);

        // Compose content: message and footer hint
        let mut lines: Vec<Line> = Vec::new();

        for paragraph in self.message.lines() {
            lines.push(Line::from(Span::raw(paragraph)));
        }

        // Spacer
        if inner.height >= 3 {
            lines.push(Line::raw(""));
        }

        // Footer hints (dimmed)
        let hint = Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::White)),
            Span::raw(": OK   "),
            Span::styled("Esc", Style::default().fg(Color::White)),
            Span::raw(": Close"),
        ])
        .fg(Color::DarkGray);

        lines.push(hint);

        let text = Text::from(lines);
        let para = Paragraph::new(text).wrap(Wrap { trim: true });

        f.render_widget(para, inner);
        Ok(())
    }
}

impl PopupComponent for AlertPopup {}
