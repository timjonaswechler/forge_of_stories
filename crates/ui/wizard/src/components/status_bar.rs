use color_eyre::Result;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    action::{Action, AppAction, NotificationLevel, UiAction, UiMode},
    app::settings::SettingsStore,
    components::Component,
    tui::Event,
};

/// StatusBar component
///
/// Renders a compact bottom bar with:
/// - Page name / context
/// - Focused component id
/// - UI mode (Normal/Edit)
/// - Key hints for the current context (e.g., F1 Help, Esc Normal)
/// - Notification counter
///
/// The component updates its state from incoming Actions:
/// - AppAction::SetKeymapContext sets the current keymap context for hint export
/// - AppAction::SetUiMode and UiAction::{EnterEditMode,ExitEditMode,ToggleEditMode}
/// - UiAction::ReportFocusedComponent updates the focused component id
/// - UiAction::ReportNotificationCount updates the visible notification count
/// - UiAction::ReportHelpVisible toggles whether help-specific hints are shown
pub struct StatusBar {
    tx: Option<UnboundedSender<Action>>,
    settings: Option<Arc<SettingsStore>>,

    // UX data
    page: String,
    context: String,
    focused: Option<String>,
    mode: UiMode,
    notif_count: u32,
    highest_severity: Option<NotificationLevel>,
    help_visible: bool,
}

impl StatusBar {
    /// Create a new status bar for the given page name.
    /// The initial keymap context defaults to "global" and should be updated via AppAction::SetKeymapContext.
    pub fn new(page: &str) -> Self {
        Self {
            tx: None,
            settings: None,
            page: page.to_string(),
            context: "global".to_string(),
            focused: None,
            mode: UiMode::Normal,
            notif_count: 0,
            highest_severity: None,
            help_visible: false,
        }
    }

    fn left_text(&self) -> Line<'static> {
        let page = Span::styled(
            format!(" {} ", self.page),
            Style::default().add_modifier(Modifier::BOLD),
        );
        let ctx = Span::raw(format!("({})  ", self.context));
        // Highlight focused component if present
        let focused_val = self.focused.as_deref().unwrap_or("-");
        let focused = if self.focused.is_some() {
            Span::styled(
                format!("Focus: {}  ", focused_val),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw(format!("Focus: {}  ", focused_val))
        };
        let mode_str = match self.mode {
            UiMode::Normal => "Normal",
            UiMode::Edit => "Edit",
        };
        let mode = Span::raw(format!("Mode: {}", mode_str));
        Line::from(vec![page, ctx, focused, mode])
    }

    fn right_text(&self) -> Line<'static> {
        // Key hints derived from settings keymap (if available)
        let hints = if let Some(settings) = &self.settings {
            let map = settings.export_keymap_for(settings::DeviceFilter::Keyboard, &self.context);
            let mut parts: Vec<String> = Vec::new();

            if self.help_visible {
                if let Some(k) = first_key(&map, "HelpToggleGlobal") {
                    parts.push(format!("{} Global", k));
                }
                if let Some(k) = first_key(&map, "HelpToggleWrap") {
                    let wrap_on = if let Some(store) = &self.settings {
                        store
                            .get::<crate::app::settings::Wizard>()
                            .map(|w| w.help_wrap_on)
                            .unwrap_or(true)
                    } else {
                        true
                    };
                    let wrap_label = if wrap_on { "Wrap On" } else { "Wrap Off" };
                    parts.push(format!("{} {}", k, wrap_label));
                }
                if let Some(k) = first_key(&map, "HelpSearch") {
                    parts.push(format!("{} Search", k));
                }
                if let Some(k) = first_key(&map, "HelpPageUp") {
                    parts.push(format!("{} PgUp", k));
                }
                if let Some(k) = first_key(&map, "HelpPageDown") {
                    parts.push(format!("{} PgDown", k));
                }
                if let Some(k) = first_key(&map, "HelpScrollUp") {
                    parts.push(format!("{} Up", k));
                }
                if let Some(k) = first_key(&map, "HelpScrollDown") {
                    parts.push(format!("{} Down", k));
                }
            } else {
                if let Some(k) = first_key(&map, "Help") {
                    parts.push(format!("{} Help", k));
                }
                if let Some(k) = first_key(&map, "ModeNormal") {
                    parts.push(format!("{} Normal", k));
                }
                if let Some(k) = first_key(&map, "ModeInsert") {
                    parts.push(format!("{} Edit", k));
                }
                if let Some(k) = first_key(&map, "OpenPopup") {
                    parts.push(format!("{} Popup", k));
                }
            }

            if parts.is_empty() {
                "Keys: n/a".to_string()
            } else {
                parts.join(" · ")
            }
        } else {
            "Keys: n/a".to_string()
        };

