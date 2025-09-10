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
use color_eyre::eyre::Result;
use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType};
use time::{Duration, OffsetDateTime};

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
    /// Optional output path hint for persistence (directory or file).
    pub output_path: Option<String>,
}

impl Default for SelfSignedParams {
    fn default() -> Self {
        Self {
            common_name: "forge-of-stories.local".into(),
            dns_names: vec!["localhost".into()],
            valid_days: 365,
            key_bits: 2048,
            output_path: None,
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

/// Result wrapper for certificate generation tasks (to be used by the
/// async task executor when mapping task completion back into internal events).
#[derive(Debug, Clone)]
pub enum CertTaskResult {
    Success { artifacts: GeneratedCertArtifacts },
    Error { message: String },
}

/// Generate a self‑signed certificate using `rcgen`.
///
/// Algorithm selection heuristic:
/// - If `key_bits <= 256` → ECDSA P-256
/// - Else → RSA 2048 (rcgen default RSA with SHA256)
///
/// Notes:
/// - For now we only treat DNS names; IP SANs can be added later.
/// - Private key is returned unencrypted (caller decides on persistence).
pub fn generate_self_signed(params: &SelfSignedParams) -> Result<GeneratedCertArtifacts> {
    // Build certificate params with SAN DNS names (rcgen uses the vector of subject alt names).
    let mut cp = CertificateParams::new(params.dns_names.clone());

    // Subject / Distinguished Name
    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, params.common_name.clone());
    cp.distinguished_name = dn;

    // Validity window
    let now = OffsetDateTime::now_utc();
    cp.not_before = now - Duration::minutes(5);
    cp.not_after = now + Duration::days(params.valid_days as i64);

    // Algorithm heuristic
    if params.key_bits <= 256 {
        cp.alg = &rcgen::PKCS_ECDSA_P256_SHA256;
    } else {
        cp.alg = &rcgen::PKCS_RSA_SHA256;
    }

    // Generate certificate + key
    let cert = Certificate::from_params(cp)?;
    let cert_pem = cert.serialize_pem()?;
    let key_pem = cert.serialize_private_key_pem();

    Ok(GeneratedCertArtifacts {
        cert_pem,
        key_pem,
        chain_pem: None,
    })
}

/// Convenience wrapper producing a task-style result enum (for future executor integration).
pub fn generate_self_signed_task(params: &SelfSignedParams) -> CertTaskResult {
    match generate_self_signed(params) {
        Ok(artifacts) => CertTaskResult::Success { artifacts },
        Err(e) => CertTaskResult::Error {
            message: e.to_string(),
        },
    }
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
    fn generates_valid_pem() {
        let params = SelfSignedParams::default();
        let res = generate_self_signed(&params).expect("generation failed");
        assert!(res.cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(res.key_pem.contains("BEGIN"));
    }

    #[test]
    fn task_result_success() {
        let params = SelfSignedParams::default();
        match generate_self_signed_task(&params) {
            CertTaskResult::Success { artifacts } => {
                assert!(artifacts.cert_pem.contains("CERTIFICATE"));
            }
            CertTaskResult::Error { message } => panic!("expected success, got error: {message}"),
        }
    }
}
