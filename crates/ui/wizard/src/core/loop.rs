use color_eyre::Result;
use tokio::sync::mpsc;

use crate::{
    action::Action,
    core::app::WizardApp,
    tui::{EventResponse, Tui},
};

use ratatui::prelude::Rect;

/// The extracted application event loop (Phase 3.2).
///
/// Responsibilities (unchanged from the ursprünglichen `WizardApp::run()`):
/// - Event polling (input + tick + render scheduling)
/// - Event → Action Dispatch
/// - Action processing (state mutation, popup/page routing)
/// - Rendering & resize handling
/// - Suspend/Resume lifecycle
///
/// This struct purposefully owns the TUI loop resources while borrowing the
/// mutable `WizardApp` state (`&mut WizardApp`). This decouples loop mechanics
/// from application state representation and prepares later phases:
///   * Reducer / Intent Layer
///   * Effect handling
///   * Separate rendering module
pub struct AppLoop<'a> {
    app: &'a mut WizardApp,
    tui: Tui,
}

impl<'a> AppLoop<'a> {
    /// Create a new loop wrapper for the given app state.
    ///
    /// `Tui::new()` kann scheitern; da der vorherige Code an dieser Stelle
    /// keine Fehler propagierte (alles in `run()`), verwenden wir hier ein
    /// `expect`, um die Signatur (`new()` ohne Result) beizubehalten.
    pub fn new(app: &'a mut WizardApp) -> Self {
        let tui = Tui::new().expect("failed to initialize TUI");
        Self { app, tui }
    }

