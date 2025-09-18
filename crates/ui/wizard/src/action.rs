use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    Ui(UiAction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum UiMode {
    Normal,
    Edit,
}
#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum UiAction {
    FocusNext,
    FocusPrev,
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    ActivateSelected,
    ToggleEditMode,
    EnterEditMode,
    ExitEditMode,
}

impl From<UiAction> for Action {
    fn from(value: UiAction) -> Self {
        Action::Ui(value)
    }
}
