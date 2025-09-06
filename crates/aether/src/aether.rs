// use crate::{
//     action::{Action},
// };
use crate::bevy::build_bevy_app;
use app::{AppBase, Application};
use bevy::prelude::App;
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
    pub bevy: App,
}

impl AetherApp {
    pub fn init(base: AppBase) -> Result<Self> {
        let mut bevy_app = App::new();
        build_bevy_app(&mut bevy_app);
        Ok(Self {
            base,
            bevy: bevy_app,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.bevy.run();
        Ok(())
    }
}
