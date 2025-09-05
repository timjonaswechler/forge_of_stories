pub mod certificate;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Constraint,
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use std::collections::HashMap;
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    action::{Action, PopupResult},
    components::Component,
    components::popups::{centered_rect_fixed, draw_popup_frame},
    tui::Frame,
};

/// A single form field kind supported by the FormPopup.
///
/// Notes:
/// - Text/Secret/Path/Number render as single-line editors (Secret draws value obfuscated, not hidden in state)
/// - Bool toggles with Left/Right/Space
/// - Select cycles between options with Left/Right
/// - ListString supports adding entries via Insert (editing), Enter to confirm; shows as a comma-joined list
#[derive(Debug, Clone)]
pub enum FormFieldKind {
    Text,
    Secret,
    Path,
    Number,
    Bool,
    Select { options: Vec<String> },
    ListString,
}

/// Declarative description of a form field.

pub struct FormField {
    pub key: String,
    pub label: String,
    pub kind: FormFieldKind,
    pub help: Option<String>,
    pub validator: Option<Box<dyn Fn(&str) -> std::result::Result<(), String> + Send + Sync>>,
}

impl FormField {
    pub fn new(key: impl Into<String>, label: impl Into<String>, kind: FormFieldKind) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            kind,
            help: None,
            validator: None,
        }
    }

    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn validator(
        mut self,
        f: impl Fn(&str) -> std::result::Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        self.validator = Some(Box::new(f));
        self
    }

    pub fn is_textual(&self) -> bool {
        matches!(
            self.kind,
            FormFieldKind::Text
                | FormFieldKind::Secret
                | FormFieldKind::Path
                | FormFieldKind::Number
        )
    }

    pub fn is_list(&self) -> bool {
        matches!(self.kind, FormFieldKind::ListString)
    }
}

/// Declarative schema for a multi-field form.

pub struct FormSchema {
    pub title: String,
    pub description: Option<String>,
    pub fields: Vec<FormField>,
    pub min_width: u16,
    pub min_height: u16,
}

