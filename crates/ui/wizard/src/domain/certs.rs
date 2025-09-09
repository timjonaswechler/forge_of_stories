//! Domain: Certificates management placeholder (Phase 8.3)
//
//! This module introduces a forward‑looking placeholder API for
//! generating self‑signed certificates that will later be used
//! during setup / security related workflows (e.g. certificate
//! wizard popup, automated bootstrap).
//!
//! Scope (now):
//! - Provide data structures & a `generate_self_signed` stub
//! - Allow UI / gateway code to compile against the future API
//! - Document intended behavior & extensibility points
//!
//! Non‑scope (future work):
//! - Real cryptographic implementation (likely via `rcgen`,
//!   `openssl`, or a custom PKI helper crate living outside
//!   the TUI)
//! - Persistence (writing to disk, rotation policies)
//! - Trust store integration
//! - ACME / Let's Encrypt flow
//!
//! Design Notes:
//! - The function returns in‑memory PEM strings so callers
//!   can decide where & how to persist them.
//! - Parameters kept minimal; can expand (KU/EKU, SAN IPs,
//!   algorithms, curve selection, etc.)
//!
//! Once implemented, this module should move out any heavy
//! crypto dependencies from the core UI build path unless
//! already transitively present.
use color_eyre::eyre::{Result, eyre};

/// Input parameters for generating a self‑signed certificate.
#[derive(Debug, Clone)]
pub struct SelfSignedParams {
    /// Common Name (CN) for the certificate subject.
    pub common_name: String,
    /// Subject Alternative Names (DNS names).
    pub dns_names: Vec<String>,
    /// Validity period in days.
    pub valid_days: u32,
    /// RSA key size in bits (when RSA is selected). Placeholder for future alg selection.
    pub key_bits: u16,
}

impl Default for SelfSignedParams {
    fn default() -> Self {
        Self {
            common_name: "forge-of-stories.local".into(),
            dns_names: vec!["localhost".into()],
            valid_days: 365,
            key_bits: 2048,
        }
    }
}

/// Output artifacts of a self‑signed certificate generation.
#[derive(Debug, Clone)]
pub struct GeneratedCertArtifacts {
    /// PEM encoded X.509 certificate
    pub cert_pem: String,
    /// PEM encoded private key (unencrypted)
    pub key_pem: String,
    /// Optional chain (unused for pure self‑signed, reserved for future)
    pub chain_pem: Option<String>,
}

/// Placeholder implementation.
///
/// TODO:
/// - Implement using a crypto crate (rcgen or openssl)
/// - Parameterize signature algorithm (RSA / ECDSA)
/// - Add support for IP SANs
/// - Optional encrypted key output
/// - Deterministic mode for tests (fixed serial / notBefore / notAfter)
pub fn generate_self_signed(_params: &SelfSignedParams) -> Result<GeneratedCertArtifacts> {
    Err(eyre!(
        "generate_self_signed() not implemented (Phase 8.3 placeholder)"
    ))
}

/// Convenience helper for quick prototyping; will be removed or
/// replaced by gateway logic once integrated in a setup workflow.
pub fn demo_insecure_dev_cert() -> Result<GeneratedCertArtifacts> {
    let params = SelfSignedParams::default();
    generate_self_signed(&params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_errors() {
        let params = SelfSignedParams::default();
        let res = generate_self_signed(&params);
        assert!(res.is_err(), "placeholder should error until implemented");
    }
}
