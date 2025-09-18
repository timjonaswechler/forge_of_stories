use ratatui::{
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    action::{Action, UiAction, UiMode},
    layers::ActionOutcome,
    ui::components::{Component, ComponentKey},
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
    id: Option<ComponentKey>,
    mode: UiMode,
    focused_component: Option<ComponentKey>,
    notif_count: usize,
    help_visible: bool,
}

impl StatusBar {
    /// Create a new status bar for the given page name.
    /// The initial keymap context defaults to "global" and should be updated via AppAction::SetKeymapContext.
    pub fn new() -> Self {
        Self {
            id: None,
            mode: UiMode::Normal,
            focused_component: None,
            notif_count: 0,
            help_visible: false,
        }
    }

    fn left_text(&self) -> Line<'static> {
        let mode_str = match self.mode {
            UiMode::Normal => "Normal",
            UiMode::Edit => "Edit",
        };
        let mode = Span::raw(format!("Mode: {}", mode_str));
        Line::from(vec![mode])
    }
}

impl Component for StatusBar {
    fn name(&self) -> &str {
        "status_bar"
    }

    fn id(&self) -> ComponentKey {
        self.id.expect("Component ID not set")
    }

    fn set_id(&mut self, id: ComponentKey) {
        self.id = Some(id);
    }

    fn focusable(&self) -> bool {
        false
    }

    fn handle_action(&mut self, action: &Action) -> ActionOutcome {
        match action {
            Action::Ui(UiAction::EnterEditMode) => {
                self.mode = UiMode::Edit;
                ActionOutcome::Consumed
            }
            Action::Ui(UiAction::ExitEditMode) => {
                self.mode = UiMode::Normal;
                ActionOutcome::Consumed
            }
            Action::Ui(UiAction::ToggleEditMode) => {
                self.mode = match self.mode {
                    UiMode::Normal => UiMode::Edit,
                    UiMode::Edit => UiMode::Normal,
                };
                ActionOutcome::Consumed
            }

            _ => ActionOutcome::NotHandled,
        }
    }

    fn render(&self, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let left = Paragraph::new(self.left_text());
        f.render_widget(left, area);
    }
}

// Helpers

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
