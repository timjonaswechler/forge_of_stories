use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span, Text},
    widgets::{Block, Clear, Paragraph},
};

use crate::{
    action::Action,
    components::Component,
    components::popup::PopupComponent,
    theme::{Theme, UiGroup},
    tui::Frame,
};

/// A simple overlay that lists the current context's keymap and allows selecting an action.
/// - Up/Down to navigate
/// - Enter to emit the selected action (mapped via label)
/// - Esc to close
pub struct KeymapOverlay {
    title: String,
    // (label, chords)
    entries: Vec<(String, Vec<String>)>,
    selected: usize,
}

impl KeymapOverlay {
    pub fn new(title: impl Into<String>, entries: Vec<(String, Vec<String>)>) -> Self {
        Self {
            title: title.into(),
            entries,
            selected: 0,
        }
    }
}

impl Component for KeymapOverlay {
    fn name(&self) -> &'static str {
        "keymap_overlay"
    }

    fn height_constraint(&self) -> Constraint {
        // Not used in layout (popup overlays control their own area), but trait requires it
        Constraint::Min(1)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let theme = Theme::from_env_auto();
        // Overlay covers bottom 1/3 of the screen (clamped at min height)
        let overlay_h = (area.height / 3).max(6).min(area.height);
        let overlay = Rect {
            x: area.x,
            y: area.y + area.height - overlay_h,
            width: area.width,
            height: overlay_h,
        };

        f.render_widget(Clear, overlay);

        let block = Block::bordered()
            .border_set(ratatui::symbols::border::ROUNDED)
            .border_style(theme.style(UiGroup::Border))
            .title(Span::styled(
                format!(" {} ", self.title),
                theme.style(UiGroup::Title),
            ));

        let inner = block.inner(overlay);
        f.render_widget(block, overlay);

        // Split into two columns: labels on left, chords on right
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(inner);

        let mut left: Vec<Line> = Vec::new();
        let mut right: Vec<Line> = Vec::new();
        for (i, (label, chords)) in self.entries.iter().enumerate() {
            let is_sel = i == self.selected;
            let lstyle = if is_sel {
                theme.style(UiGroup::ModeNormal)
            } else {
                theme.style(UiGroup::Statusline)
            };
            left.push(Line::from(Span::styled(label.clone(), lstyle)));
            let chord_line = chords.join(" / ");
            right.push(Line::from(Span::styled(
                chord_line,
                theme.style(UiGroup::Dimmed),
            )));
        }

        f.render_widget(
            Paragraph::new(Text::from(left)).wrap(ratatui::widgets::Wrap { trim: true }),
            cols[0],
        );
        f.render_widget(
            Paragraph::new(Text::from(right)).wrap(ratatui::widgets::Wrap { trim: true }),
            cols[1],
        );
        Ok(())
    }

    fn handle_key_events(
        &mut self,
        key: KeyEvent,
    ) -> Result<Option<crate::tui::EventResponse<Action>>> {
        match key.code {
            KeyCode::Up => {
                if self.selected == 0 {
                    self.selected = self.entries.len().saturating_sub(1);
                } else {
                    self.selected -= 1;
                }
                Ok(Some(crate::tui::EventResponse::Continue(Action::Update)))
            }
            KeyCode::Down => {
                self.selected = (self.selected + 1) % self.entries.len().max(1);
                Ok(Some(crate::tui::EventResponse::Continue(Action::Update)))
            }
            KeyCode::Enter => {
                if let Some((label, _)) = self.entries.get(self.selected) {
                    if let Some(a) = crate::ui::keymap::map_label_to_action(label) {
                        return Ok(Some(crate::tui::EventResponse::Stop(a)));
                    }
                }
                Ok(Some(crate::tui::EventResponse::Stop(Action::ClosePopup)))
            }
            KeyCode::Esc => Ok(Some(crate::tui::EventResponse::Stop(Action::ClosePopup))),
            _ => Ok(None),
        }
    }
}

impl PopupComponent for KeymapOverlay {}
