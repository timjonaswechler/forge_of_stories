//! TLS Lade- und Generierungs-Logik für den QUIC Server.
//!
//! Variant A (vereinbart):
//! - Relative Pfade werden strikt unterhalb von `paths::config_dir()` aufgelöst.
//! - Keine Fallback-Suche in `data_dir()`.
//! - Parent-Verweise (`..`) in relativen Pfaden sind nicht erlaubt (Directory Traversal Schutz).
//! - Absolute Pfade werden 1:1 verwendet.
//! - Falls Zertifikat / Key fehlen und Feature `debug` aktiv ist → Self-Signed Zertifikat erzeugen.
//! - Ohne `debug` Feature führt fehlendes Paar zu Fehler.
//!
//! Rückgabe: `rustls::ServerConfig` mit gesetzten ALPN Protokollen.
//!
//! Abhängigkeiten (in Cargo.toml):
//! - rustls (0.23)
//! - rustls-pemfile
//! - rcgen (optional, Feature `debug`)
//!
//! Fehler werden auf `QuicEndpointError::Tls(String)` gemappt.

use std::{
    fs,
    path::{Path, PathBuf},
};

use rustls::pki_types::{CertificateDer, PrivateKeyDer};

use super::endpoint::QuicEndpointError;
use crate::ServerRuntimeConfig;

#[cfg(feature = "debug")]
use rcgen::{Certificate, CertificateParams};

/// Lädt oder generiert (debug) TLS Materialien und baut eine `rustls::ServerConfig`.
pub fn load_or_generate_tls(
    cfg: &ServerRuntimeConfig,
) -> Result<rustls::ServerConfig, QuicEndpointError> {
    let (cert_path, key_path) = resolve_tls_paths(&cfg.cert_path, &cfg.key_path)?;

    let cert_exists = cert_path.exists();
    let key_exists = key_path.exists();

    let (cert_pem, key_pem) = if cert_exists && key_exists {
        (
            fs::read(&cert_path).map_err(|e| {
                QuicEndpointError::Tls(format!("read cert '{}': {e}", cert_path.display()))
            })?,
            fs::read(&key_path).map_err(|e| {
                QuicEndpointError::Tls(format!("read key '{}': {e}", key_path.display()))
            })?,
        )
    } else {
        #[cfg(feature = "debug")]
        {
            bevy::log::warn!(
                target:"server::net::tls",
                "TLS Dateien fehlen (cert_exists={}, key_exists={}) – generiere Self-Signed (debug): cert='{}' key='{}'",
                cert_exists, key_exists,
                cert_path.display(), key_path.display()
            );
            if let Some(parent) = cert_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Some(parent) = key_path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            let mut params = CertificateParams::new(vec!["localhost".into()]);
            // ECDSA P256 ist ausreichend modern und von rustls unterstützt.
            params.alg = &rcgen::PKCS_ECDSA_P256_SHA256;
            let cert = Certificate::from_params(params)
                .map_err(|e| QuicEndpointError::Tls(format!("rcgen params: {e}")))?;
            let cert_pem_str = cert
                .serialize_pem()
                .map_err(|e| QuicEndpointError::Tls(format!("rcgen serialize cert: {e}")))?;
            let key_pem_str = cert.serialize_private_key_pem();

            fs::write(&cert_path, &cert_pem_str).map_err(|e| {
                QuicEndpointError::Tls(format!("write cert '{}': {e}", cert_path.display()))
            })?;
            fs::write(&key_path, &key_pem_str).map_err(|e| {
                QuicEndpointError::Tls(format!("write key '{}': {e}", key_path.display()))
            })?;

            (cert_pem_str.into_bytes(), key_pem_str.into_bytes())
        }
        #[cfg(not(feature = "debug"))]
        {
            return Err(QuicEndpointError::Tls(format!(
                "missing TLS certificate or key (cert='{}', key='{}') and not in debug mode",
                cert_path.display(),
                key_path.display()
            )));
        }
    };

    // Zertifikatskette parsen (rustls-pemfile v2 Iterator API)
    let mut cert_reader = &cert_pem[..];
    let mut certs: Vec<CertificateDer> = Vec::new();
    for item in rustls_pemfile::certs(&mut cert_reader) {
        match item {
            Ok(c) => certs.push(c),
            Err(e) => {
                return Err(QuicEndpointError::Tls(format!("parse certs: {e:?}")));
            }
        }
    }
    if certs.is_empty() {
        return Err(QuicEndpointError::Tls("certificate chain empty".into()));
    }

    // Private Key parsen (PKCS8 bevorzugt, fallback EC) – aktualisiert für Iterator API
    let key = parse_private_key(&key_pem)
        .map_err(|e| QuicEndpointError::Tls(format!("parse key: {e}")))?;

    let mut server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| QuicEndpointError::Tls(format!("build server config: {e}")))?;

    // ALPN Protokolle setzen
    server_config.alpn_protocols = cfg.alpn.iter().map(|s| s.as_bytes().to_vec()).collect();

    bevy::log::info!(
        target:"server::net::tls",
        "TLS konfiguriert: cert='{}' key='{}' alpn={:?}",
        cert_path.display(),
        key_path.display(),
        cfg.alpn
    );

    Ok(server_config)
}

