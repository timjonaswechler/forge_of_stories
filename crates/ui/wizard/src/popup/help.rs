//! Help popup that displays keybinding information
//!
//! Shows a searchable table of available keybindings with context information.

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Row, Table, TableState},
};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tui_input::Input;

use crate::{
    action::{Action, UiAction},
    app::settings::SettingsStore,
    popup::{Popup, PopupConfig, PopupPosition, PopupSize},
};

/// Help popup displaying keybinding information
#[derive(Clone)]
pub struct HelpPopup {
    action_tx: Option<UnboundedSender<Action>>,
    settings: Option<Arc<SettingsStore>>,
    keymap_context: String,

    // Search functionality
    help_search: Option<String>,
    help_input: Input,
    help_prompt_active: bool,

    // Table state
    help_table_state: TableState,
    help_table_len: usize,
}

impl Default for HelpPopup {
    fn default() -> Self {
        Self::new()
    }
}

impl HelpPopup {
    pub fn new() -> Self {
        Self {
            action_tx: None,
            settings: None,
            keymap_context: "global".to_string(),
            help_search: None,
            help_input: Input::default(),
            help_prompt_active: false,
            help_table_state: TableState::default(),
            help_table_len: 0,
        }
    }

    /// Set the settings store for this help popup
    pub fn set_settings(&mut self, settings: Arc<SettingsStore>) {
        self.settings = Some(settings);
    }

    /// Set the current keymap context
    pub fn set_keymap_context(&mut self, context: String) {
        self.keymap_context = context;
    }

    /// Toggle search mode
    fn toggle_search(&mut self) -> Result<Option<Action>> {
        if self.help_prompt_active {
            // Exit search mode and apply filter
            self.help_prompt_active = false;
            let input_value = self.help_input.value().trim();
            if input_value.is_empty() {
                self.help_search = None;
            } else {
                self.help_search = Some(input_value.to_string());
            }
            // Reset selection when search changes
            self.help_table_state.select(Some(0));
        } else {
            // Enter search mode
            self.help_prompt_active = true;
            if let Some(current) = &self.help_search {
                self.help_input = Input::new(current.clone());
            } else {
                self.help_input = Input::default();
            }
        }
        Ok(None)
    }

    /// Clear the current search
    fn clear_search(&mut self) -> Result<Option<Action>> {
        self.help_search = None;
        self.help_input = Input::default();
        self.help_prompt_active = false;
        self.help_table_state.select(Some(0));
        Ok(None)
    }

    /// Navigate the help table
    fn navigate_table(&mut self, direction: TableNavigation) -> Result<Option<Action>> {
        if self.help_table_len == 0 {
            return Ok(None);
        }

        let new_index = match direction {
            TableNavigation::Up => {
                match self.help_table_state.selected() {
                    Some(i) if i > 0 => Some(i - 1),
                    Some(_) => Some(self.help_table_len - 1), // Wrap to bottom
                    None => Some(0),
                }
            }
            TableNavigation::Down => {
                match self.help_table_state.selected() {
                    Some(i) if i + 1 < self.help_table_len => Some(i + 1),
                    Some(_) => Some(0), // Wrap to top
                    None => Some(0),
                }
            }
            TableNavigation::PageUp => match self.help_table_state.selected() {
                Some(i) => Some(i.saturating_sub(10)),
                None => Some(0),
            },
            TableNavigation::PageDown => match self.help_table_state.selected() {
                Some(i) => Some((i + 10).min(self.help_table_len - 1)),
                None => Some(0),
            },
            TableNavigation::Home => Some(0),
            TableNavigation::End => Some(self.help_table_len.saturating_sub(1)),
        };

        if let Some(index) = new_index {
            self.help_table_state.select(Some(index));
        }

        Ok(None)
    }

