use std::sync::Arc;

use color_eyre::Result;
use crossterm::event::Event as CEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Row, Table, TableState},
};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    action::{Action, AppAction, LayerKind, UiAction},
    app::settings::SettingsStore,
};

//
// Layer registry
//

/// Trait for drawing named layers on top of the base UI.
/// The `id` conveys the concrete popup/overlay to render (e.g., "help", "confirm").
///
/// Implementations should avoid side-effects; they only draw based on the current state.
/// Any dynamic data needed by a layer (e.g., keymaps for help) should be stored in the registry
/// and updated by the app before calling `render_layer`.
pub trait LayerRegistry {
    fn render_layer(
        &mut self,
        f: &mut Frame<'_>,
        area: Rect,
        kind: LayerKind,
        id: Option<&str>,
        active_contexts: Option<&[String]>,
    ) -> Result<()>;
}

/// Basic in-memory registry for default layers used by Wizard (help, confirm, error_details).
/// - "help": shows a keymap table for the current `keymap_context`.
/// - "confirm": draws a placeholder confirm dialog with a title/body.
/// - "error_details": draws a placeholder error details view with multiline text.
pub struct BasicLayerRegistry {
    settings: Option<Arc<SettingsStore>>,
    keymap_context: String,

    // Help state
    show_global: bool,
    help_scroll: u16,
    help_search: Option<String>,
    /// Whether Help content wraps long lines.
    wrap_on: bool,
    // Help prompt (search input) state
    help_prompt_active: bool,
    help_prompt_buffer: String,
    /// Interactive input state and history for help search.
    help_input: Input,
    /// Table state for navigation
    help_table_state: TableState,
    /// Last table length for navigation bounds checking
    help_table_len: usize,

    // Confirm/Error state (stubs for now; set via setters before render)
    confirm_title: String,
    confirm_body: String,

    error_title: String,
    error_text: String,
}

impl Default for BasicLayerRegistry {
    fn default() -> Self {
        Self {
            settings: None,
            keymap_context: "global".to_string(),
            show_global: true,
            help_scroll: 0,
            help_search: None,
            wrap_on: true,
            help_prompt_active: false,
            help_prompt_buffer: String::new(),
            help_input: Input::default(),
            help_table_state: {
                let mut state = TableState::default();
                state.select(Some(0)); // Start with first row selected
                state
            },
            help_table_len: 0,

            confirm_title: "Confirm".to_string(),
            confirm_body: "Are you sure you want to continue?".to_string(),
            error_title: "Error".to_string(),
            error_text: "No details.".to_string(),
        }
    }
}

