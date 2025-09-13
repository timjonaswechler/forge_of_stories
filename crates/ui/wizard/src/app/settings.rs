use serde::{Deserialize, Serialize};

pub(crate) use settings::{ActionRegistry, Settings, SettingsStore};

// pub enum WizardSettingField {
//     MetaVersion,
//     WizardTickRate,
//     WizardFps,
//     WizardHelpShowGlobal,
//     WizardHelpWrapOn,
// }

// fn default_value_for(field: WizardSettingField) -> Option<toml::Value> {
//     let txt = settings::default_settings_wizard();
//     let Ok(root) = toml::from_str::<toml::Value>(txt.as_ref()) else {
//         return None;
//     };
//     let tbl = root.as_table()?;

//     let path = match field {
//         WizardSettingField::MetaVersion => &["meta", "version"][..],
//         WizardSettingField::WizardTickRate => &["wizard", "tick_rate"][..],
//         WizardSettingField::WizardFps => &["wizard", "fps"][..],
//         WizardSettingField::WizardHelpShowGlobal => &["wizard", "help_show_global"][..],
//         WizardSettingField::WizardHelpWrapOn => &["wizard", "help_wrap_on"][..],
//     };

//     let mut cur = toml::Value::Table(tbl.clone());
//     for seg in path {
//         match cur {
//             toml::Value::Table(ref t) => {
//                 let Some(v) = t.get(*seg) else {
//                     return None;
//                 };
//                 cur = v.clone();
//             }
//             _ => return None,
//         }
//     }
//     Some(cur)
// }

/// Wendet eine einzelne Feldänderung an. Leerer String => Feld wird auf den eingebetteten Default zurückgesetzt (falls vorhanden).
// pub fn apply_wizard_setting(
//     store: &settings::SettingsStore,
//     field: WizardSettingField,
//     raw_value: &str,
// ) -> color_eyre::Result<()> {
//     let s = raw_value.trim();

//     match field {
//         WizardSettingField::MetaVersion => {
//             let v: String = if s.is_empty() {
//                 default_value_for(field)
//                     .and_then(|v| v.as_str().map(|n| n.to_string()))
//                     .unwrap_or_default()
//             } else {
//                 s.parse()?
//             };
//             store.update::<Meta>(|m| m.version = v)?;
//         }
//         WizardSettingField::WizardTickRate => {
//             let v: f64 = if s.is_empty() {
//                 default_value_for(field)
//                     .and_then(|v| v.as_float().map(|n| n as f64))
//                     .unwrap_or_default()
//             } else {
//                 s.parse()?
//             };
//             store.update::<Wizard>(|w| w.tick_rate = v)?;
//         }
//         WizardSettingField::WizardFps => {
//             let v: f64 = if s.is_empty() {
//                 default_value_for(field)
//                     .and_then(|v| v.as_float().map(|n| n as f64))
//                     .unwrap_or_default()
//             } else {
//                 s.parse()?
//             };
//             store.update::<Wizard>(|w| w.fps = v)?;
//         }
//         WizardSettingField::WizardHelpShowGlobal => {
//             let v: bool = if s.is_empty() {
//                 default_value_for(field)
//                     .and_then(|v| v.as_bool())
//                     .unwrap_or(true)
//             } else {
//                 parse_bool(s)?
//             };
//             store.update::<Wizard>(|w| w.help_show_global = v)?;
//         }
//         WizardSettingField::WizardHelpWrapOn => {
//             let v: bool = if s.is_empty() {
//                 default_value_for(field)
//                     .and_then(|v| v.as_bool())
//                     .unwrap_or(true)
//             } else {
//                 parse_bool(s)?
//             };
//             store.update::<Wizard>(|w| w.help_wrap_on = v)?;
//         }
//     }

//     Ok(())
// }

// 1) Typisierte Modelle
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct MetaCfg {
    pub version: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct WizardCfg {
    pub tick_rate: f64,
    pub fps: f64,
    /// Whether Help should include global key bindings by default.
    pub help_show_global: bool,
    /// Whether Help content wraps long lines by default.
    pub help_wrap_on: bool,
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
    // .with_keymap_file_optional(paths::config_dir().join("wizard-keymap.toml"));

    let store = builder.build()?;
    store.register::<Meta>()?;
    store.register::<Wizard>()?;

    Ok(store)
}

// fn parse_bool(s: &str) -> color_eyre::Result<bool> {
//     let v = s.trim().to_ascii_lowercase();
//     match v.as_str() {
//         "1" | "true" | "yes" | "on" => Ok(true),
//         "0" | "false" | "no" | "off" => Ok(false),
//         _ => Err(color_eyre::eyre::eyre!(format!(
//             "Ungültiger bool-Wert: {s}"
//         ))),
//     }
// }
