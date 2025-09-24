use crate::{
    components::{AetherStatusListComponent, Info, Logo},
    layers::{
        Slots,
        page::{PageBuilder, PageSpec},
    },
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum DashboardSlot {
    WizardLogo,
    AetherStatus,
    ActionsHint,
    WelcomeMessage,
}

pub struct DashboardPage {
    #[allow(dead_code)]
    name: &'static str,
}

impl DashboardPage {
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }
}

impl PageSpec for DashboardPage {
    fn build(self, name: &str, b: &mut PageBuilder<'_>) {
        b.title(name);
        b.layout::<DashboardSlot>(dashboard_layout);
        let logo = b.component(Logo::new("Wizard"));
        b.place_in_slot(logo, DashboardSlot::WizardLogo);

        let status = b.component(AetherStatusListComponent::new());
        b.place_in_slot(status, DashboardSlot::AetherStatus);

        let actions = b.component(
            Info::new("Actions")
                .add_line("Bind a key to: PopupOpen { popup = 'certificate' }")
                .add_line("Then press it to open the Certificate popup (self-signed generation).")
                .add_line("Esc closes popups. Ctrl+H shows help."),
        );
        b.place_in_slot(actions, DashboardSlot::ActionsHint);

        let welcome = b.component(
            Info::new("Welcome!")
                .add_line("Use arrow keys to navigate.")
                .add_line("Press Enter to activate."),
        );
        b.place_in_slot(welcome, DashboardSlot::WelcomeMessage);
    }
}

fn dashboard_layout(area: Rect) -> Slots<DashboardSlot> {
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
        .with(DashboardSlot::WizardLogo, vertical[0])
        .with(DashboardSlot::AetherStatus, vertical[1])
        .with(DashboardSlot::ActionsHint, vertical[2])
        .with(DashboardSlot::WelcomeMessage, vertical[3])
}