        let mut spans: Vec<Span<'static>> = vec![Span::raw(hints)];
        if self.notif_count > 0 {
            let color = match self.highest_severity {
                Some(NotificationLevel::Error) => Color::Red,
                Some(NotificationLevel::Warning) => Color::Yellow,
                Some(NotificationLevel::Success) => Color::Green,
                Some(NotificationLevel::Info) => Color::Blue,
                None => Color::White,
            };
            let bell = Span::styled(
                format!("  🔔 {}", self.notif_count),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            );
            spans.push(bell);
        }

        Line::from(spans)
    }
}

impl Component for StatusBar {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn register_settings_handler(&mut self, settings: Arc<SettingsStore>) -> Result<()> {
        self.settings = Some(settings);
        Ok(())
    }

    fn handle_events(&mut self, _event: Option<Event>) -> Result<Option<Action>> {
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Ui(UiAction::ReportFocusedComponent(id)) => {
                self.focused = Some(id);
            }
            Action::Ui(UiAction::EnterEditMode) => self.mode = UiMode::Edit,
            Action::Ui(UiAction::ExitEditMode) => self.mode = UiMode::Normal,
            Action::Ui(UiAction::ToggleEditMode) => {
                self.mode = match self.mode {
                    UiMode::Normal => UiMode::Edit,
                    UiMode::Edit => UiMode::Normal,
                };
            }
            Action::Ui(UiAction::ReportNotificationCount(n)) => {
                self.notif_count = n;
            }
            Action::Ui(UiAction::ReportNotificationSeverity(sev)) => {
                self.highest_severity = sev;
            }
            Action::Ui(UiAction::ReportHelpVisible(v)) => {
                self.help_visible = v;
            }
            Action::App(AppAction::SetUiMode(m)) => self.mode = m,
            Action::App(AppAction::SetKeymapContext { name }) => self.context = name,
            _ => {}
        }
        Ok(None)
    }

    fn set_focused(&mut self, _focused: bool) {
        // StatusBar is not a focus target; ignore focus state.
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        // Split horizontally into left (info) and right (hints+notif)
        let right_w = area.width.min(50);
        let left_w = area.width.saturating_sub(right_w);

        let left_area = Rect::new(area.x, area.y, left_w, area.height);
        let right_area = Rect::new(area.x + left_w, area.y, right_w, area.height);

        let left = Paragraph::new(self.left_text());
        let right = Paragraph::new(self.right_text());

        f.render_widget(left, left_area);
        f.render_widget(right, right_area);

        Ok(())
    }
}

// Helpers

fn first_key(
    map: &std::collections::BTreeMap<String, Vec<String>>,
    action: &str,
) -> Option<String> {
    // Try exact key, then case-insensitive
    if let Some(v) = map.get(action) {
        return v.get(0).cloned().map(prettify_chord);
    }
    if let Some((_, v)) = map
        .iter()
        .find(|(k, _)| k.to_ascii_lowercase() == action.to_ascii_lowercase())
    {
        return v.get(0).cloned().map(prettify_chord);
    }
    None
}

fn prettify_chord(s: String) -> String {
    // Make it look nicer: ctrl+p -> Ctrl+P, f1 -> F1, esc -> Esc
    let mut out = String::new();
    for (i, part) in s.split('+').enumerate() {
        if i > 0 {
            out.push('+');
        }
        out.push_str(&capitalize_key(part));
    }
    out
}

fn capitalize_key(k: &str) -> String {
    let lower = k.to_ascii_lowercase();
    match lower.as_str() {
        "ctrl" => "Ctrl".to_string(),
        "alt" => "Alt".to_string(),
        "shift" => "Shift".to_string(),
        "meta" => "Meta".to_string(),
        "esc" => "Esc".to_string(),
        "enter" => "Enter".to_string(),
        "tab" => "Tab".to_string(),
        s if s.starts_with('f') && s[1..].chars().all(|c| c.is_ascii_digit()) => {
            s.to_ascii_uppercase()
        }
        s if s.len() == 1 => s.to_ascii_uppercase(),
        _ => k.to_string(),
    }
}
