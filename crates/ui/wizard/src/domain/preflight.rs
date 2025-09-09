use crate::action::{PreflightItem, PreflightStatus};
use color_eyre::eyre::Result;

/// Domain: Preflight
///
/// Phase 8.1 – Extraction of preflight / environment detection logic out of the
/// `welcome` component. The UI layer now only renders provided data and no
/// longer owns the detection concerns.
///
/// Responsibilities:
/// - Perform a series of environment / system checks (currently placeholders)
/// - Return structured results (`PreflightItem`) that the UI can display
///
/// Future ideas:
/// - Add severity levels / remediation hints
/// - Parallelize slow checks (async + join)
/// - Persist last run & only diff changes
/// - Allow partial / targeted re-run (e.g. certificates only)
///
/// All `detect_*` helpers are intentionally private; only `run_preflight`
/// constitutes the public API surface for now.
pub fn run_preflight() -> Vec<PreflightItem> {
    let mut items = Vec::new();
    let mut push = |label: &str, res: Result<bool>| match res {
        Ok(true) => items.push(PreflightItem {
            label: label.to_string(),
            status: PreflightStatus::Present,
            message: None,
        }),
        Ok(false) => items.push(PreflightItem {
            label: label.to_string(),
            status: PreflightStatus::Missing,
            message: None,
        }),
        Err(e) => items.push(PreflightItem {
            label: label.to_string(),
            status: PreflightStatus::Error,
            message: Some(e.to_string()),
        }),
    };

    push("Server settings", detect_server_settings());
    push("Server installation", detect_server_installation());
    push("Certificates", detect_certs());
    push("Server user group", detect_server_user_group());
    push("Server user", detect_server_user());
    push("UDS / IPC socket", detect_uds());

    items
}

// --- Detection helpers (placeholders) ---------------------------------------------------------
// NOTE: These are intentionally thin stubs. Replace with real logic once the
// discovery spec is defined. Keep them small & testable; push heavy IO or
// parsing into dedicated helper modules or services.

fn detect_server_settings() -> Result<bool> {
    // TODO: Inspect standard config paths or query settings gateway
    Ok(true)
}

fn detect_server_installation() -> Result<bool> {
    // TODO: Check for server binary / version / integrity
    Ok(true)
}

fn detect_certs() -> Result<bool> {
    // TODO: Validate presence of certificate chain or pending generation
    Ok(true)
}

fn detect_server_user_group() -> Result<bool> {
    // TODO: Check system group existence & permissions
    Ok(true)
}

fn detect_server_user() -> Result<bool> {
    // TODO: Check system user existence & membership in group
    Ok(true)
}

fn detect_uds() -> Result<bool> {
    // TODO: Verify control-plane socket path (exists? stale? permissions?)
    Ok(true)
}
