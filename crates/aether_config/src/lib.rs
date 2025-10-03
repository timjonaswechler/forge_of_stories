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

use semver::Version;
use serde::{Deserialize, Serialize};
use settings::{Settings, SettingsError, SettingsStore};

#[cfg(feature = "bevy")]
use bevy::Resource;

/* ------------------------------------------------------------------------- */
/* Transport Mode                                                            */
/* ------------------------------------------------------------------------- */

/// Network transport mode selection.
///
/// # Example
///
/// In your settings file (`<app_id>.settings.json`):
/// ```json
/// {
///   "network": {
///     "transport_mode": "quic"  // or "steam"
///   }
/// }
/// ```
///
/// From code:
/// ```
/// use aether_config::TransportMode;
///
/// let mode = TransportMode::Quic; // or TransportMode::Steam
/// ```
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "bevy", derive(Resource))]
#[serde(rename_all = "lowercase")]
pub enum TransportMode {
    /// QUIC-based transport (LAN/WAN).
    Quic,
    /// Steam Networking (P2P via Steam Relay).
    Steam,
}

impl Default for TransportMode {
    fn default() -> Self {
        Self::Quic
    }
}

/* ------------------------------------------------------------------------- */
/* Section Models                                                            */
/* ------------------------------------------------------------------------- */

#[derive(Clone, Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "bevy", derive(Resource))]
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
#[cfg_attr(feature = "bevy", derive(Resource))]
pub struct Network {
    /// Transport mode: QUIC or Steam.
    pub transport_mode: TransportMode,
    // QUIC-specific settings (also used as fallback defaults)
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
            transport_mode: TransportMode::default(),
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
            // Migrate uds_file -> uds_path
            if let Some(old_value) = map.remove("uds_file") {
                map.insert("uds_path".to_string(), old_value);
            }
            if !map.contains_key("uds_path") {
                map.insert(
                    "uds_path".to_string(),
                    serde_json::Value::String(Network::default().uds_path),
                );
            }

            // Add transport_mode if missing
            if !map.contains_key("transport_mode") {
                map.insert(
                    "transport_mode".to_string(),
                    serde_json::to_value(TransportMode::default()).unwrap(),
                );
            }

            return Ok((serde_json::Value::Object(map), true));
        }

        Ok((serde_json::Value::Object(map), false))
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "bevy", derive(Resource))]
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

/// Steam-specific networking settings.
///
/// # Example
///
/// In your settings file:
/// ```json
/// {
///   "steam": {
///     "lobby_name": "My Custom Server",
///     "max_players": 32,
///     "lobby_joinable": true,
///     "use_sdr": true,
///     "p2p_timeout": 60
///   }
/// }
/// ```
#[derive(Clone, Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "bevy", derive(Resource))]
pub struct Steam {
    /// Lobby name visible in Steam matchmaking.
    pub lobby_name: String,
    /// Maximum players allowed in the lobby.
    pub max_players: u32,
    /// Whether the lobby should be visible/joinable.
    pub lobby_joinable: bool,
    /// Use Steam Datagram Relay (SDR) for improved routing.
    pub use_sdr: bool,
    /// P2P session timeout in seconds.
    pub p2p_timeout: u64,
}

impl Default for Steam {
    fn default() -> Self {
        Self {
            lobby_name: "Forge of Stories Server".into(), // todo: Save name or set from user input
            max_players: 16,
            lobby_joinable: true,
            use_sdr: true,
            p2p_timeout: 30,
        }
    }
}
impl Settings for Steam {
    const SECTION: &'static str = "steam";
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
    store.register::<Steam>()?;
    Ok(store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use app::{AppBase, Application};
    use color_eyre::Result;
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
            #[allow(dead_code)]
            pub base: AppBase,
        }

        let base =
            app::init::<MyAppApp>(env!("CARGO_PKG_VERSION")).expect("Inizialisation went wrong");
        let settings_file = base.path_context.settings_file(None);

        // Ensure parent directory exists
        if let Some(parent) = settings_file.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write old-format settings with uds_file (pre-migration) and version 0.0.1
        fs::write(
            &settings_file,
            r#"{
          "__meta": {
            "version": "0.0.1"
          },
          "network": {
            "uds_file": "legacy.sock"
          }
        }
        "#,
        )?;

        // Build store - this triggers migration
        let store = build_server_settings_store(&settings_file, "0.1.0")?;

        // Verify migrated config loaded correctly
        let cfg = store.get::<Network>()?;
        assert_eq!(
            cfg.uds_path, "legacy.sock",
            "uds_file should be migrated to uds_path"
        );
        assert_eq!(
            cfg.transport_mode,
            TransportMode::Quic,
            "transport_mode should default to Quic"
        );

        // Note: The settings file may be deleted if all values match defaults after migration.
        // This is expected behavior - we only care that the config was loaded correctly.

        Ok(())
    }

    #[test]
    fn test_transport_mode_serialization() -> Result<(), Box<dyn std::error::Error>> {
        use serde_json;

        // Test QUIC mode
        let quic_json = serde_json::to_string(&TransportMode::Quic)?;
        assert_eq!(quic_json, r#""quic""#);
        let quic_parsed: TransportMode = serde_json::from_str(&quic_json)?;
        assert_eq!(quic_parsed, TransportMode::Quic);

        // Test Steam mode
        let steam_json = serde_json::to_string(&TransportMode::Steam)?;
        assert_eq!(steam_json, r#""steam""#);
        let steam_parsed: TransportMode = serde_json::from_str(&steam_json)?;
        assert_eq!(steam_parsed, TransportMode::Steam);

        Ok(())
    }

    #[test]
    fn test_steam_settings_defaults() {
        let steam = Steam::default();
        assert_eq!(steam.lobby_name, "Forge of Stories Server");
        assert_eq!(steam.max_players, 16);
        assert!(steam.lobby_joinable);
        assert!(steam.use_sdr);
        assert_eq!(steam.p2p_timeout, 30);
    }
}
