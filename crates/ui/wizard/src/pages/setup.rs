use crate::{
    action::{Action, UiAction},
    pages::Page,
};
use color_eyre::Result;
use tokio::sync::mpsc::UnboundedSender;

/// SetupPage: base page without per-page StatusBar (now rendered globally in App).
/// Focus handling is trivial (no intrinsic components).
pub struct SetupPage {
    tx: Option<UnboundedSender<Action>>,
    focused: Option<String>,
}

impl SetupPage {
    pub fn new() -> Self {
        Self {
            tx: None,
            focused: None,
        }
    }
}

impl Page for SetupPage {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn keymap_context(&self) -> &'static str {
        "setup"
    }

    fn id(&self) -> &'static str {
        "setup"
    }

    fn focused_component_id(&self) -> Option<&str> {
        self.focused.as_deref()
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::Ui(UiAction::ReportFocusedComponent(id)) = action {
            self.focused = Some(id);
        }
        Ok(None)
    }
}
