use crate::bevy::build_bevy_app;
use aether_config::bevy::AppAetherSettingsExt;
use app::{AppBase, Application};
use bevy::{app::Update, prelude::App};
use color_eyre::Result;

impl Application for AetherApp {
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
    const APP_ID: &'static str = "aether";
    fn init_platform() -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Aether application container.
///
/// Keeps `AppBase` alive (including the embedded `log_guard`) so that the
/// non-blocking tracing appender continues flushing log lines for the
/// lifetime of the server process.
pub struct AetherApp {
    pub base: AppBase,
    pub bevy: App,
}

impl AetherApp {
    pub fn init(base: AppBase) -> Result<Self> {
        let mut bevy_app =
            App::new().use_aether_server_settings(&base.config_dir, None, base.version());
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
