use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub struct AppBase {
    pub app_id: &'static str,
    pub settings: Arc<settings::SettingsStore>,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl AppBase {
    pub fn app_id(&self) -> &'static str {
        self.app_id
    }
}

impl Deref for AppBase {
    type Target = settings::SettingsStore;
    fn deref(&self) -> &Self::Target {
        &self.settings
    }
}

// impl DerefMut for AppBase {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.settings
//     }
// }
pub trait Application: Sized + 'static {
    type Error: From<BoxError> + std::fmt::Display + std::fmt::Debug + 'static;

    const APP_ID: &'static str;
    const EMBEDDED_SETTINGS_ASSET: Option<&'static str> = None;
    const EMBEDDED_KEYMAP_ASSET: Option<&'static str> = None;
    const ENV_LAYERS_VAR: Option<&'static str> = None;
    const ENV_PREFIX: Option<&'static str> = None;

    fn init_platform() -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn init<A: Application>() -> Result<AppBase, A::Error> {
    A::init_platform()?;

    let app_id = A::APP_ID;

    let mut builder = settings::SettingsStore::builder();

    if let Some(asset) = A::EMBEDDED_SETTINGS_ASSET {
        builder = builder.with_embedded_setting_asset(asset);
    }
    if let Some(asset) = A::EMBEDDED_KEYMAP_ASSET {
        builder = builder.with_embedded_keymap_asset(asset);
    }

    builder = builder.with_user_config_dir();

    let env_layers_enabled = match A::ENV_LAYERS_VAR {
        Some(var) => settings::parse_bool(var).unwrap_or(true),
        None => true,
    };
    builder = builder.enable_env_layers(env_layers_enabled);
    if env_layers_enabled {
        if let Some(prefix) = A::ENV_PREFIX {
            builder = builder.with_env_prefix(prefix);
        }
    }

    let settings = Arc::new(builder.build().expect("Build Settings Error")); // <- Arc drum

    let config_dir = paths::config_dir().join(app_id);
    let data_dir = paths::data_dir().join(app_id);
    let logs_dir = paths::logs_dir().join(app_id);

    Ok(AppBase {
        app_id,
        settings,
        config_dir,
        data_dir,
        logs_dir,
    })
}