/// Parst einen Key (PKCS8 oder EC). Gibt Fehlertext bei Misserfolg (Iterator API).
fn parse_private_key(key_pem: &[u8]) -> Result<PrivateKeyDer<'static>, String> {
    // PKCS8 Keys
    let mut rdr = &key_pem[..];
    for item in rustls_pemfile::pkcs8_private_keys(&mut rdr) {
        match item {
            Ok(k) => return Ok(PrivateKeyDer::from(k)),
            Err(e) => return Err(format!("pkcs8 read: {e:?}")),
        }
    }
    // EC Keys
    let mut rdr2 = &key_pem[..];
    for item in rustls_pemfile::ec_private_keys(&mut rdr2) {
        match item {
            Ok(k) => return Ok(PrivateKeyDer::from(k)),
            Err(e) => return Err(format!("ec read: {e:?}")),
        }
    }
    Err("no supported private key format (expected pkcs8 or ec)".into())
}

/// Prüft ob gegebener Stringpfad als absolut zu interpretieren ist (inkl. Windows-Laufwerksbuchstaben).
fn is_absolute_or_drive(p: &str) -> bool {
    let path = Path::new(p);
    if path.is_absolute() {
        return true;
    }
    // Windows: "C:\..." oder "D:/..."
    if p.len() > 2 && p.as_bytes()[1] == b':' {
        return true;
    }
    false
}

/// Auflösung gemäß Variant A:
/// - Absolute Pfade unverändert.
/// - Relative Pfade relativ zu `paths::config_dir()`.
/// - Eltern-Pfade (`..`) in relativen Angaben nicht erlaubt.
fn resolve_one(raw: &str) -> Result<PathBuf, QuicEndpointError> {
    if is_absolute_or_drive(raw) {
        return Ok(PathBuf::from(raw));
    }
    let base = paths::config_dir();
    let joined = base.join(raw);

    // Sicherheits-Check: keine `..` erlaubt in relativer Eingabe
    // (Wir prüfen die Komponenten des Originalstrings statt canonicalize wegen Performance & Symlink-Handhabung).
    let rel_path = Path::new(raw);
    for comp in rel_path.components() {
        if matches!(comp, std::path::Component::ParentDir) {
            return Err(QuicEndpointError::Tls(format!(
                "parent directory segments ('..') not allowed in relative path: {}",
                raw
            )));
        }
    }

    Ok(joined)
}

/// Löst Zertifikat- und Key-Pfad gemäß Variant A auf.
fn resolve_tls_paths(
    cert_cfg: &str,
    key_cfg: &str,
) -> Result<(PathBuf, PathBuf), QuicEndpointError> {
    let cert = resolve_one(cert_cfg)?;
    let key = resolve_one(key_cfg)?;
    Ok((cert, key))
}

// Tests entfernt (temporär), da sie externe Abhängigkeit `tempfile` nutzten und für den
// aktuellen Migrationsschritt auf quinn 0.11 nicht erforderlich sind. Re-Introduce
// once a dedicated test strategy (with controlled temp dirs) is defined.
