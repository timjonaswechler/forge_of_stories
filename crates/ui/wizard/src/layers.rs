use std::sync::Arc;

use color_eyre::Result;
use crossterm::event::{Event as CEvent, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{
        Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    action::{Action, AppAction, LayerKind, Notification, NotificationLevel, UiAction},
    app::settings::SettingsStore,
    components::Component,
    tui::Event,
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

    fn render_help(&self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        // Centered popup area (~80% of the screen)
        let w = area.width.saturating_sub(area.width / 5);
        let h = area.height.saturating_sub(area.height / 5);
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;
        let popup_area = Rect::new(x, y, w, h);

        // Build a scrollable text from keymaps
        let mut lines: Vec<String> = Vec::new();

        // Helper to push a section (context or global) optionally filtered
        let filter = self.help_search.as_ref().map(|s| s.to_ascii_lowercase());

        let push_section = |title: &str,
                            map: std::collections::BTreeMap<String, Vec<String>>,
                            out: &mut Vec<String>,
                            filter: &Option<String>| {
            out.push(format!("== {} ==", title));
            out.push("Action — Keys".to_string());
            for (action_name, chords) in map {
                let chord_str = chords.join(", ");
                let include = match filter {
                    Some(q) => {
                        let an = action_name.to_ascii_lowercase();
                        let cs = chord_str.to_ascii_lowercase();
                        an.contains(q) || cs.contains(q)
                    }
                    None => true,
                };
                if include {
                    out.push(format!("- {:<20} {}", action_name, chord_str));
                }
            }
            out.push(String::new());
        };

        if let Some(settings) = &self.settings {
            // Context
            let ctx_map =
                settings.export_keymap_for(settings::DeviceFilter::Keyboard, &self.keymap_context);
            push_section(
                &format!("Context ({})", self.keymap_context),
                ctx_map,
                &mut lines,
                &filter,
            );

            // Global (optional)
            if self.show_global && self.keymap_context != "global" {
                let g_map = settings.export_keymap_for(settings::DeviceFilter::Keyboard, "global");
                push_section("Global", g_map, &mut lines, &filter);
            }
        } else {
            // Fallback content
            let mut map = std::collections::BTreeMap::new();
            map.insert("Help".to_string(), vec!["f1".to_string()]);
            map.insert("Quit".to_string(), vec!["ctrl+c".to_string()]);
            push_section("Example (no settings)", map, &mut lines, &filter);
        }

        let title_extra = match (&self.help_search, self.show_global) {
            (Some(_), true) => format!(" — {} [global+search]", self.keymap_context),
            (Some(_), false) => format!(" — {} [search]", self.keymap_context),
            (None, true) => format!(" — {} [global]", self.keymap_context),
            (None, false) => format!(" — {}", self.keymap_context),
        };

        // Compute viewport and scroll positioning
        let content_len: u16 = lines.len() as u16;
        // account for borders inside the block (approx. 2 rows) + 1 reserved line for prompt if active
        let reserved: u16 = if self.help_prompt_active { 1 } else { 0 };
        let viewport_h: u16 = popup_area.height.saturating_sub(2 + reserved);
        let max_pos = content_len.saturating_sub(viewport_h);
        let pos = self.help_scroll.min(max_pos);

        // Reserve one column on the right for the scrollbar and optionally a top row for the prompt
        let para_area = Rect::new(
            popup_area.x,
            popup_area.y + reserved,
            popup_area.width.saturating_sub(1),
            popup_area.height.saturating_sub(reserved),
        );

        let mut para = Paragraph::new(lines.join("\n")).scroll((pos, 0)).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Help{}", title_extra)),
        );
        if self.wrap_on {
            para = para.wrap(Wrap { trim: true });
        }

        // Scrollbar state and widget (vertical, right side)
        let mut sb_state = ScrollbarState::new(content_len as usize).position(pos as usize);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_symbol("█")
            .thumb_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .track_symbol(Some(" "))
            .track_style(Style::default().fg(Color::DarkGray));

        f.render_widget(Clear, popup_area);
        f.render_widget(para, para_area);

        // Draw prompt overlay inside the help block, under the title, when active
        if self.help_prompt_active {
            // One-line area just under the top border/title; leave 1 col for left border and 2 for scrollbar/right border
            let prompt_area = Rect::new(
                popup_area.x + 1,
                popup_area.y + 1,
                popup_area.width.saturating_sub(3),
                1,
            );
            let prompt_value = self.help_input.value().to_string();
            let hint = "  [Enter: search, Esc: clear]";
            let prompt_text = format!("/{}{}", prompt_value, hint);
            let prompt_style = Style::default().fg(Color::Black).bg(Color::Cyan);
            let prompt = Paragraph::new(prompt_text).style(prompt_style);
            f.render_widget(Clear, prompt_area);
            f.render_widget(prompt, prompt_area);
            // Place cursor after '/' + visual cursor within the input
            let cursor = self.help_input.visual_cursor() as u16;
            let cx = prompt_area.x + 1 + 1 + cursor;
            let cy = prompt_area.y;
            f.set_cursor(cx, cy);
        }

        f.render_stateful_widget(scrollbar, popup_area, &mut sb_state);
        Ok(())
    }

    fn render_confirm(&self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
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

    fn render_error_details(&self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
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
    ) -> Result<()> {
        match kind {
            LayerKind::Popup | LayerKind::Overlay => match id {
                Some("help") => self.render_help(f, area)?,
                Some("confirm") => self.render_confirm(f, area)?,
                Some("error_details") => self.render_error_details(f, area)?,
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
                // Notification rendering is delegated to `ToastManager` component.
                // The registry doesn't draw here.
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

/// A simple toast manager component that:
/// - Listens for `UiAction::ShowNotification` and `UiAction::DismissNotification`.
/// - Applies default lifetime from settings if not set.
/// - Renders a stacked list of toasts with severity-based styles.
/// - Emits `UiAction::ReportNotificationCount` when the visible count changes.
pub struct ToastManager {
    tx: Option<tokio::sync::mpsc::UnboundedSender<Action>>,
    settings: Option<Arc<SettingsStore>>,

    // State
    notifications: Vec<Notification>,
    position: ToastPosition,

    // Cached last visible count to avoid spamming the channel every frame
    last_visible_count: u32,
    // Highest severity last reported
    last_highest_severity: Option<NotificationLevel>,
}

impl ToastManager {
    pub fn new() -> Self {
        Self {
            tx: None,
            settings: None,
            notifications: Vec::new(),
            position: ToastPosition::TopRight,
            last_visible_count: 0,
            last_highest_severity: None,
        }
    }

    /// Set where the toast stack should be drawn.
    pub fn set_position(&mut self, pos: ToastPosition) {
        self.position = pos;
    }

    fn settings_notification_cfg(&self) -> (u64, usize) {
        // Returns (lifetime_ms, max_visible)
        if let Some(settings) = &self.settings {
            let map = settings.effective_settings();
            let wiz_tbl = map.get("wizard").and_then(|v| v.as_table());
            let lifetime_ms: u64 = wiz_tbl
                .and_then(|t| t.get("notification_lifetime_ms"))
                .and_then(|v| v.as_integer())
                .map(|n| n as u64)
                .unwrap_or(4000);
            let max_visible: usize = wiz_tbl
                .and_then(|t| t.get("notification_max"))
                .and_then(|v| v.as_integer())
                .map(|n| n as usize)
                .unwrap_or(3);
            (lifetime_ms, max_visible)
        } else {
            (4000, 3)
        }
    }

    fn prune_and_cap(&mut self) {
        let now_ms = now_unix_ms();
        // Keep only those not expired (timeout_ms is absolute deadline)
        self.notifications
            .retain(|n| n.timeout_ms.map(|dl| now_ms < dl).unwrap_or(true));

        // Cap visible number by settings
        let (_, max_visible) = self.settings_notification_cfg();
        if self.notifications.len() > max_visible {
            let keep = self
                .notifications
                .split_off(self.notifications.len() - max_visible);
            self.notifications = keep;
        }

        let visible_count = self.notifications.len() as u32;
        if visible_count != self.last_visible_count {
            self.last_visible_count = visible_count;
            if let Some(tx) = &self.tx {
                let _ = tx.send(Action::Ui(UiAction::ReportNotificationCount(visible_count)));
            }
        }
        // compute and report highest severity
        let highest = self
            .notifications
            .iter()
            .map(|n| n.level)
            .max_by_key(|lvl| match lvl {
                NotificationLevel::Error => 4,
                NotificationLevel::Warning => 3,
                NotificationLevel::Success => 2,
                NotificationLevel::Info => 1,
            });
        if highest != self.last_highest_severity {
            self.last_highest_severity = highest;
            if let Some(tx) = &self.tx {
                let _ = tx.send(Action::Ui(UiAction::ReportNotificationSeverity(highest)));
            }
        }
    }

    fn ensure_deadline(&self, n: &mut Notification) {
        if n.timeout_ms.is_some() {
            return;
        }
        let (lifetime_ms, _) = self.settings_notification_cfg();
        n.timeout_ms = Some(now_unix_ms() + lifetime_ms);
    }
}

impl Component for ToastManager {
    fn register_action_handler(
        &mut self,
        tx: tokio::sync::mpsc::UnboundedSender<Action>,
    ) -> Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn register_settings_handler(&mut self, settings: Arc<SettingsStore>) -> Result<()> {
        self.settings = Some(settings);
        Ok(())
    }

    fn init(&mut self, _area: ratatui::layout::Size) -> Result<()> {
        Ok(())
    }

    fn handle_events(&mut self, _event: Option<Event>) -> Result<Option<Action>> {
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Ui(UiAction::ShowNotification(mut n)) => {
                self.ensure_deadline(&mut n);
                if let Some(pos) = self.notifications.iter().position(|x| x.id == n.id) {
                    self.notifications[pos] = n;
                } else {
                    self.notifications.push(n);
                }
                self.prune_and_cap();
            }
            Action::Ui(UiAction::DismissNotification { id }) => {
                if let Some(pos) = self.notifications.iter().position(|x| x.id == id) {
                    self.notifications.remove(pos);
                }
                self.prune_and_cap();
            }
            // Keep the visible count in sync if someone else prunes (harmless).
            Action::Ui(UiAction::ReportNotificationCount(_)) => {}
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        // Prune/cap on each draw to keep visuals correct.
        self.prune_and_cap();

        if self.notifications.is_empty() {
            return Ok(());
        }

        let width = area.width.min(50);
        let height_per = 3u16;

        // Last notifications are the most recent; stack them accordingly.
        for (i, notif) in self.notifications.iter().rev().enumerate() {
            let i = i as u16;
            let n_area = toast_rect(area, width, height_per, i, self.position);
            let (fg, bg) = toast_colors(notif.level);
            let style = Style::default().fg(fg).bg(bg);
            let title = format!("Notification — {:?}", notif.level);
            let block = Block::default()
                .borders(Borders::ALL)
                .style(style)
                .title(title);

            let para = Paragraph::new(notif.message.clone())
                .style(style)
                .block(block);

            // Clear background for the toast and render
            f.render_widget(Clear, n_area);
            f.render_widget(para, n_area);
        }

        Ok(())
    }
}

//
// Helpers
//

fn toast_colors(level: NotificationLevel) -> (Color, Color) {
    match level {
        NotificationLevel::Info => (Color::White, Color::Blue),
        NotificationLevel::Success => (Color::Black, Color::Green),
        NotificationLevel::Warning => (Color::Black, Color::Yellow),
        NotificationLevel::Error => (Color::White, Color::Red),
    }
}

fn toast_rect(area: Rect, w: u16, h: u16, index: u16, pos: ToastPosition) -> Rect {
    let x_left = area.x;
    let x_right = area.x + area.width.saturating_sub(w);

    match pos {
        ToastPosition::TopRight => Rect::new(x_right, area.y + index * h, w, h),
        ToastPosition::BottomRight => {
            let y0 = area.y + area.height.saturating_sub((index + 1) * h);
            Rect::new(x_right, y0, w, h)
        }
        ToastPosition::TopLeft => Rect::new(x_left, area.y + index * h, w, h),
        ToastPosition::BottomLeft => {
            let y0 = area.y + area.height.saturating_sub((index + 1) * h);
            Rect::new(x_left, y0, w, h)
        }
    }
}

fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

//
// Small utility to render a titled line (unused currently, kept for future layer expansions)
//
