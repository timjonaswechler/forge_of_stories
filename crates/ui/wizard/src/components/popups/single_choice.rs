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
    components::{Component, PopupComponent},
    tui::Frame,
};

use super::{centered_rect_fixed, draw_popup_frame};
use strum::IntoEnumIterator;

/// SingleChoicePopup allows selecting exactly one item from a list of options.
/// It is generic over enums that implement `strum::IntoEnumIterator + Display + Clone + Send + 'static`.
///
/// Emits:
/// - Action::PopupResult(PopupResult::InputSubmitted(String)) on Enter (selected item's Display)
/// - Action::PopupResult(PopupResult::Cancelled) on Esc
///
/// Navigation:
/// - Up/Down: move selection
/// - Enter: submit choice
/// - Esc: cancel
pub struct SingleChoicePopup<T>
where
    T: IntoEnumIterator + core::fmt::Display + Clone + Send + 'static,
{
    title: String,
    options: Vec<T>,
    selected: usize,
    scroll: usize,
    min_width: u16,
    min_height: u16,
}

impl<T> SingleChoicePopup<T>
where
    T: IntoEnumIterator + core::fmt::Display + Clone + Send + 'static,
{
    /// Create a SingleChoice popup from the enum iterator (T::iter()).
    pub fn from_enum(title: impl Into<String>) -> Self {
        let options: Vec<T> = T::iter().collect();
        Self::with_options(title, options)
    }

    /// Create a SingleChoice popup from an explicit list of options.
    pub fn with_options(title: impl Into<String>, options: Vec<T>) -> Self {
        Self {
            title: title.into(),
            options,
            selected: 0,
            scroll: 0,
            min_width: 60,
            min_height: 12,
        }
    }

    pub fn min_width(mut self, w: u16) -> Self {
        self.min_width = w.max(30);
        self
    }

    pub fn min_height(mut self, h: u16) -> Self {
        self.min_height = h.max(8);
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

    fn visible_bounds(&self, inner: ratatui::layout::Rect) -> (usize, usize) {
        // Reserve 2 rows for hints at the bottom when space allows.
        let reserve = if inner.height > 6 { 2 } else { 1 };
        let max_visible = inner.height.saturating_sub(reserve).max(1) as usize;

        let total = self.options.len();
        if total == 0 {
            return (0, 0);
        }

        let start = self.scroll.min(self.selected).min(total.saturating_sub(1));
        let end = (start + max_visible).min(total);
        (start, end)
    }

    fn ensure_visible(&mut self, inner: ratatui::layout::Rect) {
        if self.options.is_empty() {
            self.scroll = 0;
            self.selected = 0;
            return;
        }

        let reserve = if inner.height > 6 { 2 } else { 1 };
        let max_visible = inner.height.saturating_sub(reserve).max(1) as usize;

        // Clamp selection
        if self.selected >= self.options.len() {
            self.selected = self.options.len().saturating_sub(1);
        }

        // Adjust scroll so that selection is visible
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + max_visible {
            self.scroll = self.selected + 1 - max_visible;
        }
    }

    fn submit_action(&self) -> Option<Action> {
        if self.options.is_empty() {
            return Some(Action::PopupResult(PopupResult::Cancelled));
        }
        let label = self.options[self.selected].to_string();
        Some(Action::PopupResult(PopupResult::InputSubmitted(label)))
    }

    fn cancel_action(&self) -> Option<Action> {
        Some(Action::PopupResult(PopupResult::Cancelled))
    }
}

impl<T> Component for SingleChoicePopup<T>
where
    T: IntoEnumIterator + core::fmt::Display + Clone + Send + 'static,
{
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
            KeyCode::Up => {
                if !self.options.is_empty() {
                    if self.selected == 0 {
                        self.selected = self.options.len().saturating_sub(1);
                    } else {
                        self.selected -= 1;
                    }
                    // We'll ensure visibility in draw where we know the rect
                }
                Ok(Some(crate::tui::EventResponse::Stop(Action::Update)))
            }
            KeyCode::Down => {
                if !self.options.is_empty() {
                    self.selected = (self.selected + 1) % self.options.len();
                }
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

        // Ensure selection is visible
        self.ensure_visible(inner);

        // Compose lines for options (with selection highlighting)
        let (start, end) = self.visible_bounds(inner);
        let mut lines: Vec<Line> = Vec::new();

        if self.options.is_empty() {
            lines.push(Line::from(Span::styled(
                "No options available",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            for (idx, item) in self.options[start..end].iter().enumerate() {
                let absolute_idx = start + idx;
                let label = item.to_string();

                if absolute_idx == self.selected {
                    // Selected line
                    lines.push(Line::from(vec![
                        Span::raw("> "),
                        Span::styled(
                            label,
                            Style::default().fg(Color::Black).bg(Color::White).bold(),
                        ),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(label, Style::default().fg(Color::White)),
                    ]));
                }
            }
        }

        // Footer hints
        if inner.height >= 3 {
            lines.push(Line::raw(""));
            let hints = Line::from(vec![
                Span::styled("Up/Down", Style::default().fg(Color::White)),
                Span::raw(": Select   "),
                Span::styled("Enter", Style::default().fg(Color::White)),
                Span::raw(": Choose   "),
                Span::styled("Esc", Style::default().fg(Color::White)),
                Span::raw(": Cancel"),
            ])
            .fg(Color::DarkGray);
            lines.push(hints);
        }

        let text = Text::from(lines);
        let para = Paragraph::new(text).wrap(Wrap { trim: true });

        // Clear inner area and render
        f.render_widget(Clear, inner);
        f.render_widget(para, inner);

        Ok(())
    }
}

impl<T> PopupComponent for SingleChoicePopup<T>
where
    T: IntoEnumIterator + core::fmt::Display + Clone + Send + 'static,
{
    fn submit_action(&mut self) -> Option<Action> {
        Self::submit_action(self)
    }

    fn cancel_action(&mut self) -> Option<Action> {
        Self::cancel_action(self)
    }
}
