//! Aether (server) configuration using the new `settings` crate.
//!
//! Design goals:
//! - Strongly typed sections (each section is one struct that implements `Settings`).
//! - Defaults live in `Default` impls (no embedded TOML / layering system anymore).
//! - Single user delta file (RON) contains only deviations vs. defaults.
//! - Simple field update helper (`apply_server_setting`) for UI / CLI usage.
//! - Optional validation helpers (fail-fast; they do NOT auto‑rollback on failure).
//!
//! NOTE (Bevy Integration):
//! The existing `bevy` module expects the types `General`, `Network`, `Security`,
//! `Monitoring`, `UDS` to implement `Settings`, and Bevy resources alias the
//! corresponding `*Cfg` names. To stay compatible we implement the structs with
//! `*Cfg` suffix and expose type aliases without the suffix (`pub type General = General;` …).
//!
//! If you add new sections, repeat the pattern:
//!   1. Define `XxxCfg` struct + `Default`.
//!   2. `impl Settings for XxxCfg { const SECTION: &str = "section_name"; }`
//!   3. `pub type Xxx = XxxCfg;` (to keep Bevy/older code stable)
//!   4. Register it in `build_server_settings_store()` and (optionally) add a
//!      variant to `ServerSettingField` + logic to `apply_server_setting`.

#[cfg(feature = "bevy")]
pub mod bevy;

use serde::{Deserialize, Serialize};
use settings::{Settings, SettingsError, SettingsStore};

/* ------------------------------------------------------------------------- */
/* Section Models                                                            */
/* ------------------------------------------------------------------------- */

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct General {
    pub tick_rate: f64,
    pub autostart: bool,
}

impl Default for General {
    fn default() -> Self {
        Self {
            tick_rate: 60.0,
            autostart: true,
        }
    }
}
impl Settings for General {
    const SECTION: &'static str = "general";
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Network {
    pub ip_address: String,
    pub udp_port: u16,
    pub max_concurrent_bidi_streams: u32,
    pub max_concurrent_uni_streams: u32,
    pub max_idle_timeout: u64,    // seconds
    pub keep_alive_interval: u64, // seconds
    pub client_ip_migration: bool,
    pub zero_rtt_resumption: bool,
    pub initial_congestion_window: u32,
    pub mtu: u32,
    pub qos_traffic_prioritization: bool,
    pub nat_traversal: bool,
    pub uds_file: String,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            ip_address: "0.0.0.0".into(),
            udp_port: 7777,
            max_concurrent_bidi_streams: 512,
            max_concurrent_uni_streams: 256,
            max_idle_timeout: 300,
            keep_alive_interval: 30,
            client_ip_migration: true,
            zero_rtt_resumption: false,
            initial_congestion_window: 32,
            mtu: 1500,
            qos_traffic_prioritization: false,
            nat_traversal: true,
            uds_file: "aether.sock".into(),
        }
    }
}
impl Settings for Network {
    const SECTION: &'static str = "network";
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Security {
    pub cert_path: String,
    pub key_path: String,
    pub alpn: Vec<String>,
    pub handshake_timeout: u64, // seconds
    pub max_frame_bytes: u32,
    pub max_sessions: u32,
    pub self_signed: bool,
    pub tls_cert_algorithm: String,
    pub log_level: String,
    pub key_rotation_interval: u64, // seconds (interpretation adjustable later)
    pub client_auth: bool,
}

impl Default for Security {
    fn default() -> Self {
        Self {
            cert_path: String::new(),
            key_path: String::new(),
            alpn: vec!["h3".into()],
            handshake_timeout: 30,
            max_frame_bytes: 64 * 1024,
            max_sessions: 1000,
            self_signed: true,
            tls_cert_algorithm: "rsa".into(),
            log_level: "info".into(),
            key_rotation_interval: 3600,
            client_auth: false,
        }
    }
}
impl Settings for Security {
    const SECTION: &'static str = "security";
}

pub fn build_server_settings_store() -> Result<SettingsStore, SettingsError> {
    let store = SettingsStore::builder()
        .with_settings_file(paths::config_dir().join("aether.toml"))
        .build()?;
    store.register::<General>()?;
    store.register::<Network>()?;
    store.register::<Security>()?;
    Ok(store)
}

/// Validates the settings file by attempting to build and load the SettingsStore
/// Returns true if the settings file can be parsed and all registered sections are valid
/// Returns false if:
/// - TOML syntax errors exist
/// - Required sections are missing
/// - Field values are out of valid ranges
/// - Any other validation errors occur
pub fn settings_valid() -> bool {
    // Validation now happens automatically via Settings::validate() during store build
    match build_server_settings_store() {
        Ok(_store) => true,
        Err(_e) => false,
    }
}

/// Placeholder: was a server certificate (and optionally key) found?
/// For now we just check for a single PEM file in the config directory.
/// Adjust path / strategy once certificate management is implemented.
pub fn certificate_found() -> bool {
    paths::config_dir().join("cert.pem").exists() && paths::config_dir().join("key.pem").exists()
}

/// Placeholder: does a (planned) Unix Domain Socket file exist?
/// On non-unix platforms this always returns false.
pub fn uds_found() -> bool {
    #[cfg(unix)]
    {
        // TODO: replace with actual runtime dir / socket discovery logic.
        paths::config_dir().join("aether.sock").exists()
    }
    #[cfg(not(unix))]
    {
        false
    }
}
