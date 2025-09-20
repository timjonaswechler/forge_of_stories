use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Row, Table, TableState},
};
use settings::{DeviceFilter, SettingsStore};
use std::{collections::HashSet, sync::Arc};
use tui_input::Input;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpOverlayEvent {
    Consumed,
    Close,
}

pub struct HelpView {
    settings: Arc<SettingsStore>,
    contexts: Vec<String>,
    search_query: Option<String>,
    search_input: Input,
    search_active: bool,
    table_state: TableState,
    table_len: usize,
}

impl HelpView {
    pub fn new(settings: Arc<SettingsStore>) -> Self {
        Self {
            settings,
            contexts: Vec::new(),
            search_query: None,
            search_input: Input::default(),
            search_active: false,
            table_state: TableState::default(),
            table_len: 0,
        }
    }

    pub fn set_contexts(&mut self, contexts: &[String]) {
        self.contexts = contexts.iter().cloned().collect();
        if self.table_state.selected().is_none() && !self.contexts.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<HelpOverlayEvent> {
        if self.search_active {
            match key.code {
                KeyCode::Char(c) => {
                    self.search_input
                        .handle(tui_input::InputRequest::InsertChar(c));
                    return Ok(HelpOverlayEvent::Consumed);
                }
                KeyCode::Backspace => {
                    self.search_input
                        .handle(tui_input::InputRequest::DeletePrevChar);
                    return Ok(HelpOverlayEvent::Consumed);
                }
                KeyCode::Delete => {
                    self.search_input
                        .handle(tui_input::InputRequest::DeleteNextChar);
                    return Ok(HelpOverlayEvent::Consumed);
                }
                KeyCode::Left => {
                    self.search_input
                        .handle(tui_input::InputRequest::GoToPrevChar);
                    return Ok(HelpOverlayEvent::Consumed);
                }
                KeyCode::Right => {
                    self.search_input
                        .handle(tui_input::InputRequest::GoToNextChar);
                    return Ok(HelpOverlayEvent::Consumed);
                }
                KeyCode::Home => {
                    self.search_input.handle(tui_input::InputRequest::GoToStart);
                    return Ok(HelpOverlayEvent::Consumed);
                }
                KeyCode::End => {
                    self.search_input.handle(tui_input::InputRequest::GoToEnd);
                    return Ok(HelpOverlayEvent::Consumed);
                }
                KeyCode::Enter => {
                    self.apply_search_buffer();
                    return Ok(HelpOverlayEvent::Consumed);
                }
                KeyCode::Esc => {
                    self.search_active = false;
                    return Ok(HelpOverlayEvent::Consumed);
                }
                _ => {}
            }
            return Ok(HelpOverlayEvent::Consumed);
        }

        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => Ok(HelpOverlayEvent::Close),
            (KeyCode::Char('h'), KeyModifiers::CONTROL) => Ok(HelpOverlayEvent::Close),
            (KeyCode::Char('f'), KeyModifiers::CONTROL) | (KeyCode::Char('/'), _) => {
                self.toggle_search_mode();
                Ok(HelpOverlayEvent::Consumed)
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.clear_search();
                Ok(HelpOverlayEvent::Consumed)
            }
            (KeyCode::Up, _) => {
                self.navigate_table(TableNavigation::Up);
                Ok(HelpOverlayEvent::Consumed)
            }
            (KeyCode::Down, _) => {
                self.navigate_table(TableNavigation::Down);
                Ok(HelpOverlayEvent::Consumed)
            }
            (KeyCode::PageUp, _) => {
                self.navigate_table(TableNavigation::PageUp);
                Ok(HelpOverlayEvent::Consumed)
            }
            (KeyCode::PageDown, _) => {
                self.navigate_table(TableNavigation::PageDown);
                Ok(HelpOverlayEvent::Consumed)
            }
            (KeyCode::Home, _) => {
                self.navigate_table(TableNavigation::Home);
                Ok(HelpOverlayEvent::Consumed)
            }
            (KeyCode::End, _) => {
                self.navigate_table(TableNavigation::End);
                Ok(HelpOverlayEvent::Consumed)
            }
            _ => Ok(HelpOverlayEvent::Consumed),
        }
    }

