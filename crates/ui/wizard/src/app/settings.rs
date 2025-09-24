use serde::{Deserialize, Serialize};

pub(crate) use settings::{Settings, SettingsStore};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct MetaCfg {
    pub version: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct WizardCfg {
    pub tick_rate: f64,
    pub fps: f64,
    pub notification_lifetime_ms: u64,
    pub notification_max: u32,
}

// 2) SECTION-Bindings
pub struct Meta;
impl Settings for Meta {
    const SECTION: &'static str = "meta";
    type Model = MetaCfg;
}

pub struct Wizard;
impl Settings for Wizard {
    const SECTION: &'static str = "wizard";
    type Model = WizardCfg;
}

// 3) Zentraler Builder: überall gleich aufrufbar (Wizard & Runtime)
pub fn build_wizard_settings_store() -> color_eyre::Result<SettingsStore> {
    let builder = SettingsStore::builder()
        .with_embedded_setting_asset("settings/wizard-default.toml")
        .with_settings_file_optional(paths::config_dir().join("wizard.toml"))
        .with_embedded_keymap_asset("keymaps/wizard-default.toml");

    let store = builder.build()?;
    store.register::<Meta>()?;
    store.register::<Wizard>()?;

    Ok(store)
}
