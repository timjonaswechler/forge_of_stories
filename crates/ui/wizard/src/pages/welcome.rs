use crate::{
    action::{Action, UiAction},
    components::{AetherStatusListComponent, Component, WizardLogoComponent},
    pages::{Page, PageLayout},
};
use color_eyre::Result;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// WelcomePage: shows logo + a minimal Aether status list (settings, cert, uds)
pub struct WelcomePage {
    tx: Option<tokio::sync::mpsc::UnboundedSender<Action>>,
    focused: Option<String>,
    focusables: [&'static str; 1],
    status_component: Option<AetherStatusListComponent>,
}

impl WelcomePage {
    pub fn new() -> Self {
        Self {
            tx: None,
            focused: Some("aether_status".to_string()),
            focusables: ["aether_status"],
            status_component: Some(AetherStatusListComponent::new()),
        }
    }
}

impl Page for WelcomePage {
    fn register_action_handler(
        &mut self,
        tx: tokio::sync::mpsc::UnboundedSender<Action>,
    ) -> Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn provide_components(&mut self) -> Vec<(String, Box<dyn Component>)> {
        let mut out: Vec<(String, Box<dyn Component>)> = vec![(
            "wizard_logo".to_string(),
            Box::new(WizardLogoComponent::new()) as Box<dyn Component>,
        )];
        if let Some(c) = self.status_component.take() {
            out.push(("aether_status".to_string(), Box::new(c)));
        }
        out
    }

    fn focus(&mut self) -> Result<()> {
        if let Some(tx) = &self.tx {
            if let Some(first) = self.focusables.first() {
                let _ = tx.send(Action::Ui(UiAction::ReportFocusedComponent(
                    (*first).to_string(),
                )));
            }
        }
        Ok(())
    }

    fn keymap_context(&self) -> &'static str {
        "welcome"
    }

    fn id(&self) -> &'static str {
        "welcome"
    }

    fn focus_order(&self) -> &'static [&'static str] {
        &["aether_status"]
    }

    fn focused_component_id(&self) -> Option<&str> {
        self.focused.as_deref()
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::Ui(UiAction::ReportFocusedComponent(id)) = &action {
            self.focused = Some(id.clone());
        }
        Ok(None)
    }

    fn layout(&self, area: Rect) -> PageLayout {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(9),
                Constraint::Length(6),
                Constraint::Length(3),
            ])
            .split(area);
        PageLayout::empty()
            .with("wizard_logo", vertical[0])
            .with("aether_status", vertical[1])
            .with("welcome_message", vertical[2])
    }
}
