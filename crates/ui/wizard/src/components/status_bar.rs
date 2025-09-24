use ratatui::{
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    action::{Action, UiAction, UiMode},
    components::{Component, ComponentKey},
    layers::ActionOutcome,
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
    focus_surface: Option<String>,
    focus_component: Option<String>,
}

impl StatusBar {
    /// Create a new status bar for the given page name.
    /// The initial keymap context defaults to "global" and should be updated via AppAction::SetKeymapContext.
    pub fn new() -> Self {
        Self {
            id: None,
            mode: UiMode::Normal,
            focus_surface: None,
            focus_component: None,
        }
    }

    fn left_text(&self) -> Line<'static> {
        let mode_str = match self.mode {
            UiMode::Normal => "Normal",
            UiMode::Edit => "Edit",
        };
        let surface = self.focus_surface.as_deref().unwrap_or("-");
        let component = self.focus_component.as_deref().unwrap_or("-");

        Line::from(vec![
            Span::raw(format!("Mode: {}", mode_str)),
            Span::raw(" | Surface: "),
            Span::raw(surface.to_string()),
            Span::raw(" | Component: "),
            Span::raw(component.to_string()),
        ])
    }

    pub fn set_focus_debug(&mut self, surface: Option<String>, component: Option<String>) {
        self.focus_surface = surface;
        self.focus_component = component;
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
    fn kind(&self) -> &'static str {
        "status"
    }
    fn tags(&self) -> &'static [&'static str] {
        &["status_bar"]
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
