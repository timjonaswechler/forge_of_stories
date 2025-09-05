use crate::components::Component;
use serde::{Deserialize, Serialize};
use strum::Display;

type Command = String;
type Args = Option<String>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreflightStatus {
    Present,
    Missing,
    Disabled,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreflightItem {
    pub label: String,
    pub status: PreflightStatus,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PopupResult {
    AlertClosed,
    Confirmed,
    Cancelled,
    InputSubmitted(String),
    FormSubmitted(serde_json::Value),
}

#[derive(Serialize, Display)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    Refresh,
    Error(String),
    Help,
    FocusNext,
    FocusPrev,
    Focus,
    UnFocus,
    Up,
    Down,
    Submit,
    SwitchInputMode,
    Update,
    OpenPopup(#[serde(skip)] Box<dyn Component>),
    ClosePopup,
    PopupResult(PopupResult),
    Navigate(usize),
    /// Deliver results of pre-start preflight checks to the UI
    PreflightResults(Vec<PreflightItem>),
    IdleTimeout,
}