    /// Run the event loop until the application requests quit.
    ///
    /// 1:1 Portierung der früheren `WizardApp::run` Logik (nur strukturell verschoben).
    pub async fn run(&mut self) -> Result<()> {
        // Action channel (unbounded wie vorher)
        let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();

        // Enter terminal UI
        self.tui.enter()?;

        // Register action handlers on pages
        for page in self.app.pages.iter_mut() {
            page.register_action_handler(action_tx.clone())?;
        }

        // Initial focus
        for page in self.app.pages.iter_mut() {
            page.focus()?;
        }

        // Preflight Ergebnisse direkt injizieren
        action_tx
            .send(Action::PreflightResults(self.app.preflight.clone()))
            .ok();

        loop {
            // 1. Input / system events → Actions
            if let Some(ev) = self.tui.next().await {
                let mut stop_event_propagation = self
                    .app
                    .popup
                    .as_mut()
                    .and_then(|p| p.handle_events(ev.clone()).ok())
                    .map(|resp| match resp {
                        Some(EventResponse::Continue(a)) => {
                            action_tx.send(a).ok();
                            false
                        }
                        Some(EventResponse::Stop(a)) => {
                            action_tx.send(a).ok();
                            true
                        }
                        _ => false,
                    })
                    .unwrap_or(false);

                if !stop_event_propagation {
                    stop_event_propagation = self
                        .app
                        .pages
                        .get_mut(self.app.active_page)
                        .and_then(|page| page.handle_events(ev.clone()).ok())
                        .map(|resp| match resp {
                            Some(EventResponse::Continue(a)) => {
                                action_tx.send(a).ok();
                                false
                            }
                            Some(EventResponse::Stop(a)) => {
                                action_tx.send(a).ok();
                                true
                            }
                            _ => false,
                        })
                        .unwrap_or(false);
                }

                if !stop_event_propagation {
                    use crate::tui::Event;
                    match ev {
                        Event::Quit => {
                            action_tx.send(Action::Quit).ok();
                        }
                        Event::Tick => {
                            action_tx.send(Action::Tick).ok();
                        }
                        Event::Render => {
                            action_tx.send(Action::Render).ok();
                        }
                        Event::Resize(w, h) => {
                            action_tx.send(Action::Resize(w, h)).ok();
                        }
                        Event::Key(key) => {
                            // Zentralisiertes Key-Handling (unverändert)
                            let (context, _focused) = if let Some(popup) = self.app.popup.as_deref()
                            {
                                (popup.keymap_context(), popup.name())
                            } else if let Some(page) = self.app.pages.get(self.app.active_page) {
                                (page.keymap_context(), page.focused_component_name())
                            } else {
                                ("global", "root")
                            };
                            if let Some(mut a) =
                                crate::ui::action_from_key(&self.app.base.settings, context, key)
                            {
                                // Demo-Popup öffnen falls generische OpenPopup-Markierung
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

            // 2. Actions aus Channel konsumieren
            while let Ok(action) = action_rx.try_recv() {
                // Phase 4.2: Reducer prototype integration (Quit, Navigate, Resize routed via root_state)
                // Clone the action so the reducer can consume it without moving
                let cloned_action = action.clone();
                let _effects =
                    crate::core::reducer::reduce(&mut self.app.root_state, cloned_action);
                // Bridge reducer-derived flags back into legacy WizardApp fields:
                if self.app.root_state.quit_requested {
                    self.app.should_quit = true;
                }
                if let Some(pending) = self.app.root_state.pending_navigation.take() {
                    // Map high-level AppState back to legacy page index (temporary adapter)
                    let target_index = match pending {
                        crate::core::state::AppState::Setup(_) => 0,
                        crate::core::state::AppState::Settings(_) => 1,
                        crate::core::state::AppState::Dashboard(_) => 2,
                        crate::core::state::AppState::Health(_) => 3,
                    };
                    self.app.active_page = target_index.min(self.app.pages.len().saturating_sub(1));
                    // Mirror new high-level state
                    self.app.root_state.app_state = pending;
                    // Maintain existing preflight refresh behavior
                    action_tx
                        .send(Action::PreflightResults(self.app.preflight.clone()))
                        .ok();

                    // Phase 4.3-A Focus bridge:
                    // Page logic still handles FocusNext/FocusPrev internally, but we expose a bridge
                    // so future phases can drive page focus purely from reducer state.
                    if matches!(action, Action::FocusNext | Action::FocusPrev) {
                        // Record (or refresh) focus_total from active page heuristically if possible.
                        // For now we only update the diagnostic counter from known page types.
                        if let Some(active) = self.app.pages.get(self.app.active_page) {
                            // Downcasting skipped (trait object); leave focus_total as-is unless zero.
                            if self.app.root_state.focus_total == 0 {
                                // Fallback assumption: at least 1 focusable element present.
                                self.app.root_state.focus_total = 1;
                            }
                        }
                        // NOTE: Actual per-page focus mutation still occurs in the page's own update().
                        // The reducer's focus_index is currently informational / diagnostic.
                    }
                }

                match action {
                    Action::Tick | Action::Render => {}
                    _ => log::debug!("{action}"),
                }

                match action {
                    Action::Tick => {
                        // (Platzhalter für periodische Logik)
                    }
                    Action::Quit => {
                        // Already applied via reducer (Phase 4.2 bridge)
                    }
                    Action::Suspend => self.app.should_suspend = true,
                    Action::Resume => self.app.should_suspend = false,
                    Action::Resize(w, h) => {
                        self.tui.resize(Rect::new(0, 0, w, h))?;
                        self.tui.draw(|f| {
                            self.app.render(f).unwrap_or_else(|err| {
                                action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", err)))
                                    .unwrap();
                            })
                        })?;
                    }
                    Action::Render => {
                        self.tui.draw(|f| {
                            self.app.render(f).unwrap_or_else(|err| {
                                action_tx
                                    .send(Action::Error(format!("Failed to draw: {:?}", err)))
                                    .unwrap();
                            })
                        })?;
                    }
                    Action::OpenPopup(popup) => {
                        self.app.popup = Some(popup);
                        continue;
                    }
                    Action::ToggleKeymapOverlay => {
                        if self.app.popup.is_some() {
                            self.app.popup = None;
                        } else {
                            // Keymap Overlay aufbauen
                            let (context, focused) = if let Some(popup) = self.app.popup.as_deref()
                            {
                                (popup.keymap_context(), popup.name())
                            } else if let Some(page) = self.app.pages.get(self.app.active_page) {
                                (page.keymap_context(), page.focused_component_name())
                            } else {
                                ("global", "root")
                            };
                            let mut entries = crate::ui::mappable_entries_for_context(
                                &self.app.base.settings,
                                context,
                            );
                            entries.sort_by(|a, b| a.0.cmp(&b.0));
                            let title = format!("Keymap · {} [{}]", context, focused);
                            let overlay = crate::components::popups::keymap::KeymapOverlay::new(
                                title, entries,
                            );
                            self.app.popup = Some(Box::new(overlay));
                        }
                    }
                    Action::ClosePopup => {
                        if self.app.popup.is_some() {
                            self.app.popup = None;
                        }
                    }
                    Action::PopupResult(ref result) => {
                        if let Some(page) = self.app.pages.get_mut(self.app.active_page) {
                            if let Some(next) = page.update(Action::PopupResult(result.clone()))? {
                                action_tx.send(next).ok();
                            }
                        }
                        if matches!(result, crate::action::PopupResult::Confirmed) {
                            self.app.should_quit = true;
                        }
                    }
                    Action::Navigate(_page) => {
                        // Navigation now handled by reducer -> pending_navigation bridge (Phase 4.2)
                    }
                    _ => {}
                }

                // Page / Popup Updates nach Action
                if let Some(popup) = &mut self.app.popup {
                    if let Some(next) = popup.update(action)? {
                        action_tx.send(next).ok();
                        Some(())
                    } else {
                        None
                    };
                } else if let Some(page) = self.app.pages.get_mut(self.app.active_page) {
                    if let Some(next) = page.update(action)? {
                        action_tx.send(next).ok();
                        Some(())
                    } else {
                        None
                    };
                }
            }

            // 3. Lifecycle: suspend / quit
            if self.app.should_suspend {
                self.tui.suspend()?;
                action_tx.send(Action::Resume).ok();
                self.tui = Tui::new()?;
                self.tui.enter()?;
            } else if self.app.should_quit {
                self.tui.stop()?;
                break;
            }
        }

        self.tui.exit()?;
        Ok(())
    }
}