impl BasicLayerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_settings_handler(&mut self, settings: Arc<SettingsStore>) {
        self.settings = Some(settings);
    }

    /// Update the active keymap context used by the Help layer.
    pub fn set_keymap_context(&mut self, ctx: impl Into<String>) {
        self.keymap_context = ctx.into();
        // Reset scroll when context changes for better UX
        self.help_scroll = 0;
    }

    /// Toggle whether to include global key bindings in Help.
    pub fn toggle_show_global(&mut self) {
        self.show_global = !self.show_global;
    }

    /// Returns whether Help includes global key bindings.
    pub fn is_show_global(&self) -> bool {
        self.show_global
    }

    /// Toggle line wrapping for Help content.
    pub fn toggle_wrap(&mut self) {
        self.wrap_on = !self.wrap_on;
    }

    /// Returns whether Help wraps long lines.
    pub fn is_wrap_on(&self) -> bool {
        self.wrap_on
    }

    /// Set an absolute scroll line for the Help paragraph.
    pub fn set_help_scroll(&mut self, offset: u16) {
        self.help_scroll = offset;
    }

    /// Scroll the Help paragraph by delta lines (positive = down, negative = up).
    pub fn scroll_help_lines(&mut self, delta: i16) {
        let current = self.help_scroll as i32;
        let next = (current + delta as i32).max(0) as u16;
        self.help_scroll = next;
    }

    /// Set or clear the Help search filter (case-insensitive).
    pub fn set_help_search<S: Into<String>>(&mut self, query: Option<S>) {
        self.help_search = query.map(|s| s.into());
        self.help_scroll = 0; // reset scroll when search changes
    }

    /// Clear the Help search filter.
    pub fn clear_help_search(&mut self) {
        self.help_search = None;
        self.help_scroll = 0;
    }

    /// Navigate up in the help table
    pub fn help_navigate_up(&mut self) {
        if let Some(selected) = self.help_table_state.selected() {
            if selected > 0 {
                self.help_table_state.select(Some(selected - 1));
            }
        }
    }

    /// Navigate down in the help table
    pub fn help_navigate_down(&mut self, table_len: usize) {
        if let Some(selected) = self.help_table_state.selected() {
            if selected < table_len.saturating_sub(1) {
                self.help_table_state.select(Some(selected + 1));
            }
        } else if table_len > 0 {
            self.help_table_state.select(Some(0));
        }
    }

    /// Get currently selected help item index
    pub fn help_selected_index(&self) -> Option<usize> {
        self.help_table_state.selected()
    }

    /// Explicitly control the visibility of the help search prompt.
    pub fn set_help_prompt_active(&mut self, active: bool) {
        self.help_prompt_active = active;
        if !active {
            self.help_prompt_buffer.clear();
        }
    }

    /// Update the current input buffer for the help search prompt.
    pub fn set_help_prompt_buffer<S: Into<String>>(&mut self, buf: S) {
        self.help_prompt_active = true;
        self.help_prompt_buffer = buf.into();
    }

    /// Convenience: update internal state based on app-wide actions.
    /// Not required, but handy if you want to propagate context via broadcast.
    pub fn update_from_action(&mut self, action: &Action) {
        match action {
            Action::App(AppAction::SetKeymapContext { name }) => {
                self.set_keymap_context(name.clone());
            }
            Action::Ui(UiAction::BeginHelpSearch) => {
                self.help_prompt_active = true;
                self.help_prompt_buffer.clear();
                self.help_input = Input::default();
            }
            Action::Ui(UiAction::HelpPromptKey(key)) => {
                // Forward raw key event to the input widget
                self.help_prompt_active = true;
                let evt = CEvent::Key(*key);
                self.help_input.handle_event(&evt);
                self.help_prompt_buffer = self.help_input.value().to_string();
            }
            #[cfg(any())]
            Action::Ui(UiAction::ReportHelpSearchBuffer(_buf)) => {
                // No-op: we rely on HelpPromptKey to mutate the input state.
            }
            Action::Ui(UiAction::HelpSearch(q)) => {
                // Apply search filter, close prompt, and add to history if non-empty
                self.set_help_search(Some(q.clone()));
                self.help_prompt_active = false;
                self.help_prompt_buffer.clear();
                self.help_input = Input::default();
            }
            Action::Ui(UiAction::HelpSearchClear) => {
                self.clear_help_search();
                self.help_prompt_active = false;
                self.help_prompt_buffer.clear();
                self.help_input = Input::default();
            }
            Action::Ui(UiAction::ReportHelpVisible(false)) => {
                // If help is closed, also close the prompt
                self.help_prompt_active = false;
                self.help_prompt_buffer.clear();
                self.help_input = Input::default();
            }
            Action::Ui(UiAction::NavigateUp) => {
                // Navigate up in help table when help is visible
                self.help_navigate_up();
            }
            Action::Ui(UiAction::NavigateDown) => {
                // Navigate down in help table when help is visible
                self.help_navigate_down(self.help_table_len);
            }
            _ => {}
        }
    }

    pub fn set_confirm_content(&mut self, title: impl Into<String>, body: impl Into<String>) {
        self.confirm_title = title.into();
        self.confirm_body = body.into();
    }

    pub fn set_error_content(&mut self, title: impl Into<String>, text: impl Into<String>) {
        self.error_title = title.into();
        self.error_text = text.into();
    }

    /// Build table data from keymaps with Action|Keys|Context|Details columns
    fn build_keymap_table_data(
        &self,
        active_contexts: &[String],
    ) -> Vec<(String, String, String, String)> {
        use settings::DeviceFilter;

        let mut table_data = Vec::new();
        let filter = self.help_search.as_ref().map(|s| s.to_ascii_lowercase());

        if let Some(settings) = &self.settings {
            let mut seen_actions = std::collections::HashSet::new();

            // Iterate through ALL active contexts instead of just keymap_context

            for context_name in active_contexts {
                let ctx_map = settings.export_keymap_for(DeviceFilter::Keyboard, context_name);

                // Add context-specific bindings
                for (action_name, chords) in ctx_map.iter() {
                    let keys_str = chords.join(", ");
                    // Use the actual context name instead of self.keymap_context

                    // Generate a simple detail description (can be enhanced)
                    let details = match action_name.as_str() {
                        "Help" => "Show/hide this help dialog".to_string(),
                        "Quit" => "Exit the application".to_string(),
                        _ => format!("Execute {}", action_name.replace("_", " ").to_lowercase()),
                    };

                    // Apply search filter
                    let include = match &filter {
                        Some(query) => {
                            let action_lower = action_name.to_ascii_lowercase();
                            let keys_lower = keys_str.to_ascii_lowercase();
                            let context_lower = context_name.to_ascii_lowercase();
                            let details_lower = details.to_ascii_lowercase();

                            action_lower.contains(query)
                                || keys_lower.contains(query)
                                || context_lower.contains(query)
                                || details_lower.contains(query)
                        }
                        None => true,
                    };

                    if include {
                        // Use combination of action + context to avoid duplicates but allow same action from different contexts
                        let key_id = (action_name.clone(), context_name.clone());
                        if seen_actions.insert(key_id) {
                            table_data.push((
                                action_name.clone(),
                                keys_str,
                                context_name.clone(),
                                details,
                            ));
                        }
                    }
                }
            }

            // Note: All bindings including global are now included via active_contexts loop above
        } else {
            // Fallback content when no settings available
            let fallback_data = vec![(
                "Help".to_string(),
                "f1".to_string(),
                "global".to_string(),
                "Show this help dialog".to_string(),
            )];

            for (action, keys, context, details) in fallback_data {
                let include = match &filter {
                    Some(query) => {
                        let action_lower = action.to_ascii_lowercase();
                        let keys_lower = keys.to_ascii_lowercase();
                        let context_lower = context.to_ascii_lowercase();
                        let details_lower = details.to_ascii_lowercase();

                        action_lower.contains(query)
                            || keys_lower.contains(query)
                            || context_lower.contains(query)
                            || details_lower.contains(query)
                    }
                    None => true,
                };

                if include {
                    table_data.push((action, keys, context, details));
                }
            }
        }

        // Sort by context first, then by action name for better organization
        table_data.sort_by(|a, b| match a.2.cmp(&b.2) {
            std::cmp::Ordering::Equal => a.0.cmp(&b.0),
            other => other,
        });
        table_data
    }

    fn render_help(
        &mut self,
        f: &mut Frame<'_>,
        area: Rect,
        active_contexts: Option<&[String]>,
    ) -> Result<()> {
        // Centered popup area (~80% of the screen)
        let w = area.width.saturating_sub(area.width / 5);
        let h = area.height.saturating_sub(area.height / 5);
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.height - (h + 2);
        let popup_area = Rect::new(x, y, w, h);

        // Clear the area first
        f.render_widget(Clear, popup_area);

        let collapsed_border_set = symbols::border::Set {
            top_left: symbols::line::NORMAL.vertical_right,
            top_right: symbols::line::NORMAL.vertical_left,
            bottom_left: symbols::line::ROUNDED.bottom_left,
            bottom_right: symbols::line::ROUNDED.bottom_right,
            ..symbols::border::PLAIN
        };

        // Layout: search area at top, table at bottom
        let [search_area, key_table_area] =
            Layout::vertical([Constraint::Length(4), Constraint::Fill(1)]).areas(popup_area);

        // Render search input area
        let search_block = Block::new()
            .border_type(BorderType::Rounded)
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title("─ Search ");
        f.render_widget(search_block, search_area);

        // If help prompt is active, render the input field
        if self.help_prompt_active {
            let input_area = Rect::new(
                search_area.x + 2,
                search_area.y + 2,
                search_area.width.saturating_sub(2),
                1,
            );
            let prompt_value = self.help_input.value().to_string();
            let prompt_text = format!("{}", prompt_value);
            let prompt = Paragraph::new(prompt_text);
            f.render_widget(prompt, input_area);

            // Place cursor after '/' + visual cursor within the input
            let cursor = self.help_input.visual_cursor() as u16;
            let cx = input_area.x + cursor;
            let cy = input_area.y;
            f.set_cursor_position((cx, cy));
        } else {
            // Show current search query or placeholder
            let input_area = Rect::new(
                search_area.x + 2,
                search_area.y + 2,
                search_area.width.saturating_sub(1),
                2,
            );
            let search_text = match &self.help_search {
                Some(query) => format!("Filter: {} [Press crtl + f to search]", query),
                None => "Press crtl + f to search keybindings".to_string(),
            };
            let search_style = Style::default().fg(Color::Gray);
            let search_para = Paragraph::new(search_text).style(search_style);
            f.render_widget(search_para, input_area);
        }

        // Build table data from keymaps - use active_contexts if available, fallback to current context
        let default_contexts = vec![self.keymap_context.clone()];
        let contexts_to_use = active_contexts.unwrap_or(&default_contexts);
        let table_data = self.build_keymap_table_data(contexts_to_use);

        // Update stored table length for navigation
        self.help_table_len = table_data.len();

        // Ensure selection is within bounds
        if let Some(selected) = self.help_table_state.selected() {
            if selected >= table_data.len() {
                self.help_table_state
                    .select(if table_data.is_empty() { None } else { Some(0) });
            }
        } else if !table_data.is_empty() {
            self.help_table_state.select(Some(0));
        }

        let table_block = Block::new()
            .borders(Borders::ALL)
            .border_set(collapsed_border_set)
            .title(format!("─ Key Bindings"));

        // Calculate dynamic column widths based on content
        let max_action_len = table_data
            .iter()
            .map(|(a, _, _, _)| a.len())
            .max()
            .unwrap_or(10)
            .max(6)
            + 2;
        let max_keys_len = table_data
            .iter()
            .map(|(_, k, _, _)| k.len())
            .max()
            .unwrap_or(10)
            .max(4)
            + 2;
        let max_context_len = table_data
            .iter()
            .map(|(_, _, c, _)| c.len())
            .max()
            .unwrap_or(10)
            .max(7)
            + 2;

        let header = Row::new(vec!["", "Action", "Keys", "Context", "Details"])
            .style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .height(1);

        let rows: Vec<Row> = table_data
            .into_iter()
            .enumerate()
            .map(|(i, (action, keys, context, details))| {
                let style = if i % 2 == 0 {
                    Style::default().bg(Color::Rgb(40, 40, 60)) // Dark blue alternating background
                } else {
                    Style::default().bg(Color::Rgb(30, 30, 40)) // Darker alternating background
                };

                // Add visual marker for selected row
                let marker = if self.help_table_state.selected() == Some(i) {
                    "►" // Arrow pointing right for selected item
                } else {
                    " " // Empty space for unselected items
                };

                Row::new(vec![marker.to_string(), action, keys, context, details])
                    .style(style)
                    .height(1)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(3),
                Constraint::Length(max_action_len as u16), // Dynamic Action width
                Constraint::Length(max_keys_len as u16),   // Dynamic Keys width
                Constraint::Length(max_context_len as u16), // Dynamic Context width
                Constraint::Fill(1),                       // Details takes remaining space
            ],
        )
        .header(header)
        .block(table_block)
        .row_highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        );

        f.render_stateful_widget(table, key_table_area, &mut self.help_table_state);

        Ok(())
    }

    fn render_confirm(
        &mut self,
        f: &mut Frame<'_>,
        area: Rect,
        _active_contexts: Option<&[String]>,
    ) -> Result<()> {
        let w = area.width / 2;
        let h = area.height / 3;
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;
        let popup_area = Rect::new(x, y, w, h);

        let text = Paragraph::new(self.confirm_body.clone()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(self.confirm_title.clone()),
        );
        f.render_widget(Clear, popup_area);
        f.render_widget(text, popup_area);
        Ok(())
    }

    fn render_error_details(
        &mut self,
        f: &mut Frame<'_>,
        area: Rect,
        _active_contexts: Option<&[String]>,
    ) -> Result<()> {
        let w = area.width.saturating_sub(area.width / 6);
        let h = area.height.saturating_sub(area.height / 6);
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;
        let popup_area = Rect::new(x, y, w, h);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.error_title.clone());
        let para = Paragraph::new(self.error_text.clone()).block(block);

        f.render_widget(Clear, popup_area);
        f.render_widget(para, popup_area);
        Ok(())
    }
}

