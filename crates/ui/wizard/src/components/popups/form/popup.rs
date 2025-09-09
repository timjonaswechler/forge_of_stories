use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    action::{Action, UiOutcome},
    components::Component,
};

use super::{FormField, FormFieldKind, FormSchema, FormState};

/// Interactive multi-field form popup (logic / state only – rendering lives in `render.rs`).
///
/// Extracted from the former monolithic `form/mod.rs` (Phase 6.1).
///
/// Responsibilities:
/// - Navigation & focus management
/// - Editing lifecycle (enter edit, commit, cancel)
/// - Per-field + list item mutation
/// - Validation dispatch & JSON submission building
/// - Event handling (key mapping → `Action`s)
///
/// Rendering concerns were moved to `render::render_form_popup`.
pub struct FormPopup {
    schema: FormSchema,
    state: FormState,

    // UI / navigation state
    focused: usize,
    scroll: usize,
    editing: bool,
    input: Input,
    last_inner_height: u16, // remembered from last render for page-size heuristics
}

impl FormPopup {
    /// Create a new popup with a given schema and empty state.
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

    /// Replace the internal state (e.g., restore a draft form).
    pub fn with_state(mut self, state: FormState) -> Self {
        self.state = state;
        self
    }

    // --- Accessors used by the renderer --------------------------------------------------------

    pub(super) fn schema(&self) -> &FormSchema {
        &self.schema
    }

    pub(super) fn state(&self) -> &FormState {
        &self.state
    }

    pub(super) fn field_count(&self) -> usize {
        self.schema.fields.len()
    }

    pub(super) fn focused_index(&self) -> usize {
        self.focused
    }

    pub(super) fn scroll(&self) -> usize {
        self.scroll
    }

    pub(super) fn is_editing(&self) -> bool {
        self.editing
    }

    pub(super) fn input_value(&self) -> &str {
        self.input.value()
    }

    pub(super) fn set_last_inner_height(&mut self, h: u16) {
        self.last_inner_height = h;
    }

    /// Compute visible field bounds (start, end) given the inner height.
    pub(super) fn visible_bounds(&self, inner_height: u16) -> (usize, usize) {
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

    /// Ensure the focused field is within the current visible window.
    pub(super) fn ensure_visible(&mut self, inner_height: u16) {
        let reserve = if inner_height > 8 { 4 } else { 2 };
        let max_visible = inner_height.saturating_sub(reserve).max(3) as usize;
        if self.focused < self.scroll {
            self.scroll = self.focused;
        } else if self.focused >= self.scroll + max_visible {
            self.scroll = self.focused + 1 - max_visible;
        }
    }

    // --- Internal navigation / editing helpers -------------------------------------------------

    fn current_field(&self) -> Option<&FormField> {
        self.schema.fields.get(self.focused)
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

    /// Produce a display string for a field's current value (mirrors original logic).
    pub(super) fn field_display_value(&self, field: &FormField) -> String {
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
                    "true".into()
                } else {
                    "false".into()
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

    // --- Validation & submission ---------------------------------------------------------------

    fn validate(&mut self) -> bool {
        self.state.errors.clear();
        self.state.global_errors.clear();

        for f in &self.schema.fields {
            let v_owned = match f.kind {
                FormFieldKind::ListString => {
                    // Validate each list entry if a validator is present
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

            // Built-in sanity check for numeric fields
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

        Some(Action::UiOutcome(UiOutcome::SubmitJson(JsonValue::Object(
            map,
        ))))
    }
}

impl Component for FormPopup {
    fn height_constraint(&self) -> ratatui::layout::Constraint {
        ratatui::layout::Constraint::Min(self.schema.min_height)
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
        // Editing mode
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

        // Navigation / interaction
        match key.code {
            KeyCode::Up => {
                self.focus_prev();
                Ok(Some(crate::tui::EventResponse::Stop(Action::Update)))
            }
            KeyCode::Down => {
                self.focus_next();
                Ok(Some(crate::tui::EventResponse::Stop(Action::Update)))
            }
            KeyCode::Tab => {
                self.focus_next();
                Ok(Some(crate::tui::EventResponse::Stop(Action::Update)))
            }
            KeyCode::BackTab => {
                self.focus_prev();
                Ok(Some(crate::tui::EventResponse::Stop(Action::Update)))
            }
            KeyCode::PageDown => {
                let reserve = if self.last_inner_height > 8 { 4 } else { 2 };
                let visible = self.last_inner_height.saturating_sub(reserve).max(3) as usize;
                let jump = visible.saturating_sub(1).max(1);
                for _ in 0..jump {
                    self.focus_next();
                }
                Ok(Some(crate::tui::EventResponse::Stop(Action::Update)))
            }
            KeyCode::PageUp => {
                let reserve = if self.last_inner_height > 8 { 4 } else { 2 };
                let visible = self.last_inner_height.saturating_sub(reserve).max(3) as usize;
                let jump = visible.saturating_sub(1).max(1);
                for _ in 0..jump {
                    self.focus_prev();
                }
                Ok(Some(crate::tui::EventResponse::Stop(Action::Update)))
            }
            KeyCode::Home => {
                if self.field_count() > 0 {
                    self.focused = 0;
                }
                Ok(Some(crate::tui::EventResponse::Stop(Action::Update)))
            }
            KeyCode::End => {
                if self.field_count() > 0 {
                    self.focused = self.field_count() - 1;
                }
                Ok(Some(crate::tui::EventResponse::Stop(Action::Update)))
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
                Ok(None)
            }
            KeyCode::Insert => {
                if self.current_field().map(|f| f.is_list()).unwrap_or(false) {
                    self.start_editing();
                    Ok(Some(crate::tui::EventResponse::Stop(Action::Update)))
                } else {
                    Ok(None)
                }
            }
            KeyCode::Enter => {
                if self
                    .current_field()
                    .map(|f| f.is_textual() || f.is_list())
                    .unwrap_or(false)
                {
                    self.start_editing();
                    Ok(Some(crate::tui::EventResponse::Stop(Action::Update)))
                } else {
                    Ok(Some(crate::tui::EventResponse::Stop(Action::Submit)))
                }
            }
            KeyCode::Esc => Ok(Some(crate::tui::EventResponse::Stop(Action::UiOutcome(
                UiOutcome::Cancelled,
            )))),
            _ => Ok(None),
        }
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Submit => Ok(self.submit()),
            _ => Ok(None),
        }
    }

    fn draw(&mut self, f: &mut crate::tui::Frame<'_>, area: ratatui::layout::Rect) -> Result<()> {
        // Delegate to the extracted rendering function
        super::render::render_form_popup(self, f, area)?;
        Ok(())
    }
}
