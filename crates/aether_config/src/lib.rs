#[cfg(feature = "bevy")]
pub mod bevy;

use serde::{Deserialize, Serialize};

fn parse_bool(s: &str) -> color_eyre::Result<bool> {
    let v = s.trim().to_ascii_lowercase();
    match v.as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(color_eyre::eyre::eyre!(format!(
            "Ungültiger bool-Wert: {s}"
        ))),
    }
}

fn default_value_for(field: ServerSettingField) -> Option<toml::Value> {
    let txt = settings::default_settings_server();
    let Ok(root) = toml::from_str::<toml::Value>(txt.as_ref()) else {
        return None;
    };
    let tbl = root.as_table()?;

    let path = match field {
        ServerSettingField::GeneralTickRate => &["general", "tick_rate"][..],
        ServerSettingField::GeneralAutostart => &["general", "autostart"][..],
        ServerSettingField::NetworkIpAddress => &["network", "ip_address"][..],
        ServerSettingField::NetworkUdpPort => &["network", "udp_port"][..],
        ServerSettingField::SecurityTlsCertPath => &["security", "tls_cert_path"][..],
        ServerSettingField::MonitoringMetricsEnabled => &["monitoring", "metrics_enabled"][..],
        ServerSettingField::UdsPath => &["uds", "path"][..],
    };

    let mut cur = toml::Value::Table(tbl.clone());
    for seg in path {
        match cur {
            toml::Value::Table(ref t) => {
                let Some(v) = t.get(*seg) else {
                    return None;
                };
                cur = v.clone();
            }
            _ => return None,
        }
    }
    Some(cur)
}

/// Wendet eine einzelne Feldänderung an. Leerer String => Feld wird auf den eingebetteten Default zurückgesetzt (falls vorhanden).
pub fn apply_server_setting(
    store: &settings::SettingsStore,
    field: ServerSettingField,
    raw_value: &str,
) -> color_eyre::Result<()> {
    let s = raw_value.trim();

    match field {
        ServerSettingField::GeneralTickRate => {
            let v: f64 = if s.is_empty() {
                default_value_for(field)
                    .and_then(|v| v.as_integer().map(|n| n as f64))
                    .unwrap_or_default()
            } else {
                s.parse()?
            };
            store.update::<General>(|m| m.tick_rate = v)?;
        }
        ServerSettingField::GeneralAutostart => {
            let v: bool = if s.is_empty() {
                default_value_for(field)
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            } else {
                parse_bool(s)?
            };
            store.update::<General>(|m| m.autostart = v)?;
        }
        ServerSettingField::NetworkIpAddress => {
            let v: String = if s.is_empty() {
                default_value_for(field)
                    .and_then(|v| v.as_str().map(|x| x.to_string()))
                    .unwrap_or_default()
            } else {
                s.to_string()
            };
            store.update::<Network>(|m| m.ip_address = v)?;
        }
        ServerSettingField::NetworkUdpPort => {
            let v: u16 = if s.is_empty() {
                default_value_for(field)
                    .and_then(|v| v.as_integer().map(|n| n as u16))
                    .unwrap_or_default()
            } else {
                s.parse()?
            };
            store.update::<Network>(|m| m.udp_port = v)?;
        }
        ServerSettingField::SecurityTlsCertPath => {
            let v: String = if s.is_empty() {
                default_value_for(field)
                    .and_then(|v| v.as_str().map(|x| x.to_string()))
                    .unwrap_or_default()
            } else {
                s.to_string()
            };
            store.update::<Security>(|m| m.cert_path = v)?;
        }
        ServerSettingField::MonitoringMetricsEnabled => {
            let v: bool = if s.is_empty() {
                default_value_for(field)
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            } else {
                parse_bool(s)?
            };
            store.update::<Monitoring>(|m| m.metrics_enabled = v)?;
        }
        ServerSettingField::UdsPath => {
            let v: String = if s.is_empty() {
                default_value_for(field)
                    .and_then(|v| v.as_str().map(|x| x.to_string()))
                    .unwrap_or_default()
            } else {
                s.to_string()
            };
            store.update::<Uds>(|m| m.path = v)?;
        }
    }

    Ok(())
}
use settings::{Settings, SettingsStore};
use std::{sync::Arc, thread::panicking};

