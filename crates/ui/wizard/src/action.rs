use crate::components::Component;
use crate::theme::Mode;
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    SetMode(Mode),
    CycleMode,
    Update,
    OpenPopup(#[serde(skip)] Box<dyn Component>),
    ClosePopup,
    ToggleKeymapOverlay,
    PopupResult(PopupResult),
    Navigate(usize),
    /// Deliver results of pre-start preflight checks to the UI
    PreflightResults(Vec<PreflightItem>),
    IdleTimeout,
}

impl Clone for Action {
    fn clone(&self) -> Self {
        match self {
            Action::Tick => Action::Tick,
            Action::Render => Action::Render,
            Action::Resize(w, h) => Action::Resize(*w, *h),
            Action::Suspend => Action::Suspend,
            Action::Resume => Action::Resume,
            Action::Quit => Action::Quit,
            Action::Refresh => Action::Refresh,
            Action::Error(e) => Action::Error(e.clone()),
            Action::Help => Action::Help,
            Action::FocusNext => Action::FocusNext,
            Action::FocusPrev => Action::FocusPrev,
            Action::Focus => Action::Focus,
            Action::UnFocus => Action::UnFocus,
            Action::Up => Action::Up,
            Action::Down => Action::Down,
            Action::Submit => Action::Submit,
            Action::SwitchInputMode => Action::SwitchInputMode,
            Action::SetMode(m) => Action::SetMode(*m),
            Action::CycleMode => Action::CycleMode,
            Action::Update => Action::Update,
            // Cloning an OpenPopup (with a boxed trait object) isn't supported; map to an Error.
            Action::OpenPopup(_) => Action::Error("Clone not supported for OpenPopup".into()),
            Action::ClosePopup => Action::ClosePopup,
            Action::ToggleKeymapOverlay => Action::ToggleKeymapOverlay,
            Action::PopupResult(r) => Action::PopupResult(r.clone()),
            Action::Navigate(i) => Action::Navigate(*i),
            Action::PreflightResults(items) => Action::PreflightResults(items.clone()),
            Action::IdleTimeout => Action::IdleTimeout,
        }
    }
}
