use crate::{
    components::{AetherStatusListComponent, Info, Logo},
    layers::{
        Slots,
        page::{PageBuilder, PageSpec},
    },
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum WelcomeSlot {
    WizardLogo,
    AetherStatus,
    WelcomeMessage,
}

pub struct WelcomePage {
    #[allow(dead_code)]
    name: &'static str,
}

impl WelcomePage {
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }
}

impl PageSpec for WelcomePage {
    fn build(self, name: &str, b: &mut PageBuilder<'_>) {
        b.title(name);
        b.layout::<WelcomeSlot>(welcome_layout);
        let logo = b.component(Logo::new("Wizard"));
        b.place_in_slot(logo, WelcomeSlot::WizardLogo);

        let status = b.component(AetherStatusListComponent::new());
        b.place_in_slot(status, WelcomeSlot::AetherStatus);

        let welcome = b.component(
            Info::new("Getting Started")
                .add_line("Run `wizard run setup` to configure Aether.")
                .add_line("Use Tab/Shift+Tab to move focus."),
        );
        b.place_in_slot(welcome, WelcomeSlot::WelcomeMessage);
    }
}

fn welcome_layout(area: Rect) -> Slots<WelcomeSlot> {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),
            Constraint::Length(6),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(area);

    Slots::empty()
        .with(WelcomeSlot::WizardLogo, vertical[0])
        .with(WelcomeSlot::AetherStatus, vertical[1])
        .with(WelcomeSlot::WelcomeMessage, vertical[3])
}