    /// Build the keymap table data from settings
    fn build_keymap_table_data(
        &self,
        active_contexts: &[String],
    ) -> Vec<(String, String, String, String)> {
        use settings::DeviceFilter;

        let mut table_data = Vec::new();
        let filter = self.help_search.as_ref().map(|s| s.to_ascii_lowercase());

        if let Some(settings) = &self.settings {
            let mut seen_actions = std::collections::HashSet::new();

            // Iterate through ALL active contexts
            for context_name in active_contexts {
                let ctx_map = settings.export_keymap_for(DeviceFilter::Keyboard, context_name);

                // Add context-specific bindings
                for (action_name, chords) in ctx_map.iter() {
                    let keys_str = chords.join(", ");

                    // Generate a simple detail description
                    let details = match action_name.as_str() {
                        "Help" => "Show/hide this help dialog".to_string(),
                        "Quit" => "Exit the application".to_string(),
                        "FocusNext" | "NextField" => "Move focus to next element".to_string(),
                        "FocusPrev" | "PrevField" => "Move focus to previous element".to_string(),
                        "NavigateUp" => "Navigate up".to_string(),
                        "NavigateDown" => "Navigate down".to_string(),
                        "NavigateLeft" => "Navigate left".to_string(),
                        "NavigateRight" => "Navigate right".to_string(),
                        "ActivateSelected" => "Activate selected item".to_string(),
                        "ToggleEditMode" | "ModeCycle" => "Toggle edit mode".to_string(),
                        "NextPage" => "Go to next page".to_string(),
                        "PrevPage" | "PreviousPage" => "Go to previous page".to_string(),
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
                        // Use combination of action + context to avoid duplicates
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
        } else {
            // Fallback content when no settings available
            let fallback_data = vec![
                (
                    "Help".to_string(),
                    "ctrl+h".to_string(),
                    "global".to_string(),
                    "Show this help dialog".to_string(),
                ),
                (
                    "Quit".to_string(),
                    "ctrl+c".to_string(),
                    "global".to_string(),
                    "Exit the application".to_string(),
                ),
            ];

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
}

/// Navigation directions for the help table
enum TableNavigation {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
}

impl Popup for HelpPopup {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn init(&mut self) -> Result<()> {
        // Initialize table selection
        self.help_table_state.select(Some(0));
        Ok(())
    }

    fn keymap_context(&self) -> &'static str {
        "help"
    }

    fn id(&self) -> &'static str {
        "help"
    }

    fn config(&self) -> PopupConfig {
        PopupConfig {
            size: PopupSize::Percentage {
                width: 85,
                height: 85,
            },
            position: PopupPosition::Center,
            modal: true,
            closable: true,
            resizable: false,
        }
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        // Handle different key events based on current mode
        if self.help_prompt_active {
            // Search mode - handle input
            match key.code {
                KeyCode::Char(c) => {
                    self.help_input
                        .handle(tui_input::InputRequest::InsertChar(c));
                }
                KeyCode::Backspace => {
                    self.help_input
                        .handle(tui_input::InputRequest::DeletePrevChar);
                }
                KeyCode::Delete => {
                    self.help_input
                        .handle(tui_input::InputRequest::DeleteNextChar);
                }
                KeyCode::Left => {
                    self.help_input
                        .handle(tui_input::InputRequest::GoToPrevChar);
                }
                KeyCode::Right => {
                    self.help_input
                        .handle(tui_input::InputRequest::GoToNextChar);
                }
                KeyCode::Home => {
                    self.help_input.handle(tui_input::InputRequest::GoToStart);
                }
                KeyCode::End => {
                    self.help_input.handle(tui_input::InputRequest::GoToEnd);
                }
                KeyCode::Enter => {
                    return self.toggle_search();
                }
                KeyCode::Esc => {
                    self.help_prompt_active = false;
                    return Ok(None);
                }
                _ => {}
            }
        } else {
            // Normal mode - handle navigation and commands
            match (key.code, key.modifiers) {
                (KeyCode::Esc, _) => {
                    return Ok(Some(Action::Ui(UiAction::ClosePopup {
                        id: "help".to_string(),
                    })));
                }
                (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                    return self.toggle_search();
                }
                (KeyCode::Char('/'), _) => {
                    return self.toggle_search();
                }
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    return self.clear_search();
                }
                (KeyCode::Up, _) => {
                    return self.navigate_table(TableNavigation::Up);
                }
                (KeyCode::Down, _) => {
                    return self.navigate_table(TableNavigation::Down);
                }
                (KeyCode::PageUp, _) => {
                    return self.navigate_table(TableNavigation::PageUp);
                }
                (KeyCode::PageDown, _) => {
                    return self.navigate_table(TableNavigation::PageDown);
                }
                (KeyCode::Home, _) => {
                    return self.navigate_table(TableNavigation::Home);
                }
                (KeyCode::End, _) => {
                    return self.navigate_table(TableNavigation::End);
                }
                _ => {}
            }
        }

        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Ui(UiAction::ComponentCommand { command, data: _ }) => match command.as_str() {
                "toggle_search" => return self.toggle_search(),
                "clear_search" => return self.clear_search(),
                _ => {}
            },
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame, area: Rect) -> Result<()> {
        // Clear the area first for modal effect
        f.render_widget(Clear, area);

        let collapsed_border_set = symbols::border::Set {
            top_left: symbols::line::NORMAL.vertical_right,
            top_right: symbols::line::NORMAL.vertical_left,
            bottom_left: symbols::line::ROUNDED.bottom_left,
            bottom_right: symbols::line::ROUNDED.bottom_right,
            ..symbols::border::PLAIN
        };

        // Layout: search area at top, table at bottom
        let [search_area, key_table_area] =
            Layout::vertical([Constraint::Length(4), Constraint::Fill(1)]).areas(area);

        // Render search input area
        let search_block = Block::new()
            .border_type(BorderType::Rounded)
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title("─ Search ");
        f.render_widget(search_block, search_area);

        // Render search input or status
        if self.help_prompt_active {
            let input_area = Rect::new(
                search_area.x + 2,
                search_area.y + 2,
                search_area.width.saturating_sub(4),
                1,
            );
            let prompt_value = self.help_input.value();
            let prompt_text = format!("{}", prompt_value);
            let prompt = Paragraph::new(prompt_text);
            f.render_widget(prompt, input_area);

            // Place cursor
            let cursor = self.help_input.visual_cursor() as u16;
            let cx = input_area.x + cursor;
            let cy = input_area.y;
            f.set_cursor_position((cx, cy));
        } else {
            // Show current search query or placeholder
            let input_area = Rect::new(
                search_area.x + 2,
                search_area.y + 2,
                search_area.width.saturating_sub(4),
                1,
            );
            let search_text = match &self.help_search {
                Some(query) => format!("Filter: {} [Press Ctrl+F to search]", query),
                None => "Press Ctrl+F to search keybindings".to_string(),
            };
            let search_style = Style::default().fg(Color::Gray);
            let search_para = Paragraph::new(search_text).style(search_style);
            f.render_widget(search_para, input_area);
        }

        // Build table data - use current context for now
        let contexts = vec![self.keymap_context.clone()];
        let table_data = self.build_keymap_table_data(&contexts);

        // Update stored table length for navigation
        self.help_table_len = table_data.len();

        // Ensure selection is within bounds
        if let Some(selected) = self.help_table_state.selected() {
            if selected >= table_data.len() && !table_data.is_empty() {
                self.help_table_state.select(Some(0));
            }
        } else if !table_data.is_empty() {
            self.help_table_state.select(Some(0));
        }

        let table_block = Block::new()
            .borders(Borders::ALL)
            .border_set(collapsed_border_set)
            .title("─ Key Bindings ");

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
                    Style::default().bg(Color::Rgb(40, 40, 60))
                } else {
                    Style::default().bg(Color::Rgb(30, 30, 40))
                };

                // Add visual marker for selected row
                let marker = if self.help_table_state.selected() == Some(i) {
                    "►"
                } else {
                    " "
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
                Constraint::Length(max_action_len as u16),
                Constraint::Length(max_keys_len as u16),
                Constraint::Length(max_context_len as u16),
                Constraint::Fill(1),
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
}