    pub fn snapshot(&mut self) -> HelpRenderSnapshot {
        let rows = self.build_table_data();
        self.table_len = rows.len();

        if let Some(selected) = self.table_state.selected() {
            if selected >= self.table_len && self.table_len > 0 {
                self.table_state.select(Some(0));
            }
        } else if self.table_len > 0 {
            self.table_state.select(Some(0));
        }

        HelpRenderSnapshot {
            search_active: self.search_active,
            search_buffer: self.search_input.value().to_string(),
            search_query: self.search_query.clone(),
            cursor_offset: if self.search_active {
                Some(self.search_input.visual_cursor() as u16)
            } else {
                None
            },
            rows,
            selected: self.table_state.selected(),
        }
    }

    fn toggle_search_mode(&mut self) {
        if self.search_active {
            self.apply_search_buffer();
        } else {
            self.search_active = true;
            if let Some(current) = &self.search_query {
                self.search_input = Input::new(current.clone());
            } else {
                self.search_input = Input::default();
            }
        }
    }

    fn apply_search_buffer(&mut self) {
        let value = self.search_input.value().trim();
        if value.is_empty() {
            self.search_query = None;
        } else {
            self.search_query = Some(value.to_string());
        }
        self.search_active = false;
        self.table_state.select(Some(0));
    }

    fn clear_search(&mut self) {
        self.search_query = None;
        self.search_active = false;
        self.search_input = Input::default();
        self.table_state.select(Some(0));
    }

    fn navigate_table(&mut self, direction: TableNavigation) {
        if self.table_len == 0 {
            return;
        }

        let current = self.table_state.selected().unwrap_or(0);
        let next = match direction {
            TableNavigation::Up => {
                if current == 0 {
                    self.table_len - 1
                } else {
                    current - 1
                }
            }
            TableNavigation::Down => (current + 1) % self.table_len,
            TableNavigation::PageUp => current.saturating_sub(10),
            TableNavigation::PageDown => (current + 10).min(self.table_len.saturating_sub(1)),
            TableNavigation::Home => 0,
            TableNavigation::End => self.table_len.saturating_sub(1),
        };
        self.table_state.select(Some(next));
    }

    fn build_table_data(&self) -> Vec<TableRow> {
        let mut rows = Vec::new();
        let filter = self.search_query.as_ref().map(|s| s.to_ascii_lowercase());
        let mut seen = HashSet::new();

        for context in &self.contexts {
            let exported = self
                .settings
                .export_keymap_for(DeviceFilter::Keyboard, context);
            for (action_name, chords) in exported {
                let key = (action_name.clone(), context.clone());
                if !seen.insert(key) {
                    continue;
                }

                let keys = chords.join(", ");
                let details = describe_action(&action_name);

                let include = match &filter {
                    Some(query) => {
                        let a = action_name.to_ascii_lowercase();
                        let k = keys.to_ascii_lowercase();
                        let c = context.to_ascii_lowercase();
                        let d = details.to_ascii_lowercase();
                        a.contains(query)
                            || k.contains(query)
                            || c.contains(query)
                            || d.contains(query)
                    }
                    None => true,
                };

                if include {
                    rows.push(TableRow {
                        action: action_name,
                        keys,
                        context: context.clone(),
                        details,
                    });
                }
            }
        }

        if rows.is_empty() {
            rows.push(TableRow {
                action: "Help".into(),
                keys: "ctrl+h".into(),
                context: "global".into(),
                details: "Show this help dialog".into(),
            });
        }

        rows.sort_by(|a, b| match a.context.cmp(&b.context) {
            std::cmp::Ordering::Equal => a.action.cmp(&b.action),
            other => other,
        });

        rows
    }
}

pub struct HelpRenderSnapshot {
    search_active: bool,
    search_buffer: String,
    search_query: Option<String>,
    cursor_offset: Option<u16>,
    rows: Vec<TableRow>,
    selected: Option<usize>,
}

