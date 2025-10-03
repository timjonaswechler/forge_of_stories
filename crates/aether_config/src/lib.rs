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

use bevy::Resource;
use semver::Version;
use serde::{Deserialize, Serialize};
use settings::{Settings, SettingsError, SettingsStore};

/* ------------------------------------------------------------------------- */
/* Section Models                                                            */
/* ------------------------------------------------------------------------- */

#[derive(Clone, Serialize, Deserialize, Debug, Resource)]
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

#[derive(Clone, Serialize, Deserialize, Debug, Resource)]
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
    pub uds_path: String,
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
            uds_path: "aether.sock".into(),
        }
    }
}
impl Settings for Network {
    const SECTION: &'static str = "network";

    fn migrate(
        file_version: Option<&Version>,
        target_version: &Version,
        data: serde_json::Value,
    ) -> Result<(serde_json::Value, bool), SettingsError> {
        let mut map = match data {
            serde_json::Value::Object(map) => map,
            _ => return Err(SettingsError::Invalid("network settings not an object")),
        };

        let needs_upgrade = file_version.map(|ver| ver < target_version).unwrap_or(true);

        if needs_upgrade {
            if let Some(old_value) = map.remove("uds_file") {
                map.insert("uds_path".to_string(), old_value);
            }
            if !map.contains_key("uds_path") {
                map.insert(
                    "uds_path".to_string(),
                    serde_json::Value::String(Network::default().uds_path),
                );
            }
            return Ok((serde_json::Value::Object(map), true));
        }

        Ok((serde_json::Value::Object(map), false))
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Resource)]
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

/// Build the server settings store inside an explicit application config directory:
/// <config_root>/settings.json  (RON format; preferred new location).
pub fn build_server_settings_store<P: Into<std::path::PathBuf>>(
    settings_file: P,
    version: &'static str,
) -> Result<SettingsStore, SettingsError> {
    let store = SettingsStore::builder(version)
        .with_settings_file(settings_file)
        .build()?;
    store.register::<General>()?;
    store.register::<Network>()?;
    store.register::<Security>()?;
    Ok(store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use app::{AppBase, Application};
    use color_eyre::Result;
    use serde_json::Value as JsonValue;
    use std::fs;

    #[test]
    fn migrates_network_uds_field() -> Result<(), Box<dyn std::error::Error>> {
        impl Application for MyAppApp {
            type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
            const APP_ID: &'static str = "MyApp";
            fn init_platform() -> Result<(), Self::Error> {
                Ok(())
            }
        }

        pub struct MyAppApp {
            pub base: AppBase,
        }

        impl MyAppApp {
            pub fn init(base: AppBase) -> Result<Self> {
                Ok(Self { base: base })
            }
        }

        let base =
            app::init::<MyAppApp>(env!("CARGO_PKG_VERSION")).expect("Inizialisation went wrong");
        base.path_context.settings_file(None);

        fs::write(
            &base.path_context.settings_file(None),
            r#"{
          "network": {
            "uds_file": "legacy.sock"
          }
        }
        "#,
        )?;

        let store = build_server_settings_store(&base.path_context.settings_file(None), "0.1.0")?;
        let expected_version = store.schema_version().to_string();

        let doc: JsonValue = serde_json::from_str(&fs::read_to_string(
            &&base.path_context.settings_file(None),
        )?)?;
        dbg!(&doc);

        let cfg = store.get::<Network>()?;
        dbg!(&cfg.uds_path);
        assert_eq!(cfg.uds_path, "legacy.sock");

        assert_eq!(
            doc["__meta"]["version"].as_str(),
            Some(expected_version.as_str()),
        );
        assert!(doc["network"].get("uds_file").is_none());
        assert_eq!(doc["network"]["uds_path"].as_str(), Some("legacy.sock"),);

        Ok(())
    }
}
