use crate::action::{Action, UiAction};
use settings::keymap::ActionRegistry;

pub struct WizardActionRegistry {
    action_names: Vec<String>,
}

impl WizardActionRegistry {
    pub fn new() -> Self {
        Self {
            action_names: vec![
                // Legacy actions
                "Quit".to_string(),
                "Help".to_string(),
                "Tick".to_string(),
                "Render".to_string(),
                "Resize".to_string(),
                "ClearScreen".to_string(),
                // UI Actions
                "FocusNext".to_string(),
                "FocusPrev".to_string(),
                "NavigateUp".to_string(),
                "NavigateDown".to_string(),
                "NavigateLeft".to_string(),
                "NavigateRight".to_string(),
                "ActivateSelected".to_string(),
                "ToggleEditMode".to_string(),
                "EnterEditMode".to_string(),
                "ExitEditMode".to_string(),
            ],
        }
    }
}

impl ActionRegistry for WizardActionRegistry {
    type Action = Action;

    fn resolve_action(
        &self,
        action_name: &str,
        _action_data: Option<&toml::Value>,
    ) -> Option<Self::Action> {
        match action_name {
            // Legacy actions
            "Quit" => Some(Action::Quit),
            "Help" => Some(Action::Help),
            "Tick" => Some(Action::Tick),
            "Render" => Some(Action::Render),
            "ClearScreen" => Some(Action::ClearScreen),

            // UI Actions - Focus
            "FocusNext" => Some(Action::Ui(UiAction::FocusNext)),
            "FocusPrev" => Some(Action::Ui(UiAction::FocusPrev)),

            // UI Actions - Navigation
            "NavigateUp" => Some(Action::Ui(UiAction::NavigateUp)),
            "NavigateDown" => Some(Action::Ui(UiAction::NavigateDown)),
            "NavigateLeft" => Some(Action::Ui(UiAction::NavigateLeft)),
            "NavigateRight" => Some(Action::Ui(UiAction::NavigateRight)),
            "ActivateSelected" => Some(Action::Ui(UiAction::ActivateSelected)),

            // UI Actions - Edit Mode
            "ToggleEditMode" => Some(Action::Ui(UiAction::ToggleEditMode)),
            "EnterEditMode" => Some(Action::Ui(UiAction::EnterEditMode)),
            "ExitEditMode" => Some(Action::Ui(UiAction::ExitEditMode)),

            _ => None,
        }
    }

    fn get_action_names(&self) -> Vec<String> {
        self.action_names.clone()
    }
}
