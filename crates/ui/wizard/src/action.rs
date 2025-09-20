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
    /// Move focus to the next focusable component on the active surface.
    FocusNext,
    /// Move focus to the previous focusable component on the active surface.
    FocusPrev,
    /// Focus a concrete component by its identifier ("ThisComponent").
    FocusComponent { id: String },

    /// Activate the next page registered in the layer system.
    PageNext,
    /// Activate the previous page registered in the layer system.
    PagePrev,
    /// Activate a concrete page by its identifier ("ThisPage").
    PageSet { id: String },

    /// Move to the next item inside the focused component.
    ItemNext,
    /// Move to the previous item inside the focused component.
    ItemPrev,
    /// Select/activate the current item of the focused component.
    ItemSelect,
    /// Select or focus a concrete item within the focused component.
    ItemSet { id: String },

    /// Open a popup layer by identifier.
    PopupOpen { id: String },
    /// Close the currently visible popup layer (if any).
    PopupClose,

    /// Convenience alias for toggling edit mode.
    ToggleEditMode,
    /// Enter edit mode explicitly.
    EnterEditMode,
    /// Leave edit mode explicitly.
    ExitEditMode,
}

impl From<UiAction> for Action {
    fn from(value: UiAction) -> Self {
        Action::Ui(value)
    }
}
