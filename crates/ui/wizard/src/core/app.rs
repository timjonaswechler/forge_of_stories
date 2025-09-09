use crate::theme::{Mode, Theme};
use crate::{
    action::{Action, PreflightItem},
    cli::{Cli, Cmd, RunMode},
    components::Component,
    pages::{DashboardPage, HealthPage, Page, SettingsPage, SetupPage},
    tui::{EventResponse, Tui},
};
use app::{AppBase, Application};
use color_eyre::Result;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    prelude::Rect,
    style::Style,
};

use tokio::sync::mpsc;

impl Application for WizardApp {
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    const APP_ID: &'static str = "wizard";

    // eingebettete Assets für Wizard
    const EMBEDDED_SETTINGS_ASSET: Option<&'static str> = Some("settings/wizard-default.toml");
    const EMBEDDED_KEYMAP_ASSET: Option<&'static str> = Some("keymaps/wizard-default.toml");

    // ENV-Integration wie in deinem bisherigen build_wizard_settings_store()
    const ENV_LAYERS_VAR: Option<&'static str> = Some("FOS_WIZARD_ENV_LAYERS");
    const ENV_PREFIX: Option<&'static str> = Some("FOS_WIZARD");

    fn init_platform() -> Result<(), Self::Error> {
        // Falls du die Init gern hier zentral haben willst:
        crate::errors::init()?;
        crate::logging::init()?;
        Ok(())
    }
}

pub struct WizardApp {
    pub base: AppBase,
    // Transitional high-level state machine root (Phase 4.1)
    pub root_state: crate::core::state::RootState,
    pub pages: Vec<Box<dyn Page>>,
    pub active_page: usize,
    pub popup: Option<Box<dyn Component>>,
    pub should_quit: bool,
    pub should_suspend: bool,
    pub preflight: Vec<PreflightItem>,
    pub theme: Theme,
    pub footer_mode: Mode,
}

impl WizardApp {
    pub fn new(cli: Cli, base: AppBase) -> Result<Self> {
        let preflight = crate::components::welcome::run_preflight();
        let theme = Theme::from_env_auto();
        let root_state = crate::core::state::initial_root_state(&cli);

        match cli.cmd {
            Cmd::Run { mode } => match mode {
                RunMode::Setup => Ok(Self {
                    base,
                    root_state,
                    pages: vec![Box::new(SetupPage::new()?), Box::new(SettingsPage::new()?)],
                    active_page: 0,
                    popup: None,
                    should_quit: false,
                    should_suspend: false,
                    preflight,
                    theme: theme.clone(),
                    footer_mode: Mode::Normal,
                }),
                RunMode::Dashboard => Ok(Self {
                    base,
                    root_state,
                    pages: vec![Box::new(DashboardPage::new()?)],
                    active_page: 0,
                    popup: None,
                    should_quit: false,
                    should_suspend: false,
                    preflight,
                    theme: theme.clone(),
                    footer_mode: Mode::Normal,
                }),
            },
            Cmd::Health => Ok(Self {
                base,
                root_state,
                pages: vec![Box::new(HealthPage::new()?)],
                active_page: 0,
                popup: None,
                should_quit: false,
                should_suspend: false,
                preflight,
                theme,
                footer_mode: Mode::Normal,
            }),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // Phase 3.2: Event-Loop wurde in `core/loop.rs` (AppLoop) ausgelagert.
        // Diese Methode delegiert nur noch. (Die Implementierung in loop.rs
        // enthält unverändert die frühere Schleifenlogik.)
        crate::core::r#loop::AppLoop::new(self).run().await
    }

    pub fn render(&mut self, frame: &mut Frame<'_>) -> Result<()> {
        // Phase 3.3: Rendering ausgelagert in ui::render::render
        crate::ui::render::render(self, frame)
    }
}