impl LayerRegistry for BasicLayerRegistry {
    fn render_layer(
        &mut self,
        f: &mut Frame<'_>,
        area: Rect,
        kind: LayerKind,
        id: Option<&str>,
        active_contexts: Option<&[String]>,
    ) -> Result<()> {
        match kind {
            LayerKind::Popup | LayerKind::Overlay | LayerKind::ModalOverlay => match id {
                Some("help") => self.render_help(f, area, active_contexts)?,
                Some("confirm") => self.render_confirm(f, area, active_contexts)?,
                Some("error_details") => self.render_error_details(f, area, active_contexts)?,
                Some(other) => {
                    // Fallback frame with the given id
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Layer: {:?} — {}", kind, other));
                    f.render_widget(Clear, area);
                    f.render_widget(block, area);
                }
                None => {
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .title(format!("Layer: {:?}", kind));
                    f.render_widget(Clear, area);
                    f.render_widget(block, area);
                }
            },
            LayerKind::Notification => {
                // Notifications (toasts) are now rendered directly by the App (toast system).
                // No drawing required here.
            }
        }
        Ok(())
    }
}

//
// Toast manager
//

/// Position of the toast stack on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastPosition {
    TopRight,
    BottomRight,
    TopLeft,
    BottomLeft,
}

impl Default for ToastPosition {
    fn default() -> Self {
        ToastPosition::TopRight
    }
}
