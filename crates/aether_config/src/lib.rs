use serde::{Deserialize, Serialize};
use settings::{Settings, SettingsStore};
use std::{result::Result, sync::Arc};

// 1) Typisierte Modelle
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct GeneralCfg {
    pub tick_rate: u32,
    pub autostart: bool,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct NetworkCfg {
    pub ip_address: String,
    pub udp_port: u16, /* ... */
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
struct General;
impl Settings for General {
    const SECTION: &'static str = "general";
    type Model = GeneralCfg;
}

struct Network;
impl Settings for Network {
    const SECTION: &'static str = "network";
    type Model = NetworkCfg;
}

struct Monitoring;
impl Settings for Monitoring {
    const SECTION: &'static str = "monitoring";
    type Model = MonitoringCfg;
}

struct Security;
impl Settings for Security {
    const SECTION: &'static str = "security";
    type Model = SecurityCfg;
}

struct Uds;
impl Settings for Uds {
    const SECTION: &'static str = "uds";
    type Model = UdsCfg;
}

// ... Security, Monitoring, Uds analog

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
        .with_embedded_setting_asset("settings/server-default.toml")
        .with_settings_file_optional("aether.toml".into());

    let env_layers_enabled = std::env::var(ENV_LAYERS_VAR)
        .ok()
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(true);

    builder = builder.enable_env_layers(env_layers_enabled);
    if env_layers_enabled {
        builder = builder.with_env_prefix(ENV_PREFIX);
    }

    Ok(builder.build()?)
}

// 4) Registrieren + typisiertes Aggregat ziehen
pub fn load_typed_server_settings(
    store: &settings::SettingsStore,
) -> color_eyre::Result<ServerSettings> {
    store.register::<General>()?;
    store.register::<Network>()?;
    store.register::<Security>()?;
    store.register::<Monitoring>()?;
    store.register::<Uds>()?;

    Ok(ServerSettings {
        general: store.get::<General>()?,
        network: store.get::<Network>()?,
        security: store.get::<Security>()?,
        monitoring: store.get::<Monitoring>()?,
        uds: store.get::<Uds>()?,
    })
}
