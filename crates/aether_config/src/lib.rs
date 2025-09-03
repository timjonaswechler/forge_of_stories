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
            let v: u32 = if s.is_empty() {
                default_value_for(field)
                    .and_then(|v| v.as_integer().map(|n| n as u32))
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
            store.update::<Security>(|m| m.tls_cert_path = v)?;
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
    pub tick_rate: u32,
    pub autostart: bool,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct NetworkCfg {
    pub ip_address: String,
    pub udp_port: u16,
    pub max_concurrent_bidi_streams: u32,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct SecurityCfg {
    pub tls_cert_path: String, /* ... */
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
