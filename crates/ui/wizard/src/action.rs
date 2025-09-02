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

#[derive(Debug, Clone, PartialEq, Serialize, Display, Deserialize)]
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
    Update,
    OpenPopup,
    ClosePopup,
    Navigate(usize),
    /// Deliver results of pre-start preflight checks to the UI
    PreflightResults(Vec<PreflightItem>),
    IdleTimeout,
}
