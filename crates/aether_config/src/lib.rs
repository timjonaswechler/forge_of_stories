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
use std::sync::Arc;

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
}

pub struct Network;
impl Settings for Network {
    const SECTION: &'static str = "network";
    type Model = NetworkCfg;
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

pub fn find_setting() -> bool {
    if paths::config_dir().join("aether.toml").exists() {
        true
    } else {
        false
    }
}

/// Placeholder: does the settings file exist?
/// Alias kept for clearer intent at call sites inside UI code.
pub fn settings_found() -> bool {
    find_setting()
}

/// Placeholder: are the settings valid?
/// Currently returns true if the settings file exists. Replace with real semantic validation:
/// - Parse TOML
/// - Validate required sections/fields
/// - Range / value checks
pub fn settings_valid() -> bool {
    // TODO: implement real validation logic
    settings_found()
}

/// Placeholder: was a server certificate (and optionally key) found?
/// For now we just check for a single PEM file in the config directory.
/// Adjust path / strategy once certificate management is implemented.
pub fn certificate_found() -> bool {
    paths::config_dir().join("server_cert.pem").exists()
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
