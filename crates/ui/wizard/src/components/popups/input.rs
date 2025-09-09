use color_eyre::Result;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    action::{Action, PopupResult},
    components::{Component, PopupComponent},
    tui::Frame,
};

use super::{centered_rect_fixed, draw_popup_frame};

/// Popup for single-line text input with optional validation.
/// - Enter: submit (validates and emits Action::PopupResult(PopupResult::InputSubmitted(String)))
/// - Esc: cancel (emits Action::PopupResult(PopupResult::Cancelled))
/// - Other keystrokes edit the input field (and stop event propagation)
pub struct InputPopup {
    title: String,
    label: String,
    input: Input,
    error: Option<String>,
    validator: Option<Box<dyn Fn(&str) -> std::result::Result<(), String> + Send + Sync + 'static>>,
    min_width: u16,
    min_height: u16,
}

impl InputPopup {
    pub fn new<T: Into<String>, L: Into<String>, V: Into<String>>(
        title: T,
        label: L,
        initial_value: V,
        validator: Option<
            Box<dyn Fn(&str) -> std::result::Result<(), String> + Send + Sync + 'static>,
        >,
    ) -> Self {
        let mut _input = Input::default();
        // Best-effort set initial value. If this API differs, adjust to your local version.
        _input = Input::new(initial_value.into());
        Self {
            title: title.into(),
            label: label.into(),
            input: _input,
            error: None,
            validator,
            min_width: 60,
            min_height: 9,
        }
    }

    pub fn min_width(mut self, w: u16) -> Self {
        self.min_width = w.max(30);
        self
    }

    pub fn min_height(mut self, h: u16) -> Self {
        self.min_height = h.max(7);
        self
    }

    fn inner_rect(area: Rect) -> Rect {
        Rect {
            x: area.x.saturating_add(1),
            y: area.y.saturating_add(1),
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        }
    }

    fn validate_current(&mut self) -> std::result::Result<(), String> {
        if let Some(v) = &self.validator {
            v(self.input.value())
        } else {
            Ok(())
        }
    }

    fn submit(&mut self) -> Option<Action> {
        match self.validate_current() {
            Ok(()) => {
                self.error = None;
                Some(Action::PopupResult(PopupResult::InputSubmitted(
                    self.input.value().to_string(),
                )))
            }
            Err(msg) => {
                self.error = Some(msg);
                Some(Action::Update)
            }
        }
    }

    fn input_box_area(inner: Rect) -> Rect {
        // Place input box roughly in the upper half, leaving room for label above and errors/hints below
        let box_height = 3; // border + 1 content line + border
        let y = inner.y.saturating_add(2).min(
            inner
                .y
                .saturating_add(inner.height.saturating_sub(box_height)),
        );
        Rect {
            x: inner.x,
            y,
            width: inner.width,
            height: box_height,
        }
    }
}

impl Component for InputPopup {
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
        // Interpret Enter/Esc directly; edit input on other keys.
        match key.code {
            KeyCode::Enter => {
                // Delegate to update via Submit to keep behavior consistent
                Ok(Some(crate::tui::EventResponse::Stop(Action::Submit)))
            }
            KeyCode::Esc => Ok(Some(crate::tui::EventResponse::Stop(Action::PopupResult(
                PopupResult::Cancelled,
            )))),
            _ => {
                // Let tui-input handle the keystroke and stop further propagation.
                self.input.handle_event(&CrosstermEvent::Key(key));
                Ok(Some(crate::tui::EventResponse::Stop(Action::Update)))
            }
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            // Enter (or mapped submit)
            Action::Submit => Ok(self.submit()),
            // Close when results are reinjected
            Action::PopupResult(PopupResult::InputSubmitted(_))
            | Action::PopupResult(PopupResult::Cancelled) => Ok(Some(Action::ClosePopup)),
            _ => Ok(None),
        }
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        if area.width < 5 || area.height < 5 {
            return Ok(());
        }

        let w = self.min_width.min(area.width);
        let h = self.min_height.min(area.height);
        let dialog = centered_rect_fixed(area, w, h);

        // Outer frame w/ title
        let _ = draw_popup_frame(f, dialog, &self.title);
        let inner = Self::inner_rect(dialog);

        // Clear inner area to avoid bleed
        f.render_widget(Clear, inner);

        // Label
        let label_line = Paragraph::new(Text::from(Line::from(vec![
            Span::styled(&self.label, Style::default().bold()),
            Span::raw(":"),
        ])))
        .wrap(Wrap { trim: true });
        f.render_widget(label_line, Rect { height: 1, ..inner });

        // Input box
        let input_box_area = Self::input_box_area(inner);
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded);
        let input_inner = input_block.inner(input_box_area);
        f.render_widget(input_block, input_box_area);

        // Render current input value (truncate to fit)
        let value = self.input.value().to_string();
        let mut visible = value.clone();
        if input_inner.width > 0 && visible.len() as u16 > input_inner.width {
            // crude truncation; no grapheme awareness here
            let take = input_inner.width as usize;
            visible = visible
                .chars()
                .take(take.saturating_sub(1))
                .collect::<String>();
        }

        let value_line = Paragraph::new(Text::from(Span::raw(visible.clone())));
        f.render_widget(value_line, input_inner);

        // Set cursor if there's room inside the input area
        if input_inner.width > 0 && input_inner.height > 0 {
            // Attempt to position the cursor at the logical column
            let cursor_col =
                (self.input.visual_cursor() as u16).min(input_inner.width.saturating_sub(1));
            f.set_cursor_position((input_inner.x + cursor_col, input_inner.y));
        }

        // Error message (if any)
        if let Some(err) = &self.error {
            let y = input_box_area
                .y
                .saturating_add(input_box_area.height)
                .min(inner.y + inner.height.saturating_sub(1));
            let err_rect = Rect {
                x: inner.x,
                y,
                width: inner.width,
                height: 1,
            };
            let err_line = Paragraph::new(Text::from(Span::styled(
                err.as_str(),
                Style::default().fg(Color::Red),
            )));
            f.render_widget(err_line, err_rect);
        }

        // Footer hints
        let hints_y = inner.y + inner.height.saturating_sub(2);
        if hints_y > inner.y + 1 {
            let hints_rect = Rect {
                x: inner.x,
                y: hints_y,
                width: inner.width,
                height: 1,
            };
            let hints = Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::White)),
                Span::raw(": Submit   "),
                Span::styled("Esc", Style::default().fg(Color::White)),
                Span::raw(": Cancel"),
            ])
            .fg(Color::DarkGray);
            let hints_para = Paragraph::new(Text::from(hints));
            f.render_widget(hints_para, hints_rect);
        }

        Ok(())
    }
}

impl PopupComponent for InputPopup {
    fn submit_action(&mut self) -> Option<Action> {
        self.submit()
    }

    fn cancel_action(&mut self) -> Option<Action> {
        Some(Action::PopupResult(PopupResult::Cancelled))
    }
}