impl FormSchema {
    pub fn new(title: impl Into<String>, fields: Vec<FormField>) -> Self {
        Self {
            title: title.into(),
            description: None,
            fields,
            min_width: 60,
            min_height: 16,
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn min_size(mut self, w: u16, h: u16) -> Self {
        self.min_width = w.max(40);
        self.min_height = h.max(10);
        self
    }
}

/// Mutable state captured while editing a form.
#[derive(Default, Clone)]
pub struct FormState {
    /// Scalar values for textual and single-choice fields. For Bool: "true"/"false".
    pub values: HashMap<String, String>,
    /// List values (e.g., SANs) keyed by field key.
    pub lists: HashMap<String, Vec<String>>,
    /// Per-field validation errors.
    pub errors: HashMap<String, String>,
    /// Global (cross-field) validation errors.
    pub global_errors: Vec<String>,
}

impl FormState {
    pub fn set_value(&mut self, key: &str, value: impl Into<String>) {
        self.values.insert(key.to_string(), value.into());
    }
    pub fn get_value(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }
    pub fn set_list(&mut self, key: &str, items: Vec<String>) {
        self.lists.insert(key.to_string(), items);
    }
    pub fn get_list(&self, key: &str) -> Option<&[String]> {
        self.lists.get(key).map(|v| v.as_slice())
    }
}

/// A multi-input popup (form) that supports basic navigation, validation, and JSON result.
pub struct FormPopup {
    schema: FormSchema,
    state: FormState,

    // UI state
    focused: usize,
    scroll: usize,
    editing: bool,
    input: Input,
    // Remember last inner height from draw() to compute page size for PageUp/PageDown
    last_inner_height: u16,
}

impl FormPopup {
    pub fn new(schema: FormSchema) -> Self {
        Self {
            schema,
            state: FormState::default(),
            focused: 0,
            scroll: 0,
            editing: false,
            input: Input::default(),
            last_inner_height: 0,
        }
    }

    pub fn with_state(mut self, state: FormState) -> Self {
        self.state = state;
        self
    }

    fn field_count(&self) -> usize {
        self.schema.fields.len()
    }

    fn current_field(&self) -> Option<&FormField> {
        self.schema.fields.get(self.focused)
    }

    fn current_field_mut(&mut self) -> Option<&mut FormField> {
        self.schema.fields.get_mut(self.focused)
    }

    fn focus_next(&mut self) {
        if self.field_count() == 0 {
            return;
        }
        self.focused = (self.focused + 1) % self.field_count();
    }

    fn focus_prev(&mut self) {
        if self.field_count() == 0 {
            return;
        }
        if self.focused == 0 {
            self.focused = self.field_count() - 1;
        } else {
            self.focused -= 1;
        }
    }

    fn visible_bounds(&self, inner_height: u16) -> (usize, usize) {
        // Reserve some rows for description/errors/footer; compute max visible
        let reserve = if inner_height > 8 { 4 } else { 2 };
        let max_visible = inner_height.saturating_sub(reserve).max(3) as usize;

        let total = self.field_count();
        if total == 0 {
            return (0, 0);
        }

        let start = self.scroll.min(self.focused).min(total.saturating_sub(1));
        let end = (start + max_visible).min(total);
        (start, end)
    }

    fn ensure_visible(&mut self, inner_height: u16) {
        let reserve = if inner_height > 8 { 4 } else { 2 };
        let max_visible = inner_height.saturating_sub(reserve).max(3) as usize;
        if self.focused < self.scroll {
            self.scroll = self.focused;
        } else if self.focused >= self.scroll + max_visible {
            self.scroll = self.focused + 1 - max_visible;
        }
    }

    fn toggle_bool(&mut self, key: &str) {
        let cur = self.state.get_value(key).unwrap_or("false");
        let next = if cur == "true" { "false" } else { "true" };
        self.state.set_value(key, next);
    }

    fn cycle_select(&mut self, key: &str, options: &[String], dir: i32) {
        if options.is_empty() {
            return;
        }
        let cur = self
            .state
            .get_value(key)
            .unwrap_or_else(|| options[0].as_str());
        let idx = options.iter().position(|o| o == cur).unwrap_or(0) as i32;
        let len = options.len() as i32;
        let next = (idx + dir).rem_euclid(len) as usize;
        self.state.set_value(key, options[next].clone());
    }

    fn start_editing(&mut self) {
        let (is_textual, is_list, existing_value) = if let Some(field) = self.current_field() {
            (
                field.is_textual(),
                field.is_list(),
                self.state.get_value(&field.key).map(|s| s.to_string()),
            )
        } else {
            return;
        };
        self.editing = true;
        self.input = Input::default();
        if is_textual {
            if let Some(v) = existing_value {
                self.input = self.input.clone().with_value(v);
            }
        } else if is_list {
            // Start with empty new item
            self.input = self.input.clone().with_value(String::new());
        }
    }

    fn cancel_editing(&mut self) {
        self.editing = false;
        self.input = Input::default();
    }

    fn commit_editing(&mut self) {
        let (key, is_textual, is_list) = if let Some(field) = self.current_field() {
            (field.key.clone(), field.is_textual(), field.is_list())
        } else {
            self.editing = false;
            self.input = Input::default();
            return;
        };
        let value = self.input.value().to_string();
        if is_textual {
            self.state.set_value(&key, value);
        } else if is_list {
            if !value.trim().is_empty() {
                let entry = self.state.lists.entry(key).or_insert_with(Vec::new);
                entry.push(value);
            }
        }
        self.editing = false;
        self.input = Input::default();
    }

    fn field_display_value(&self, field: &FormField) -> String {
        match &field.kind {
            FormFieldKind::Text | FormFieldKind::Path | FormFieldKind::Number => {
                self.state.get_value(&field.key).unwrap_or("").to_string()
            }
            FormFieldKind::Secret => {
                let len = self.state.get_value(&field.key).unwrap_or("").len();
                if len == 0 {
                    "".to_string()
                } else {
                    "•".repeat(len)
                }
            }
            FormFieldKind::Bool => {
                if self.state.get_value(&field.key).unwrap_or("false") == "true" {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            FormFieldKind::Select { options } => {
                let v = self
                    .state
                    .get_value(&field.key)
                    .unwrap_or_else(|| options.get(0).map(|s| s.as_str()).unwrap_or(""));
                v.to_string()
            }
            FormFieldKind::ListString => {
                let items = self.state.get_list(&field.key).unwrap_or(&[]);
                if items.is_empty() {
                    "".to_string()
                } else {
                    items.join(", ")
                }
            }
        }
    }

    fn validate(&mut self) -> bool {
        self.state.errors.clear();
        self.state.global_errors.clear();

        for f in &self.schema.fields {
            let v_owned = match f.kind {
                FormFieldKind::ListString => {
                    // For lists, allow empty; per-field validator can enforce rules
                    let items = self.state.get_list(&f.key).unwrap_or(&[]);
                    if let Some(val) = &f.validator {
                        for item in items {
                            if let Err(msg) = (val)(item) {
                                self.state.errors.insert(f.key.clone(), msg);
                                break;
                            }
                        }
                    }
                    continue;
                }
                _ => self.state.get_value(&f.key).unwrap_or("").to_string(),
            };
            let v = v_owned.as_str();

            if let Some(val) = &f.validator {
                if let Err(msg) = (val)(v) {
                    self.state.errors.insert(f.key.clone(), msg);
                }
            }

            // Built-in sanity for Number
            if matches!(f.kind, FormFieldKind::Number) && !v.is_empty() {
                if v.parse::<i64>().is_err() {
                    self.state
                        .errors
                        .insert(f.key.clone(), "Must be a number".into());
                }
            }
        }

        self.state.errors.is_empty() && self.state.global_errors.is_empty()
    }

    fn submit(&mut self) -> Option<Action> {
        if !self.validate() {
            return Some(Action::Update);
        }
        let mut map = JsonMap::new();

        for f in &self.schema.fields {
            match &f.kind {
                FormFieldKind::Text
                | FormFieldKind::Secret
                | FormFieldKind::Path
                | FormFieldKind::Select { .. } => {
                    let v = self.state.get_value(&f.key).unwrap_or("").to_string();
                    map.insert(f.key.clone(), JsonValue::String(v));
                }
                FormFieldKind::Number => {
                    let v = self.state.get_value(&f.key).unwrap_or("").to_string();
                    if let Ok(n) = v.parse::<i64>() {
                        map.insert(f.key.clone(), json!(n));
                    } else {
                        map.insert(f.key.clone(), JsonValue::String(v));
                    }
                }
                FormFieldKind::Bool => {
                    let v = self.state.get_value(&f.key).unwrap_or("false") == "true";
                    map.insert(f.key.clone(), json!(v));
                }
                FormFieldKind::ListString => {
                    let items = self
                        .state
                        .lists
                        .get(&f.key)
                        .cloned()
                        .unwrap_or_else(Vec::new);
                    map.insert(f.key.clone(), json!(items));
                }
            }
        }

        Some(Action::PopupResult(PopupResult::FormSubmitted(
            JsonValue::Object(map),
        )))
    }
}

impl Component for FormPopup {
    fn height_constraint(&self) -> Constraint {
        Constraint::Min(self.schema.min_height)
    }

    fn keymap_context(&self) -> &'static str {
        "popup"
    }

    fn popup_min_size(&self) -> Option<(u16, u16)> {
        Some((self.schema.min_width, self.schema.min_height))
    }

    fn handle_key_events(
        &mut self,
        key: KeyEvent,
    ) -> Result<Option<crate::tui::EventResponse<Action>>> {
        // Editing mode: route to input
        if self.editing {
            match key.code {
                KeyCode::Enter => {
                    self.commit_editing();
                    return Ok(Some(crate::tui::EventResponse::Stop(Action::Update)));
                }
                KeyCode::Esc => {
                    self.cancel_editing();
                    return Ok(Some(crate::tui::EventResponse::Stop(Action::Update)));
                }
                _ => {
                    self.input.handle_event(&crossterm::event::Event::Key(key));
                    return Ok(Some(crate::tui::EventResponse::Stop(Action::Update)));
                }
            }
        }

        // Not editing: navigate/toggle/submit
        match key.code {
            KeyCode::Up => {
                self.focus_prev();
                return Ok(Some(crate::tui::EventResponse::Stop(Action::Update)));
            }
            KeyCode::Down => {
                self.focus_next();
                return Ok(Some(crate::tui::EventResponse::Stop(Action::Update)));
            }
            KeyCode::Tab => {
                self.focus_next();
                return Ok(Some(crate::tui::EventResponse::Stop(Action::Update)));
            }
            KeyCode::BackTab => {
                self.focus_prev();
                return Ok(Some(crate::tui::EventResponse::Stop(Action::Update)));
            }
            KeyCode::PageDown => {
                // Jump down by a page (visible window size - 1)
                let reserve = if self.last_inner_height > 8 { 4 } else { 2 };
                let visible = self.last_inner_height.saturating_sub(reserve).max(3) as usize;
                let jump = visible.saturating_sub(1).max(1);
                for _ in 0..jump {
                    self.focus_next();
                }
                return Ok(Some(crate::tui::EventResponse::Stop(Action::Update)));
            }
            KeyCode::PageUp => {
                // Jump up by a page (visible window size - 1)
                let reserve = if self.last_inner_height > 8 { 4 } else { 2 };
                let visible = self.last_inner_height.saturating_sub(reserve).max(3) as usize;
                let jump = visible.saturating_sub(1).max(1);
                for _ in 0..jump {
                    self.focus_prev();
                }
                return Ok(Some(crate::tui::EventResponse::Stop(Action::Update)));
            }
            KeyCode::Home => {
                // Go to first field
                if self.field_count() > 0 {
                    self.focused = 0;
                }
                return Ok(Some(crate::tui::EventResponse::Stop(Action::Update)));
            }
            KeyCode::End => {
                // Go to last field
                if self.field_count() > 0 {
                    self.focused = self.field_count() - 1;
                }
                return Ok(Some(crate::tui::EventResponse::Stop(Action::Update)));
            }
            KeyCode::Left | KeyCode::Right | KeyCode::Char(' ') => {
                if let Some(field) = self.current_field() {
                    match &field.kind {
                        FormFieldKind::Bool => {
                            let k = field.key.clone();
                            self.toggle_bool(&k);
                            return Ok(Some(crate::tui::EventResponse::Stop(Action::Update)));
                        }
                        FormFieldKind::Select { options } => {
                            let k = field.key.clone();
                            let opts = options.clone();
                            let dir = if matches!(key.code, KeyCode::Left) {
                                -1
                            } else {
                                1
                            };
                            self.cycle_select(&k, &opts, dir);
                            return Ok(Some(crate::tui::EventResponse::Stop(Action::Update)));
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::Insert => {
                if self.current_field().map(|f| f.is_list()).unwrap_or(false) {
                    self.start_editing(); // add new list item
                    return Ok(Some(crate::tui::EventResponse::Stop(Action::Update)));
                }
            }
            KeyCode::Enter => {
                // If field is textual and not editing: start editing the current field
                if self
                    .current_field()
                    .map(|f| f.is_textual() || f.is_list())
                    .unwrap_or(false)
                {
                    self.start_editing();
                    return Ok(Some(crate::tui::EventResponse::Stop(Action::Update)));
                }
                // Otherwise, treat Enter as Submit
                return Ok(Some(crate::tui::EventResponse::Stop(Action::Submit)));
            }
            KeyCode::Esc => {
                return Ok(Some(crate::tui::EventResponse::Stop(Action::PopupResult(
                    PopupResult::Cancelled,
                ))));
            }
            _ => {}
        }

        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Submit => Ok(self.submit()),
            Action::PopupResult(PopupResult::FormSubmitted(_)) => Ok(Some(Action::ClosePopup)),
            _ => Ok(None),
        }
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: ratatui::layout::Rect) -> Result<()> {
        if area.width < 5 || area.height < 5 {
            return Ok(());
        }

        // Compute dialog rect and draw frame
        let w = self.schema.min_width.min(area.width);
        let h = self.schema.min_height.min(area.height);
        let dialog = centered_rect_fixed(area, w, h);
        let _ = draw_popup_frame(f, dialog, &self.schema.title);

        // Inner area
        let inner = ratatui::layout::Rect {
            x: dialog.x.saturating_add(1),
            y: dialog.y.saturating_add(1),
            width: dialog.width.saturating_sub(2),
            height: dialog.height.saturating_sub(2),
        };
        f.render_widget(Clear, inner);
        // store inner height for PageUp/PageDown calculation
        self.last_inner_height = inner.height;

        // Assemble lines to render
        let mut lines: Vec<Line> = Vec::new();

        // Description (optional)
        if let Some(desc) = &self.schema.description {
            for l in desc.lines() {
                lines.push(Line::from(Span::styled(
                    l.to_string(),
                    Style::default().fg(Color::Gray),
                )));
            }
            lines.push(Line::raw("")); // spacer
        }

        // Global errors
        if !self.state.global_errors.is_empty() {
            lines.push(
                Line::from("Errors:").style(
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                ),
            );
            for e in &self.state.global_errors {
                lines.push(Line::from(Span::styled(
                    format!("• {}", e),
                    Style::default().fg(Color::Red),
                )));
            }
            lines.push(Line::raw(""));
        }

        // Ensure focused field is within view
        self.ensure_visible(inner.height);
        let (start, end) = self.visible_bounds(inner.height);

        // Render a window of fields
        for (idx, field) in self.schema.fields[start..end].iter().enumerate() {
            let absolute_idx = start + idx;
            let focused = absolute_idx == self.focused;
            let mut label_spans = vec![Span::styled(
                format!("{}:", field.label),
                Style::default().fg(Color::White).add_modifier(if focused {
                    ratatui::style::Modifier::BOLD
                } else {
                    ratatui::style::Modifier::empty()
                }),
            )];

            let value = if focused && self.editing && (field.is_textual() || field.is_list()) {
                self.input.value().to_string()
            } else {
                self.field_display_value(field)
            };

            label_spans.push(Span::raw(" "));
            let value_style = if focused {
                Style::default().fg(Color::Black).bg(Color::White)
            } else {
                Style::default().fg(Color::Cyan)
            };

            // Obfuscate secret in value display already handled by field_display_value
            label_spans.push(Span::styled(value, value_style));

            lines.push(Line::from(label_spans));

            // Help text (dimmed)
            if let Some(h) = &field.help {
                lines.push(Line::from(Span::styled(
                    h,
                    Style::default().fg(Color::DarkGray),
                )));
            }

            // Field error line (if any)
            if let Some(err) = self.state.errors.get(&field.key) {
                lines.push(Line::from(Span::styled(
                    err,
                    Style::default().fg(Color::Red),
                )));
            }

            // Spacer between fields
            lines.push(Line::raw(""));
        }

        // Footer hints
        lines.push(Line::raw(""));
        let footer = Line::from(vec![
            Span::styled("Up/Down", Style::default().fg(Color::White)),
            Span::raw(": Navigate   "),
            Span::styled("Enter", Style::default().fg(Color::White)),
            Span::raw(": "),
            // Context-aware hint for Enter
            if self.editing {
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

        // Render main content first
        f.render_widget(para, inner);

        // Simple scrollbar-like indicator on the right edge if there are more fields than visible
        let total = self.field_count();
        if total > 0 {
            let reserve = if inner.height > 8 { 4 } else { 2 };
            let visible = inner.height.saturating_sub(reserve).max(3) as usize;
            if total > visible && inner.width >= 1 {
                // One-column track on the far right
                let track_rect = ratatui::layout::Rect {
                    x: inner.x + inner.width.saturating_sub(1),
                    y: inner.y,
                    width: 1,
                    height: inner.height,
                };

                // Compute thumb position
                let max_thumb_y = track_rect.height.saturating_sub(1) as usize;
                let denom = total.saturating_sub(visible).max(1);
                let ratio = (self.scroll as f32) / (denom as f32);
                let thumb_y = (ratio * (max_thumb_y as f32)).round() as usize;

                // Build vertical track
                let mut track_lines: Vec<Line> = Vec::new();
                for i in 0..track_rect.height {
                    if i as usize == thumb_y {
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
                // Render track on top
                f.render_widget(track_para, track_rect);
            }
        }

        Ok(())
    }
}
