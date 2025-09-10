use color_eyre::Result;
use tokio::sync::mpsc;

use crate::{
    action::{Action, UiOutcome},
    core::app::WizardApp,
    core::effects::{Effect, InternalEvent, TaskResultKind},
    core::executor::TaskExecutor,
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
        // Internal event channel for executor callbacks
        let (internal_tx, mut internal_rx) = mpsc::unbounded_channel::<InternalEvent>();
        // Initialize TaskExecutor with internal event channel
        let executor = TaskExecutor::new_with_internal_tx(internal_tx.clone());

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
                            // Intent-only: handle quit immediately
                            let effs = crate::core::reducer::reduce_intent(
                                &mut self.app.root_state,
                                crate::core::intent_model::Intent::Quit,
                            );
                            for eff in effs {
                                match eff {
                                    Effect::None => {}
                                    Effect::Log(msg) => {
                                        log::info!("[effect] {msg}");
                                    }
                                    Effect::Async(task) => {
                                        log::info!("[effect] schedule async task: {task}");
                                        executor.spawn(task);
                                    }
                                }
                            }
                            self.app.should_quit = true;
                        }
                        Event::Tick => {
                            action_tx.send(Action::Tick).ok();
                        }
                        Event::Render => {
                            action_tx.send(Action::Render).ok();
                        }
                        Event::Resize(w, h) => {
                            // Intent-only: reduce resize and apply immediately
                            let effs = crate::core::reducer::reduce_intent(
                                &mut self.app.root_state,
                                crate::core::intent_model::Intent::Resize(w, h),
                            );
                            for eff in effs {
                                match eff {
                                    Effect::None => {}
                                    Effect::Log(msg) => {
                                        log::info!("[effect] {msg}");
                                    }
                                    Effect::Async(task) => {
                                        log::info!("[effect] schedule async task: {task}");
                                        executor.spawn(task);
                                    }
                                }
                            }
                            self.tui.resize(Rect::new(0, 0, w, h))?;
                            self.tui.draw(|f| {
                                self.app.render(f).unwrap_or_else(|err| {
                                    action_tx
                                        .send(Action::Error(format!("Failed to draw: {:?}", err)))
                                        .unwrap();
                                })
                            })?;
                        }
                        Event::Key(key) => {
                            // Phase 7.2: Kontext-Fallback (popup > page > global)
                            let mut context_chain: Vec<&str> = Vec::new();
                            if let Some(popup) = self.app.popup.as_deref() {
                                context_chain.push(popup.keymap_context());
                            }
                            if let Some(page) = self.app.pages.get(self.app.active_page) {
                                context_chain.push(page.keymap_context());
                            }
                            // Always include global fallback last
                            context_chain.push("global");

                            if let Some(res) = crate::ui::keymap::mapper::resolve_with_fallback(
                                &self.app.base.settings,
                                &context_chain,
                                key,
                            ) {
                                match res {
                                    crate::ui::keymap::mapper::Resolved::Intent(intent) => {
                                        let effs = crate::core::reducer::reduce_intent(
                                            &mut self.app.root_state,
                                            intent,
                                        );
                                        for eff in effs {
                                            match eff {
                                                crate::core::effects::Effect::None => {}
                                                crate::core::effects::Effect::Log(msg) => {
                                                    log::info!("[effect] {msg}");
                                                }
                                                crate::core::effects::Effect::Async(task) => {
                                                    log::info!(
                                                        "[effect] schedule async task: {task}"
                                                    );
                                                    executor.spawn(task);
                                                }
                                            }
                                        }
                                        // Apply pending navigation directly after handling intents
                                        if let Some(pending) =
                                            self.app.root_state.pending_navigation.take()
                                        {
                                            let target_index = match pending {
                                                crate::core::state::AppState::Setup(_) => 0,
                                                crate::core::state::AppState::Settings(_) => 1,
                                                crate::core::state::AppState::Dashboard(_) => 2,
                                                crate::core::state::AppState::Health(_) => 3,
                                            };
                                            self.app.active_page = target_index
                                                .min(self.app.pages.len().saturating_sub(1));
                                            self.app.root_state.app_state = pending;
                                            action_tx
                                                .send(Action::PreflightResults(
                                                    self.app.preflight.clone(),
                                                ))
                                                .ok();
                                            if self.app.root_state.focus_total == 0 {
                                                self.app.root_state.focus_total = 1;
                                            }
                                        }
                                    }
                                    crate::ui::keymap::mapper::Resolved::UiCommand(cmd) => {
                                        match cmd {
                                            crate::core::intent_model::UiCommand::OpenPopup => {
                                                // Keymap does not produce concrete popups; ignore
                                            }
                                            crate::core::intent_model::UiCommand::ClosePopup => {
                                                self.app.popup = None;
                                            }
                                            crate::core::intent_model::UiCommand::ToggleKeymapOverlay => {
                                                if self.app.popup.is_some() {
                                                    self.app.popup = None;
                                                } else {
                                                    // Build Keymap overlay
                                                    let (context, focused) =
                                                        if let Some(popup) = self.app.popup.as_deref() {
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
                                                    let title =
                                                        format!("Keymap · {} [{}]", context, focused);
                                                    let overlay = crate::components::popups::keymap::KeymapOverlay::new(
                                                        title, entries,
                                                    );
                                                    self.app.popup = Some(Box::new(overlay));
                                                }
                                            }
                                            crate::core::intent_model::UiCommand::RenderNow => {
                                                // Force redraw
                                                self.tui.draw(|f| {
                                                    self.app.render(f).unwrap_or_else(|err| {
                                                        action_tx
                                                            .send(Action::Error(format!(
                                                                "Failed to draw: {:?}",
                                                                err
                                                            )))
                                                            .unwrap();
                                                    })
                                                })?;
                                            }
                                            crate::core::intent_model::UiCommand::OpenAlert { .. } => {
                                                // Not produced by key mappings
                                            }
                                        }
                                    }
                                    crate::ui::keymap::mapper::Resolved::UiOutcome(ref outcome) => {
                                        // Forward outcome to active page (optional handling)
                                        if let Some(page) =
                                            self.app.pages.get_mut(self.app.active_page)
                                        {
                                            if let Some(next) =
                                                page.update(Action::UiOutcome(outcome.clone()))?
                                            {
                                                action_tx.send(next).ok();
                                            }
                                        }
                                        // Central lifecycle: decide whether to close popup
                                        match outcome {
                                            crate::action::UiOutcome::RequestClose
                                            | crate::action::UiOutcome::Confirmed
                                            | crate::action::UiOutcome::Cancelled
                                            | crate::action::UiOutcome::SubmitString(_)
                                            | crate::action::UiOutcome::SubmitJson(_) => {
                                                if self.app.popup.is_some() {
                                                    self.app.popup = None;
                                                }
                                            }
                                            crate::action::UiOutcome::None => {}
                                        }
                                        if matches!(outcome, crate::action::UiOutcome::Confirmed) {
                                            self.app.should_quit = true;
                                        }
                                    }
                                }
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
                let split = crate::core::intent_model::classify_action(&action);
                log::trace!(
                    "[classify] intent={:?} ui_command={:?} internal={:?} outcome={:?}",
                    split.intent,
                    split.ui_command,
                    split.internal_event,
                    split.ui_outcome
                );
                let mut effects =
                    crate::core::reducer::reduce(&mut self.app.root_state, cloned_action);
                if let Some(intent) = split.intent.clone() {
                    let eff2 =
                        crate::core::reducer::reduce_intent(&mut self.app.root_state, intent);
                    effects.extend(eff2);
                }
                // Handle produced effects via executor (Phase 10 integration)
                for eff in effects {
                    match eff {
                        Effect::None => {}
                        Effect::Log(msg) => {
                            log::info!("[effect] {msg}");
                        }
                        Effect::Async(task) => {
                            log::info!("[effect] schedule async task: {task}");
                            executor.spawn(task);
                        }
                    }
                }
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
                        if let Some(_active) = self.app.pages.get(self.app.active_page) {
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

                match &action {
                    Action::Tick | Action::Render => {}
                    _ => log::debug!("{action}"),
                }

                match action {
                    Action::Tick => {
                        // (Platzhalter für periodische Logik)
                    }
                    Action::Quit => {
                        // Handled via Intent reducer; no legacy handling here
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
                        // Legacy adapter: convert old PopupResult into unified UiOutcome
                        let outcome: UiOutcome = result.clone().into();
                        action_tx.send(Action::UiOutcome(outcome)).ok();
                    }
                    Action::UiOutcome(ref outcome) => {
                        // Forward outcome to active page (optional handling)
                        if let Some(page) = self.app.pages.get_mut(self.app.active_page) {
                            if let Some(next) = page.update(Action::UiOutcome(outcome.clone()))? {
                                action_tx.send(next).ok();
                            }
                        }
                        // Central lifecycle: decide whether to close popup
                        match outcome {
                            UiOutcome::RequestClose
                            | UiOutcome::Confirmed
                            | UiOutcome::Cancelled
                            | UiOutcome::SubmitString(_)
                            | UiOutcome::SubmitJson(_) => {
                                if self.app.popup.is_some() {
                                    self.app.popup = None;
                                }
                            }
                            UiOutcome::None => {}
                        }
                        // Preserve previous Confirmed->quit semantic
                        if matches!(outcome, UiOutcome::Confirmed) {
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

            // Process internal events from executor (TaskStarted/TaskLog/TaskFinished)
            while let Ok(event) = internal_rx.try_recv() {
                // Route internal event to reducer and handle effects
                let internal_effects = crate::core::reducer::reduce_internal_event(
                    &mut self.app.root_state,
                    event.clone(),
                );
                for eff in internal_effects {
                    match eff {
                        Effect::None => {}
                        Effect::Log(msg) => {
                            log::info!("[effect] {msg}");
                        }
                        Effect::Async(task) => {
                            log::info!("[effect] schedule async task: {task}");
                            executor.spawn(task);
                        }
                    }
                }
                match event {
                    InternalEvent::TaskStarted { id, label } => {
                        log::info!("[task:{id}] started {label}");
                    }
                    InternalEvent::TaskLog { id, message } => {
                        log::info!("[task:{id}] {message}");
                    }
                    InternalEvent::PreflightUpdated(items) => {
                        action_tx.send(Action::PreflightResults(items)).ok();
                    }
                    InternalEvent::TaskFinished { id, result } => {
                        match result.clone() {
                            TaskResultKind::CertGenerated { cn, .. } => {
                                log::info!("[task:{id}] certificate generated for {cn}");
                                // UI feedback: show an alert popup to confirm success
                                let cmd = crate::core::intent_model::UiCommand::OpenAlert {
                                    title: "Certificate generated".to_string(),
                                    message: format!("Certificate generated for {cn}"),
                                };
                                if let crate::core::intent_model::UiCommand::OpenAlert {
                                    title,
                                    message,
                                } = cmd
                                {
                                    let popup = crate::components::popups::alert::AlertPopup::new(
                                        title, message,
                                    );
                                    self.app.popup = Some(Box::new(popup));
                                }

                                // Auto-schedule persistence when an output path is provided
                                if let TaskResultKind::CertGenerated {
                                    cert_pem,
                                    key_pem,
                                    output_path: Some(output_path),
                                    ..
                                } = result.clone()
                                {
                                    let task = crate::core::effects::TaskKind::PersistCert {
                                        output_path,
                                        cert_pem,
                                        key_pem,
                                    };
                                    executor.spawn(task);
                                }
                            }
                            TaskResultKind::CertFailed { cn, error } => {
                                log::warn!(
                                    "[task:{id}] certificate generation failed for {cn}: {error}"
                                );
                            }
                            TaskResultKind::Persisted { path } => {
                                log::info!("[task:{id}] persisted certificate and key at {path}");
                                let cmd = crate::core::intent_model::UiCommand::OpenAlert {
                                    title: "Certificate saved".to_string(),
                                    message: format!("Saved to {}", path),
                                };
                                if let crate::core::intent_model::UiCommand::OpenAlert {
                                    title,
                                    message,
                                } = cmd
                                {
                                    let popup = crate::components::popups::alert::AlertPopup::new(
                                        title, message,
                                    );
                                    self.app.popup = Some(Box::new(popup));
                                }
                            }
                            TaskResultKind::PersistFailed { path, error } => {
                                log::warn!(
                                    "[task:{id}] failed to persist certificate/key at {path}: {error}"
                                );
                            }
                        }
                    }
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