// 1) Typisierte Modelle
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct GeneralCfg {
    pub tick_rate: f64,
    pub autostart: bool,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct NetworkCfg {
    pub ip_address: String,
    pub udp_port: u16,
    pub max_concurrent_bidi_streams: u32,
    // Extended fields (loaded from [network] in aether-default.toml)
    pub max_concurrent_uni_streams: u32,
    pub max_idle_timeout: u64,    // seconds
    pub keep_alive_interval: u64, // seconds
    pub client_ip_migration: bool,
    pub zero_rtt_resumption: bool,
    pub initial_congestion_window: u32,
    pub mtu: u32,
    pub qos_traffic_prioritization: bool,
    pub nat_traversal: bool,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct SecurityCfg {
    pub cert_path: String,
    pub key_path: String,
    pub alpn: Vec<String>,
    // Extended fields (from [security] in aether-default.toml)
    pub handshake_timeout: u64, // seconds
    pub max_frame_bytes: u32,
    pub max_sessions: u32,
    pub self_signed: bool,
    pub tls_cert_algorithm: String,
    pub log_level: String,
    pub key_rotation_interval: u64, // minutes or seconds (clarify later)
    pub client_auth: bool,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct MonitoringCfg {
    pub metrics_enabled: bool,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct UdsCfg {
    pub path: String, /* ... */
}

// 2) SECTION-Bindings
pub struct General;
impl Settings for General {
    const SECTION: &'static str = "general";
    type Model = GeneralCfg;

    fn validate(model: &Self::Model) -> Result<(), settings::SettingsError> {
        if model.tick_rate <= 0.0 {
            return Err(settings::SettingsError::Invalid(
                "tick_rate must be greater than 0",
            ));
        }
        if model.tick_rate > 1000.0 {
            return Err(settings::SettingsError::Invalid(
                "tick_rate too high (> 1000.0)",
            ));
        }
        Ok(())
    }
}

pub struct Network;
impl Settings for Network {
    const SECTION: &'static str = "network";
    type Model = NetworkCfg;

    fn validate(model: &Self::Model) -> Result<(), settings::SettingsError> {
        // Validate port range
        if model.udp_port <= 1024 || model.udp_port > 49152 {
            return Err(settings::SettingsError::Invalid(
                "udp_port must be between 1024 and 49152",
            ));
        }

        // Validate IP address format
        if !model.ip_address.is_empty() {
            use std::net::IpAddr;
            if model.ip_address.parse::<IpAddr>().is_err() && model.ip_address != "0.0.0.0" {
                return Err(settings::SettingsError::Invalid(
                    "invalid IP address format",
                ));
            }
        }

        // Validate reasonable value ranges
        if model.max_concurrent_bidi_streams > 10000 {
            return Err(settings::SettingsError::Invalid(
                "max_concurrent_bidi_streams too high (> 10000)",
            ));
        }

        if model.mtu > 0 && model.mtu < 256 {
            return Err(settings::SettingsError::Invalid("mtu too small (< 256)"));
        }

        if model.mtu > 65535 {
            return Err(settings::SettingsError::Invalid("mtu too large (> 65535)"));
        }

        // Validate timeout values
        if model.max_idle_timeout > 3600 {
            return Err(settings::SettingsError::Invalid(
                "max_idle_timeout too high (> 3600 seconds)",
            ));
        }

        if model.keep_alive_interval > model.max_idle_timeout / 2 {
            return Err(settings::SettingsError::Invalid(
                "keep_alive_interval should be less than half of max_idle_timeout",
            ));
        }

        Ok(())
    }
}

pub struct Monitoring;
impl Settings for Monitoring {
    const SECTION: &'static str = "monitoring";
    type Model = MonitoringCfg;
}

pub struct Security;
impl Settings for Security {
    const SECTION: &'static str = "security";
    type Model = SecurityCfg;

    fn validate(model: &Self::Model) -> Result<(), settings::SettingsError> {
        // Validate cert_path exists if specified and not empty
        if !model.self_signed && !model.cert_path.is_empty() {
            let cert_path = std::path::Path::new(&model.cert_path);
            if !cert_path.exists() {
                return Err(settings::SettingsError::Invalid(
                    "certificate file not found",
                ));
            }
            if !cert_path.is_file() {
                return Err(settings::SettingsError::Invalid(
                    "certificate path is not a file",
                ));
            }
        }

        // Validate key_path exists if specified and not empty
        if !model.self_signed && !model.key_path.is_empty() {
            let key_path = std::path::Path::new(&model.key_path);
            if !key_path.exists() {
                return Err(settings::SettingsError::Invalid("key file not found"));
            }
            if !key_path.is_file() {
                return Err(settings::SettingsError::Invalid("key path is not a file"));
            }
        }

        // Validate timeout values are reasonable
        if model.handshake_timeout == 0 {
            return Err(settings::SettingsError::Invalid(
                "handshake_timeout cannot be 0",
            ));
        }

        if model.handshake_timeout > 300 {
            return Err(settings::SettingsError::Invalid(
                "handshake_timeout too high (> 300 seconds)",
            ));
        }

        // Validate algorithm is supported
        if !model.tls_cert_algorithm.is_empty() {
            match model.tls_cert_algorithm.to_lowercase().as_str() {
                "rsa" | "ecdsa" | "ed25519" => {}
                _ => {
                    return Err(settings::SettingsError::Invalid(
                        "unsupported tls_cert_algorithm (supported: rsa, ecdsa, ed25519)",
                    ));
                }
            }
        }

        // Validate log level
        if !model.log_level.is_empty() {
            match model.log_level.to_lowercase().as_str() {
                "error" | "warn" | "info" | "debug" | "trace" => {}
                _ => {
                    return Err(settings::SettingsError::Invalid(
                        "invalid log_level (supported: error, warn, info, debug, trace)",
                    ));
                }
            }
        }

        // Validate max_frame_bytes
        if model.max_frame_bytes > 0 && model.max_frame_bytes < 1024 {
            return Err(settings::SettingsError::Invalid(
                "max_frame_bytes too small (< 1024)",
            ));
        }

        if model.max_frame_bytes > 16_777_216 {
            // 16MB
            return Err(settings::SettingsError::Invalid(
                "max_frame_bytes too large (> 16MB)",
            ));
        }

        Ok(())
    }
}

pub struct Uds;
impl Settings for Uds {
    const SECTION: &'static str = "uds";
    type Model = UdsCfg;
}

pub struct ServerSettings {
    pub general: Arc<GeneralCfg>,
    pub network: Arc<NetworkCfg>,
    pub security: Arc<SecurityCfg>,
    pub monitoring: Arc<MonitoringCfg>,
    pub uds: Arc<UdsCfg>,
}

pub const ENV_LAYERS_VAR: &str = "FOS_SERVER_ENV_LAYERS";
pub const ENV_PREFIX: &str = "FOS_SERVER";

// 3) Zentraler Builder: überall gleich aufrufbar (Wizard & Runtime)
pub fn build_server_settings_store() -> color_eyre::Result<SettingsStore> {
    let mut builder = SettingsStore::builder()
        .with_embedded_setting_asset("settings/aether-default.toml")
        .with_settings_file_optional(paths::config_dir().join("aether.toml"));

    let env_layers_enabled = std::env::var(ENV_LAYERS_VAR)
        .ok()
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(true);

    builder = builder.enable_env_layers(env_layers_enabled);
    if env_layers_enabled {
        builder = builder.with_env_prefix(ENV_PREFIX);
    }
    let store = builder.build()?;
    store.register::<General>()?;
    store.register::<Network>()?;
    store.register::<Security>()?;
    store.register::<Monitoring>()?;
    store.register::<Uds>()?;
    Ok(store)
}

pub enum ServerSettingField {
    GeneralTickRate,
    GeneralAutostart,
    NetworkIpAddress,
    NetworkUdpPort,
    SecurityTlsCertPath,
    MonitoringMetricsEnabled,
    UdsPath,
}

/// Placeholder: does the settings file exist?
/// Alias kept for clearer intent at call sites inside UI code.
pub fn settings_found() -> bool {
    if paths::config_dir().join("aether.toml").exists() {
        true
    } else {
        false
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_valid_with_defaults() {
        // Should be valid with default embedded settings
        match build_server_settings_store() {
            Ok(_) => println!("Settings store built successfully"),
            Err(e) => {
                println!("Settings store failed: {:?}", e);
                panic!("Settings validation failed: {:?}", e);
            }
        }
        assert!(settings_valid());
    }

    #[test]
    fn test_general_validation() {
        use settings::Settings;

        // Valid case
        let valid = GeneralCfg {
            tick_rate: 60.0,
            autostart: true,
        };
        assert!(General::validate(&valid).is_ok());

        // Invalid case - negative tick rate
        let invalid = GeneralCfg {
            tick_rate: -1.0,
            autostart: true,
        };
        assert!(General::validate(&invalid).is_err());

        // Invalid case - too high tick rate
        let invalid2 = GeneralCfg {
            tick_rate: 2000.0,
            autostart: true,
        };
        assert!(General::validate(&invalid2).is_err());
    }

    #[test]
    fn test_network_validation() {
        use settings::Settings;

        // Valid case
        let valid = NetworkCfg {
            ip_address: "127.0.0.1".to_string(),
            udp_port: 8080,
            max_concurrent_bidi_streams: 100,
            max_concurrent_uni_streams: 200,
            max_idle_timeout: 300,
            keep_alive_interval: 30,
            client_ip_migration: true,
            zero_rtt_resumption: false,
            initial_congestion_window: 32,
            mtu: 1500,
            qos_traffic_prioritization: false,
            nat_traversal: true,
        };
        assert!(Network::validate(&valid).is_ok());

        // Invalid case - port 0
        let mut invalid = valid.clone();
        invalid.udp_port = 0;
        assert!(Network::validate(&invalid).is_err());

        // Invalid case - invalid IP
        let mut invalid2 = valid.clone();
        invalid2.ip_address = "not.an.ip".to_string();
        assert!(Network::validate(&invalid2).is_err());

        // Invalid case - MTU too small
        let mut invalid3 = valid.clone();
        invalid3.mtu = 100;
        assert!(Network::validate(&invalid3).is_err());
    }

    #[test]
    fn test_security_validation() {
        use settings::Settings;

        // Valid case - empty paths (optional)
        let valid = SecurityCfg {
            cert_path: String::new(),
            key_path: String::new(),
            alpn: vec!["h3".to_string()],
            handshake_timeout: 30,
            max_frame_bytes: 65536,
            max_sessions: 1000,
            self_signed: false,
            tls_cert_algorithm: "rsa".to_string(),
            log_level: "info".to_string(),
            key_rotation_interval: 3600,
            client_auth: false,
        };
        assert!(Security::validate(&valid).is_ok());

        // Invalid case - timeout 0
        let mut invalid = valid.clone();
        invalid.handshake_timeout = 0;
        assert!(Security::validate(&invalid).is_err());

        // Invalid case - unsupported algorithm
        let mut invalid2 = valid.clone();
        invalid2.tls_cert_algorithm = "unsupported".to_string();
        assert!(Security::validate(&invalid2).is_err());

        // Invalid case - invalid log level
        let mut invalid3 = valid.clone();
        invalid3.log_level = "invalid".to_string();
        assert!(Security::validate(&invalid3).is_err());
    }
}
