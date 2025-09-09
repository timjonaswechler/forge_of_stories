//! Rendering module (Phase 3.3)
//!
//! Diese Datei kapselt das reine Rendering des Wizard UI Zustands.
//! Ursprünglich befand sich der Code in `WizardApp::render` (core/app.rs).
//! Ziel:
//!   * Entkopplung von Steuer-/Event-Logik (Loop) und Darstellungslogik
//!   * Vorbereitung für spätere Schritte (Reducer, alternative Renderer, Theming-Erweiterungen)
//!
//! Keine Logikänderungen gegenüber der vorherigen Implementierung.
//!
//! Aufrufstelle: `AppLoop` (core/loop.rs) nutzt jetzt `app.render(...)` indirekt über diese Funktion,
//! wenn `WizardApp::render` künftig weiter vereinfacht oder entfernt wird.

use crate::core::app::WizardApp;
use color_eyre::Result;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    text::Span,
    widgets::Paragraph,
};

/// Render den kompletten Frame (Page, optional Popup, Footer).
///
/// Erwartung:
/// - `WizardApp` enthält bereits gültige Seiten + aktiven Index.
/// - Popups (falls vorhanden) werden überlagert.
/// - Footer zeigt Modus + Fokus-Kontext an.
pub fn render(app: &mut WizardApp, frame: &mut Frame<'_>) -> Result<()> {
    // Gesamt-Layout: Inhalt + Statuszeile
    let vertical_layout =
        Layout::vertical(vec![Constraint::Fill(1), Constraint::Length(1)]).split(frame.area());

    // Aktive Page
    if let Some(page) = app.pages.get_mut(app.active_page) {
        page.draw(frame, vertical_layout[0])?;
    }

    // Popup (mit Backdrop)
    if let Some(popup) = app.popup.as_mut() {
        crate::components::popups::render_backdrop(frame, vertical_layout[0]);
        let (min_w, min_h) = popup.popup_min_size().unwrap_or((60, 10));
        let w = min_w.min(vertical_layout[0].width);
        let h = min_h.min(vertical_layout[0].height);
        let dialog = crate::components::popups::centered_rect_fixed(vertical_layout[0], w, h);
        popup.draw(frame, dialog)?;
    }

    // Kontext / Fokus ermitteln (Footer)
    let (context, focused) = if let Some(popup) = app.popup.as_deref() {
        (popup.keymap_context(), popup.name())
    } else if let Some(page) = app.pages.get(app.active_page) {
        (page.keymap_context(), page.focused_component_name())
    } else {
        ("global", "root")
    };

    // Footer Layout (links / rechts)
    let footer_area = vertical_layout[1];
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(footer_area);

    // Linker Teil: [ MODE | context:focus ]
    let mut left_spans: Vec<ratatui::text::Span> = Vec::new();
    left_spans.push(Span::styled(
        format!(" {} ", app.footer_mode.label()),
        app.footer_mode.status_segment_style(&app.theme),
    ));

    let mode_bg = app.theme.mode_bg_color(app.footer_mode);
    let chip_bg = app.theme.chip_bg_color();

    if app.theme.supports_powerline() {
        left_spans.push(Span::styled(
            app.theme.sep_left().to_string(),
            ratatui::style::Style::default().fg(mode_bg).bg(chip_bg),
        ));
    } else {
        left_spans.push(Span::raw(" "));
    }

    let focus_label = format!(" {}:{} ", context, focused);
    left_spans.push(Span::styled(focus_label, app.theme.chip_style()));

    if app.theme.supports_powerline() {
        left_spans.push(Span::styled(
            app.theme.sep_left().to_string(),
            ratatui::style::Style::default().fg(chip_bg),
        ));
    }

    let left_para = Paragraph::new(ratatui::text::Line::from(left_spans))
        .wrap(ratatui::widgets::Wrap { trim: true });

    // Rechter Teil: ggf. farbmodus + context (reversed order)
    let mut right_spans: Vec<ratatui::text::Span> = Vec::new();
    if app.theme.supports_powerline() {
        // Farbschema Chip
        right_spans.push(Span::styled(
            app.theme.sep_right().to_string(),
            ratatui::style::Style::default().fg(chip_bg),
        ));
        right_spans.push(Span::styled(
            format!(" {} ", app.theme.mode_label()),
            app.theme.chip_style(),
        ));
        right_spans.push(Span::raw(" "));
        // Kontext Chip
        right_spans.push(Span::styled(
            app.theme.sep_right().to_string(),
            ratatui::style::Style::default().fg(chip_bg),
        ));
        right_spans.push(Span::styled(
            format!(" {} ", context),
            app.theme.chip_style(),
        ));
    } else {
        right_spans.push(Span::styled(
            format!(" {} ", context),
            app.theme.chip_style(),
        ));
        right_spans.push(Span::raw(" "));
        right_spans.push(Span::styled(
            format!(" {} ", app.theme.mode_label()),
            app.theme.chip_style(),
        ));
    }

    let right_para = Paragraph::new(ratatui::text::Line::from(right_spans))
        .wrap(ratatui::widgets::Wrap { trim: true })
        .alignment(ratatui::layout::Alignment::Right);

    frame.render_widget(left_para, cols[0]);
    frame.render_widget(right_para, cols[1]);

    Ok(())
}
