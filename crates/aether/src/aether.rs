// use crate::{
//     action::{Action},
// };
use app::{AppBase, Application};
use color_eyre::Result;

impl Application for AetherApp {
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    const APP_ID: &'static str = "aether";

    // eingebettete Assets für Aether
    const EMBEDDED_SETTINGS_ASSET: Option<&'static str> = Some("settings/aether-default.toml");
    const EMBEDDED_KEYMAP_ASSET: Option<&'static str> = None;

    // ENV-Integration wie in deinem bisherigen build_aether_settings_store()
    const ENV_LAYERS_VAR: Option<&'static str> = Some("FOS_AETHER_ENV_LAYERS");
    const ENV_PREFIX: Option<&'static str> = Some("FOS_AETHER");

    fn init_platform() -> Result<(), Self::Error> {
        // Falls du die Init gern hier zentral haben willst:
        // crate::errors::init()?;
        // crate::logging::init()?;
        Ok(())
    }
}

pub struct AetherApp {
    pub base: AppBase,
}

impl AetherApp {
    pub fn init(base: AppBase) -> Result<Self> {
        Ok(Self { base })
    }

    pub async fn run(&mut self) -> Result<()> {
        Ok(())
    }
}
