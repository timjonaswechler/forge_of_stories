use crate::action::{Action, UiAction};
use settings::keymap::ActionRegistry;
use toml::Value;

pub struct WizardActionRegistry {
    action_names: Vec<String>,
}

impl WizardActionRegistry {
    pub fn new() -> Self {
        Self {
            action_names: vec![
                "Quit".to_string(),
                "Help".to_string(),
                "Tick".to_string(),
                "Render".to_string(),
                "ClearScreen".to_string(),
                "Suspend".to_string(),
                "Resume".to_string(),
                "FocusNext".to_string(),
                "FocusPrev".to_string(),
                "FocusComponent".to_string(),
                "PageNext".to_string(),
                "PagePrev".to_string(),
                "PageSet".to_string(),
                "ItemNext".to_string(),
                "ItemPrev".to_string(),
                "ItemSelect".to_string(),
                "ItemSet".to_string(),
                "PopupOpen".to_string(),
                "PopupClose".to_string(),
                "ToggleEditMode".to_string(),
                "EnterEditMode".to_string(),
                "ExitEditMode".to_string(),
            ],
        }
    }

    fn parse_target(data: Option<&Value>, keys: &[&str]) -> Option<String> {
        let table = data?.as_table()?;
        keys.iter()
            .find_map(|key| table.get(*key).and_then(|v| v.as_str()))
            .map(|s| s.to_string())
    }
}

impl ActionRegistry for WizardActionRegistry {
    type Action = Action;

    fn resolve_action(
        &self,
        action_name: &str,
        action_data: Option<&toml::Value>,
    ) -> Option<Self::Action> {
        let trimmed = action_name.trim();
        let lower = trimmed.to_ascii_lowercase();
        match lower.as_str() {
            "quit" => Some(Action::Quit),
            "help" => Some(Action::Help),
            "tick" => Some(Action::Tick),
            "render" => Some(Action::Render),
            "clearscreen" => Some(Action::ClearScreen),
            "suspend" => Some(Action::Suspend),
            "resume" => Some(Action::Resume),

            "focusnext" | "nextfocus" => Some(Action::Ui(UiAction::FocusNext)),
            "focusprev" | "prevfocus" => Some(Action::Ui(UiAction::FocusPrev)),

            "focuscomponent" | "thiscomponent" => {
                Self::parse_target(action_data, &["id", "name", "component"])
                    .map(|id| Action::Ui(UiAction::FocusComponent { id }))
            }

            "pagenext" | "nextpage" => Some(Action::Ui(UiAction::PageNext)),
            "pageprev" | "prevpage" => Some(Action::Ui(UiAction::PagePrev)),
            "pageset" | "thispage" => Self::parse_target(action_data, &["id", "name", "page"])
                .map(|id| Action::Ui(UiAction::PageSet { id })),

            "itemnext" | "nextitem" | "navigatedown" | "navigateright" => {
                Some(Action::Ui(UiAction::ItemNext))
            }
            "itemprev" | "previtem" | "navigateup" | "navigateleft" => {
                Some(Action::Ui(UiAction::ItemPrev))
            }
            "itemselect" | "selectitem" | "activateselected" => {
                Some(Action::Ui(UiAction::ItemSelect))
            }
            "itemset" | "thisitem" => Self::parse_target(action_data, &["id", "name", "item"])
                .map(|id| Action::Ui(UiAction::ItemSet { id })),

            "popupopen" | "openpopup" | "showpopup" => {
                Self::parse_target(action_data, &["id", "name", "popup"])
                    .map(|id| Action::Ui(UiAction::PopupOpen { id }))
            }
            "popupclose" | "closepopup" | "closetoppopup" | "closeallpopups" => {
                Some(Action::Ui(UiAction::PopupClose))
            }

            "toggleeditmode" | "modecycle" => Some(Action::Ui(UiAction::ToggleEditMode)),
            "entereditmode" | "modeinsert" => Some(Action::Ui(UiAction::EnterEditMode)),
            "exiteditmode" | "modenormal" => Some(Action::Ui(UiAction::ExitEditMode)),

            _ => None,
        }
    }

    fn get_action_names(&self) -> Vec<String> {
        self.action_names.clone()
    }
}