impl HelpRenderSnapshot {
    pub fn render(&self, frame: &mut Frame, area: Rect, context: Vec<String>) {
        frame.render_widget(Clear, area);

        let [search_area, table_area] =
            Layout::vertical([Constraint::Length(4), Constraint::Fill(1)]).areas(area);

        let search_block = Block::new()
            .border_type(BorderType::Rounded)
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title("─ Search ");
        frame.render_widget(search_block, search_area);

        if self.search_active {
            let input_area = Rect::new(
                search_area.x + 2,
                search_area.y + 2,
                search_area.width.saturating_sub(4),
                1,
            );
            let prompt = Paragraph::new(self.search_buffer.clone());
            frame.render_widget(prompt, input_area);

            if let Some(cursor) = self.cursor_offset {
                frame.set_cursor_position((input_area.x + cursor, input_area.y));
            }
        } else {
            let input_area = Rect::new(
                search_area.x + 2,
                search_area.y + 2,
                search_area.width.saturating_sub(4),
                1,
            );
            let text = match &self.search_query {
                Some(query) => format!("Filter: {} [Ctrl+C to clear]", query),
                // None => "Press Ctrl+F or / to search keybindings".to_string(),
                None => format!("{:?}", context),
            };
            let widget = Paragraph::new(text).style(Style::default().fg(Color::Gray));
            frame.render_widget(widget, input_area);
        }

        let palette = symbols::border::Set {
            top_left: symbols::line::NORMAL.vertical_right,
            top_right: symbols::line::NORMAL.vertical_left,
            bottom_left: symbols::line::ROUNDED.bottom_left,
            bottom_right: symbols::line::ROUNDED.bottom_right,
            ..symbols::border::PLAIN
        };

        let max_action = self
            .rows
            .iter()
            .map(|row| row.action.len())
            .max()
            .unwrap_or(10)
            .max(6)
            + 2;
        let max_keys = self
            .rows
            .iter()
            .map(|row| row.keys.len())
            .max()
            .unwrap_or(10)
            .max(4)
            + 2;
        let max_context = self
            .rows
            .iter()
            .map(|row| row.context.len())
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

        let rows: Vec<Row> = self
            .rows
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let mut style = if index % 2 == 0 {
                    Style::default().bg(Color::Rgb(40, 40, 60))
                } else {
                    Style::default().bg(Color::Rgb(30, 30, 40))
                };
                if Some(index) == self.selected {
                    style = Style::default()
                        .bg(Color::Blue)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD);
                }
                let marker = if Some(index) == self.selected {
                    "►"
                } else {
                    " "
                };
                Row::new(vec![
                    marker.to_string(),
                    row.action.clone(),
                    row.keys.clone(),
                    row.context.clone(),
                    row.details.clone(),
                ])
                .style(style)
                .height(1)
            })
            .collect();

        let table_block = Block::new()
            .borders(Borders::ALL)
            .border_set(palette)
            .title("─ Key Bindings ");

        let table = Table::new(
            rows,
            [
                Constraint::Length(3),
                Constraint::Length(max_action as u16),
                Constraint::Length(max_keys as u16),
                Constraint::Length(max_context as u16),
                Constraint::Fill(1),
            ],
        )
        .header(header)
        .block(table_block);

        frame.render_widget(table, table_area);
    }
}

struct TableRow {
    action: String,
    keys: String,
    context: String,
    details: String,
}

#[derive(Clone, Copy)]
enum TableNavigation {
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
}

fn describe_action(name: &str) -> String {
    match name {
        "Help" => "Show/hide this help dialog".into(),
        "Quit" => "Exit the application".into(),
        "FocusNext" | "NextField" => "Move focus to next element".into(),
        "FocusPrev" | "PrevField" | "PreviousField" => "Move focus to previous element".into(),
        "NavigateUp" => "Navigate up".into(),
        "NavigateDown" => "Navigate down".into(),
        "NavigateLeft" => "Navigate left".into(),
        "NavigateRight" => "Navigate right".into(),
        "ActivateSelected" => "Activate selected item".into(),
        "ToggleEditMode" | "ModeCycle" => "Toggle edit mode".into(),
        "EnterEditMode" | "ModeInsert" => "Enter edit mode".into(),
        "ExitEditMode" | "ModeNormal" => "Return to normal mode".into(),
        "PageNext" | "NextPage" => "Go to next page".into(),
        "PagePrev" | "PrevPage" | "PreviousPage" => "Go to previous page".into(),
        other => format!("Execute {}", other.replace('_', " ").to_lowercase()),
    }
}
