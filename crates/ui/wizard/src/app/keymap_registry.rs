use crate::action::{Action, AppAction, LogicAction, NotificationLevel, UiAction};
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
                "FocusById".to_string(),
                "NavigateUp".to_string(),
                "NavigateDown".to_string(),
                "NavigateLeft".to_string(),
                "NavigateRight".to_string(),
                "ActivateSelected".to_string(),
                "ToggleEditMode".to_string(),
                "EnterEditMode".to_string(),
                "ExitEditMode".to_string(),
                "OpenPopup".to_string(),
                "ClosePopup".to_string(),
                "CloseTopPopup".to_string(),
                "CloseAllPopups".to_string(),
                "ShowNotification".to_string(),
                "DismissNotification".to_string(),
                "HelpToggleGlobal".to_string(),
                "BeginHelpSearch".to_string(),
                "HelpSearchClear".to_string(),
                "NextPage".to_string(),
                "PrevPage".to_string(),
                // App Actions
                "SetActivePage".to_string(),
                "SetKeymapContext".to_string(),
                "SaveSettings".to_string(),
                "LoadSettings".to_string(),
                // Logic Actions
                "LoadConfig".to_string(),
                "SaveConfig".to_string(),
            ],
        }
    }
}

impl ActionRegistry for WizardActionRegistry {
    type Action = Action;

    fn resolve_action(
        &self,
        action_name: &str,
        action_data: Option<&toml::Value>,
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
            "FocusById" => action_data
                .and_then(|data| data.get("id"))
                .and_then(|id| id.as_str())
                .map(|id| Action::Ui(UiAction::FocusById(id.to_string()))),

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

            // UI Actions - Popups
            "OpenPopup" => action_data
                .and_then(|data| data.get("id"))
                .and_then(|id| id.as_str())
                .map(|id| {
                    let priority = action_data
                        .and_then(|data| data.get("priority"))
                        .and_then(|p| p.as_integer())
                        .map(|p| p as i32);
                    Action::Ui(UiAction::OpenPopup {
                        id: id.to_string(),
                        priority,
                    })
                }),
            "ClosePopup" => action_data
                .and_then(|data| data.get("id"))
                .and_then(|id| id.as_str())
                .map(|id| Action::Ui(UiAction::ClosePopup { id: id.to_string() })),
            "CloseTopPopup" => Some(Action::Ui(UiAction::CloseTopPopup)),
            "CloseAllPopups" => Some(Action::Ui(UiAction::CloseAllPopups)),

            // UI Actions - Notifications
            "ShowNotification" => {
                if let Some(data) = action_data {
                    let id = data.get("id")?.as_str()?.to_string();
                    let message = data.get("message")?.as_str()?.to_string();
                    let level = data
                        .get("level")
                        .and_then(|l| l.as_str())
                        .and_then(|l| match l {
                            "info" => Some(NotificationLevel::Info),
                            "success" => Some(NotificationLevel::Success),
                            "warning" => Some(NotificationLevel::Warning),
                            "error" => Some(NotificationLevel::Error),
                            _ => None,
                        })
                        .unwrap_or(NotificationLevel::Info);
                    let timeout_ms = data
                        .get("timeout_ms")
                        .and_then(|t| t.as_integer())
                        .map(|t| t as u64);

                    Some(Action::Ui(UiAction::ShowNotification(
                        crate::action::Notification {
                            id,
                            level,
                            message,
                            timeout_ms,
                        },
                    )))
                } else {
                    None
                }
            }
            "DismissNotification" => action_data
                .and_then(|data| data.get("id"))
                .and_then(|id| id.as_str())
                .map(|id| Action::Ui(UiAction::DismissNotification { id: id.to_string() })),

            // UI Actions - Help
            "HelpToggleGlobal" => Some(Action::Ui(UiAction::HelpToggleGlobal)),
            "BeginHelpSearch" => Some(Action::Ui(UiAction::BeginHelpSearch)),
            "HelpSearchClear" => Some(Action::Ui(UiAction::HelpSearchClear)),

            // UI Actions - Page Navigation
            "NextPage" => Some(Action::Ui(UiAction::NextPage)),
            "PrevPage" => Some(Action::Ui(UiAction::PrevPage)),

            // App Actions
            "SetActivePage" => action_data
                .and_then(|data| data.get("id"))
                .and_then(|id| id.as_str())
                .map(|id| Action::App(AppAction::SetActivePage { id: id.to_string() })),
            "SetKeymapContext" => action_data
                .and_then(|data| data.get("name"))
                .and_then(|name| name.as_str())
                .map(|name| {
                    Action::App(AppAction::SetKeymapContext {
                        name: name.to_string(),
                    })
                }),
            "SaveSettings" => Some(Action::App(AppAction::SaveSettings)),
            "LoadSettings" => Some(Action::App(AppAction::LoadSettings)),

            // Logic Actions
            "LoadConfig" => Some(Action::Logic(LogicAction::LoadConfig)),
            "SaveConfig" => Some(Action::Logic(LogicAction::SaveConfig)),

            _ => None,
        }
    }

    fn get_action_names(&self) -> Vec<String> {
        self.action_names.clone()
    }
}
