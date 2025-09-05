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
    layout::{Constraint, Layout},
    prelude::Rect,
    style::{Color, Style, Stylize},
};
use settings::DeviceFilter;
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
}

impl WizardApp {
    pub fn new(cli: Cli, base: AppBase) -> Result<Self> {
        let preflight = crate::components::welcome::run_preflight();

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
                }),
                RunMode::Dashboard => Ok(Self {
                    base,
                    pages: vec![Box::new(DashboardPage::new()?)],
                    active_page: 0,
                    popup: None,
                    should_quit: false,
                    should_suspend: false,
                    preflight,
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
            }),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();

        let mut tui = Tui::new()?;
        tui.enter()?;
        for page in self.pages.iter_mut() {
            page.register_action_handler(action_tx.clone())?;
        }

        for page in self.pages.iter_mut() {
            page.focus()?;
        }
        // Pass preflight results to the active page(s) so they can render immediately
        action_tx
            .send(Action::PreflightResults(self.preflight.clone()))
            .ok();

        loop {
            if let Some(e) = tui.next().await {
                let mut stop_event_propagation = self
                    .popup
                    .as_mut()
                    .and_then(|pane| pane.handle_events(e.clone()).ok())
                    .map(|response| match response {
                        Some(EventResponse::Continue(action)) => {
                            action_tx.send(action).ok();
                            false
                        }
                        Some(EventResponse::Stop(action)) => {
                            action_tx.send(action).ok();
                            true
                        }
                        _ => false,
                    })
                    .unwrap_or(false);
                if !stop_event_propagation {
                    stop_event_propagation = self
                        .pages
                        .get_mut(self.active_page)
                        .and_then(|page| page.handle_events(e.clone()).ok())
                        .map(|response| match response {
                            Some(crate::tui::EventResponse::Continue(action)) => {
                                action_tx.send(action).ok();
                                false
                            }
                            Some(crate::tui::EventResponse::Stop(action)) => {
                                action_tx.send(action).ok();
                                true
                            }
                            _ => false,
                        })
                        .unwrap_or(false);
                }

                if !stop_event_propagation {
                    match e {
                        crate::tui::Event::Quit => {
                            action_tx.send(Action::Quit).ok();
                        }
                        crate::tui::Event::Tick => {
                            action_tx.send(Action::Tick).ok();
                        }
                        crate::tui::Event::Render => {
                            action_tx.send(Action::Render).ok();
                        }
                        crate::tui::Event::Resize(x, y) => {
                            action_tx.send(Action::Resize(x, y)).ok();
                        }
                        crate::tui::Event::Key(key) => {
                            // Centralized key handling via services::keymap_binding
                            let (context, focused) = if let Some(popup) = self.popup.as_deref() {
                                (popup.keymap_context(), popup.name())
                            } else if let Some(page) = self.pages.get(self.active_page) {
                                (page.keymap_context(), page.focused_component_name())
                            } else {
                                ("global", "root")
                            };
                            if let Some(mut a) = crate::services::keymap_binding::action_from_key(
                                &self.base.settings,
                                context,
                                key,
                            ) {
                                // If this is an OpenPopup trigger, build the demo ConfirmPopup here
                                if core::mem::discriminant(&a)
                                    == core::mem::discriminant(&Action::OpenPopup(Box::new(
                                        crate::components::popups::confirm::ConfirmPopup::new(
                                            "", "",
                                        ),
                                    )))
                                {
                                    let popup =
                                        crate::components::popups::confirm::ConfirmPopup::new(
                                            "Confirm",
                                            "Exit wizard?",
                                        )
                                        .ok_label("Exit")
                                        .cancel_label("Cancel");
                                    a = Action::OpenPopup(Box::new(popup));
                                }
                                action_tx.send(a).ok();
                            }
                        }
                        _ => {}
                    }
                }
            }

            while let Ok(action) = action_rx.try_recv() {
                match action {
                    Action::Tick | Action::Render => {}
                    _ => log::debug!("{action}"),
                }
                match action {
                    Action::Tick => {
                        // self.last_tick_key_events.drain(..);
                    }
                    Action::Quit => self.should_quit = true,
                    Action::Suspend => self.should_suspend = true,
                    Action::Resume => self.should_suspend = false,
                    Action::Resize(w, h) => {
                        tui.resize(Rect::new(0, 0, w, h))?;
                        tui.draw(|f| {
                            self.render(f).unwrap_or_else(|err| {
                                action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", err)))
                                    .unwrap();
                            })
                        })?;
                    }
                    Action::Render => {
                        tui.draw(|f| {
                            self.render(f).unwrap_or_else(|err| {
                                action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", err)))
                                    .unwrap();
                            })
                        })?;
                    }

                    Action::OpenPopup(popup) => {
                        self.popup = Some(popup);
                        continue;
                    }
                    Action::ClosePopup => {
                        if self.popup.is_some() {
                            self.popup = None;
                        }
                    }
                    Action::PopupResult(ref result) => {
                        // Forward popup results to the active page even while a popup is open
                        if let Some(page) = self.pages.get_mut(self.active_page) {
                            if let Some(next) = page.update(Action::PopupResult(result.clone()))? {
                                action_tx.send(next).ok();
                            }
                        }
                        // Demo: if user confirmed the demo exit popup, quit the wizard
                        if matches!(result, crate::action::PopupResult::Confirmed) {
                            self.should_quit = true;
                        }
                    }
                    Action::Navigate(page) => {
                        self.active_page = page;
                        action_tx
                            .send(Action::PreflightResults(self.preflight.clone()))
                            .ok();
                    }
                    _ => {}
                }

                if let Some(popup) = &mut self.popup {
                    if let Some(action) = popup.update(action)? {
                        action_tx.send(action).ok();
                        Some(())
                    } else {
                        None
                    };
                } else if let Some(page) = self.pages.get_mut(self.active_page) {
                    if let Some(action) = page.update(action)? {
                        action_tx.send(action).ok();
                        Some(())
                    } else {
                        None
                    };
                }
            }

            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume).ok();
                tui = crate::tui::Tui::new()?;
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame<'_>) -> Result<()> {
        let vertical_layout =
            Layout::vertical(vec![Constraint::Fill(1), Constraint::Length(3)]).split(frame.area());

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
        let keymap = self
            .base
            .settings
            .export_keymap_for(DeviceFilter::Keyboard, context);

        let title = format!(" {} [{}] ", context, focused);
        let keybinds_block = ratatui::widgets::Block::bordered()
            .title(title)
            .border_set(ratatui::symbols::border::ROUNDED)
            .border_style(Style::default().fg(Color::DarkGray));

        let keybind_render_width = keybinds_block.inner(vertical_layout[1]).width;

        let mut lines: Vec<ratatui::text::Line> = Vec::new();
        let mut current: Vec<ratatui::text::Span> = Vec::new();
        let mut width: u16 = 0;

        for (action, chords) in keymap {
            let entry = format!("{} [{}]", action, chords.join(" / "));
            let entry_len = entry.len() as u16;
            let sep_len = if current.is_empty() { 0 } else { 3 };

            if width + sep_len + entry_len > keybind_render_width && !current.is_empty() {
                lines.push(ratatui::text::Line::from(current));
                current = Vec::new();
                width = 0;
            }

            if sep_len > 0 {
                current.push(ratatui::text::Span::raw("   "));
                width += sep_len;
            }

            current.push(ratatui::text::Span::raw(entry));
            width += entry_len;
        }

        if !current.is_empty() {
            lines.push(ratatui::text::Line::from(current).fg(Color::DarkGray));
        }

        let keybind_para = ratatui::widgets::Paragraph::new(ratatui::text::Text::from_iter(lines))
            .block(keybinds_block)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(keybind_para, vertical_layout[1]);
        Ok(())
    }
}
