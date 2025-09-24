//! Certificate Popup (Wizard)
//!
//! Minimal vertical slice: when the popup's single component gains focus,
//! it generates a self-signed certificate and private key and writes them
//! to the configured Aether security paths (relative to paths::config_dir()).
//!
//! Usage pattern (from the app):
//! - Register once with `register_certificate_popup(&mut layers)`.
//! - Open via Action::Ui(UiAction::PopupOpen { id: "certificate".into() }).
//! - Close with Esc (already bound globally for popups).
//!
//! Notes:
//! - This implementation runs generation on first focus automatically (no extra keybinding needed).
//! - It reads Aether security settings (cert_path/key_path) when available; otherwise falls back
//!   to "cert.pem" / "key.pem" inside `paths::config_dir()`.
//! - Algorithm: ECDSA P-256 (compatible with rustls/quinn). Extendable later.
//!
//! Bevy-first, network-safe: This is UI-only and cross-platform.

use crate::{
    components::{Component, ComponentKey},
    layers::{
        LayerSystem, Slots,
        popup::{PopupBuilder, PopupKey, PopupSpec},
    },
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use std::{fs, path::PathBuf};

/// Public helper to create/register the popup into the layer system.
/// Returns the `PopupKey` so callers can open it by key later.
///
/// Example wiring:
/// let popup_key = register_certificate_popup(&mut layers);
/// layers.show_popup(popup_key);
pub fn register_certificate_popup(layers: &mut LayerSystem) -> PopupKey {
    layers.create_popup(
        "certificate",
        CertificatePopup::new("Self-Signed Certificate"),
    )
}

/// Concrete popup spec (builder) for the certificate UI.
pub struct CertificatePopup {
    title: String,
}

impl CertificatePopup {
    pub fn new<T: Into<String>>(title: T) -> Self {
        Self {
            title: title.into(),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum CertPopupSlot {
    Body,
}

impl PopupSpec for CertificatePopup {
    fn build(self, _name: &str, b: &mut PopupBuilder<'_>) {
        b.title(self.title);
        b.layout::<CertPopupSlot>(popup_layout);

        let comp = b.component(GenerateDevCertComponent::new());
        b.place_in_slot(comp, CertPopupSlot::Body);
    }
}

fn popup_layout(area: Rect) -> Slots<CertPopupSlot> {
    // Simple one-slot layout using the entire container
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3)])
        .split(area);
    Slots::empty().with(CertPopupSlot::Body, vertical[0])
}

/// Internal state machine for the generator component.
enum GenState {
    Idle,
    Completed {
        cert_path: PathBuf,
        key_path: PathBuf,
    },
    Failed(String),
}

/// Focusable component that performs the actual certificate generation on first focus.
/// Re-running is allowed with ItemSelect (Enter/Space) if bound later in keymap.
struct GenerateDevCertComponent {
    id: Option<ComponentKey>,
    focused: bool,
    state: GenState,
}

impl GenerateDevCertComponent {
    fn new() -> Self {
        Self {
            id: None,
            focused: false,
            state: GenState::Idle,
        }
    }

    fn generate_now() -> Result<(PathBuf, PathBuf), String> {
        // 1) Determine target paths from Aether settings (fallback to defaults).
        #[allow(unused_mut)]
        let mut cert_rel = String::from("cert.pem");
        #[allow(unused_mut)]
        let mut key_rel = String::from("key.pem");

        if let Ok(store) = aether_config::build_server_settings_store() {
            if let Ok(sec) = store.get::<aether_config::Security>() {
                if !sec.cert_path.trim().is_empty() {
                    cert_rel = sec.cert_path.trim().to_string();
                }
                if !sec.key_path.trim().is_empty() {
                    key_rel = sec.key_path.trim().to_string();
                }
            }
        }

        let resolve = |raw: &str| -> PathBuf {
            let p = std::path::Path::new(raw);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                paths::config_dir().join(raw)
            }
        };
        let cert_path = resolve(&cert_rel);
        let key_path = resolve(&key_rel);

        if let Some(parent) = cert_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create cert dir: {e}"))?;
        }
        if let Some(parent) = key_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create key dir: {e}"))?;
        }

        // 2) Build params (ECDSA P-256) with a minimal SAN set (localhost + 127.0.0.1).
        let mut params = rcgen::CertificateParams::new(vec!["localhost".into()]);
        params.alg = &rcgen::PKCS_ECDSA_P256_SHA256;
        // Add 127.0.0.1 as IP SAN (optional, ignore failure)
        #[allow(unused_must_use)]
        {
            use std::net::IpAddr;
            params
                .subject_alt_names
                .push(rcgen::SanType::IpAddress(IpAddr::from([127, 0, 0, 1])));
        }
        // 3) Generate and write PEMs
        let cert =
            rcgen::Certificate::from_params(params).map_err(|e| format!("rcgen params: {e}"))?;
        let cert_pem = cert
            .serialize_pem()
            .map_err(|e| format!("serialize cert: {e}"))?;
        let key_pem = cert.serialize_private_key_pem();

        fs::write(&cert_path, cert_pem)
            .map_err(|e| format!("write cert '{}': {e}", cert_path.display()))?;
        fs::write(&key_path, key_pem)
            .map_err(|e| format!("write key '{}': {e}", key_path.display()))?;

        Ok((cert_path, key_path))
    }

    fn render_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let heading = |text: &str| {
            Line::from(Span::styled(
                String::from(text),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ))
        };

        lines.push(heading("Self-Signed Certificate"));
        lines.push(Line::raw(""));

        match &self.state {
            GenState::Idle => {
                lines.push(Line::raw("Preparing to generate certificate..."));
                lines.push(Line::raw(""));
                lines.push(Line::raw("Focus this popup to start. Close with Esc."));
            }
            GenState::Completed {
                cert_path,
                key_path,
            } => {
                lines.push(Line::from(vec![
                    Span::raw("Status: "),
                    Span::styled(
                        "OK",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
                lines.push(Line::raw(""));
                lines.push(Line::from(vec![
                    Span::styled(
                        "Certificate: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(cert_path.display().to_string()),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        "Private Key: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(key_path.display().to_string()),
                ]));
                lines.push(Line::raw(""));
                lines.push(Line::raw("You can close this popup with Esc."));
            }
            GenState::Failed(err) => {
                lines.push(Line::from(vec![
                    Span::raw("Status: "),
                    Span::styled(
                        "FAILED",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                ]));
                lines.push(Line::raw(""));
                lines.push(Line::from(vec![
                    Span::styled("Error: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(err.clone()),
                ]));
                lines.push(Line::raw(""));
                lines.push(Line::raw(
                    "Close with Esc. Fix paths/permissions, then retry.",
                ));
            }
        }

        lines
    }
}

impl Component for GenerateDevCertComponent {
    fn name(&self) -> &str {
        // Name matters for keymap context (component:<name>) if you later add actions.
        "cert_popup"
    }

    fn id(&self) -> ComponentKey {
        self.id.expect("Component ID not set")
    }

    fn set_id(&mut self, id: ComponentKey) {
        self.id = Some(id);
    }

    fn focusable(&self) -> bool {
        true
    }

    fn kind(&self) -> &'static str {
        "info"
    }

    fn tags(&self) -> &'static [&'static str] {
        &["certificate"]
    }

    fn on_focus(&mut self, gained: bool) {
        self.focused = gained;
        if gained {
            // Run once on initial focus if still idle.
            if let GenState::Idle = self.state {
                match Self::generate_now() {
                    Ok((cert, key)) => {
                        self.state = GenState::Completed {
                            cert_path: cert,
                            key_path: key,
                        };
                        // Status refresh will be triggered by the next Tick action
                    }
                    Err(e) => {
                        self.state = GenState::Failed(e);
                    }
                }
            }
        }
    }

    fn handle_action(&mut self, action: &crate::action::Action) -> crate::layers::ActionOutcome {
        use crate::action::UiAction;
        match action {
            // Optional: allow re-run with ItemSelect if you later bind Enter for this component.
            crate::action::Action::Ui(UiAction::ItemSelect) => match Self::generate_now() {
                Ok((cert, key)) => {
                    self.state = GenState::Completed {
                        cert_path: cert,
                        key_path: key,
                    };
                    crate::layers::ActionOutcome::RefreshStatus
                }
                Err(e) => {
                    self.state = GenState::Failed(e);
                    crate::layers::ActionOutcome::Consumed
                }
            },
            // Redraw periodically to reflect state; Tick is broadcast by the App.
            crate::action::Action::Tick => {
                // If we just completed generation, trigger status refresh once
                if matches!(self.state, GenState::Completed { .. }) {
                    crate::layers::ActionOutcome::RefreshStatus
                } else {
                    crate::layers::ActionOutcome::Consumed
                }
            }
            _ => crate::layers::ActionOutcome::NotHandled,
        }
    }

    fn render(&self, f: &mut ratatui::Frame, body: Rect) {
        // Center a max-width column for readability
        let [col] = Layout::horizontal([Constraint::Min(30)]).areas(body);
        let lines = self.render_lines();
        let p = Paragraph::new(lines).style(Style::default());
        f.render_widget(p, col);
    }
}
