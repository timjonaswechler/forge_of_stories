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

        match cli.cmd {
            Cmd::Run { mode } => match mode {
                RunMode::Setup => Ok(Self {
                    base,
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
        let vertical_layout =
            Layout::vertical(vec![Constraint::Fill(1), Constraint::Length(1)]).split(frame.area());

        if let Some(page) = self.pages.get_mut(self.active_page) {
            page.draw(frame, vertical_layout[0])?;
        };
        // If a popup is active, draw a backdrop and the popup centered on top of the page
        if let Some(popup) = self.popup.as_mut() {
            crate::components::popups::render_backdrop(frame, vertical_layout[0]);
            let (min_w, min_h) = popup.popup_min_size().unwrap_or((60, 10));
            let w = min_w.min(vertical_layout[0].width);
            let h = min_h.min(vertical_layout[0].height);
            let dialog = crate::components::popups::centered_rect_fixed(vertical_layout[0], w, h);
            popup.draw(frame, dialog)?;
        }

        // Determine active keymap context and focused component for footer
        let (context, focused) = if let Some(popup) = self.popup.as_deref() {
            (popup.keymap_context(), popup.name())
        } else if let Some(page) = self.pages.get(self.active_page) {
            (page.keymap_context(), page.focused_component_name())
        } else {
            ("global", "root")
        };
        // nvim-like single line: left [MODE  context:focus], right [ color-mode   context]
        let footer_area = vertical_layout[1];
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(footer_area);

        let mut left_spans: Vec<ratatui::text::Span> = Vec::new();
        // Mode segment with background
        left_spans.push(ratatui::text::Span::styled(
            format!(" {} ", self.footer_mode.label()),
            self.footer_mode.status_segment_style(&self.theme),
        ));
        // Powerline arrow into a chip background for focus label
        let mode_bg = self.theme.mode_bg_color(self.footer_mode);
        let chip_bg = self.theme.chip_bg_color();
        if self.theme.supports_powerline() {
            left_spans.push(ratatui::text::Span::styled(
                self.theme.sep_left().to_string(),
                Style::default().fg(mode_bg).bg(chip_bg),
            ));
        } else {
            left_spans.push(ratatui::text::Span::raw(" "));
        }
        let focus_label = format!(" {}:{} ", context, focused);
        left_spans.push(ratatui::text::Span::styled(
            focus_label,
            self.theme.chip_style(),
        ));
        // Fade out back to default background
        if self.theme.supports_powerline() {
            left_spans.push(ratatui::text::Span::styled(
                self.theme.sep_left().to_string(),
                Style::default().fg(chip_bg),
            ));
        }
        let left_para = ratatui::widgets::Paragraph::new(ratatui::text::Line::from(left_spans))
            .wrap(ratatui::widgets::Wrap { trim: true });

        let mut right_spans: Vec<ratatui::text::Span> = Vec::new();
        // Build right side chips with powerline separators if available
        if self.theme.supports_powerline() {
            // First: color mode chip
            right_spans.push(ratatui::text::Span::styled(
                self.theme.sep_right().to_string(),
                Style::default().fg(chip_bg),
            ));
            right_spans.push(ratatui::text::Span::styled(
                format!(" {} ", self.theme.mode_label()),
                self.theme.chip_style(),
            ));
            right_spans.push(ratatui::text::Span::raw(" "));
            // Then: context chip
            right_spans.push(ratatui::text::Span::styled(
                self.theme.sep_right().to_string(),
                Style::default().fg(chip_bg),
            ));
            right_spans.push(ratatui::text::Span::styled(
                format!(" {} ", context),
                self.theme.chip_style(),
            ));
        } else {
            right_spans.push(ratatui::text::Span::styled(
                format!(" {} ", context),
                self.theme.chip_style(),
            ));
            right_spans.push(ratatui::text::Span::raw(" "));
            right_spans.push(ratatui::text::Span::styled(
                format!(" {} ", self.theme.mode_label()),
                self.theme.chip_style(),
            ));
        }
        let right_para = ratatui::widgets::Paragraph::new(ratatui::text::Line::from(right_spans))
            .wrap(ratatui::widgets::Wrap { trim: true })
            .alignment(ratatui::layout::Alignment::Right);

        frame.render_widget(left_para, cols[0]);
        frame.render_widget(right_para, cols[1]);
        Ok(())
    }
}
