mod aether;
pub mod bevy;

use crate::aether::AetherApp;
use crate::bevy::build_bevy_app;
use aether_config::bevy::AppAetherSettingsExt;
use app::AppBuilder;

#[tokio::main]
pub async fn main() {
    let mut app = AppBuilder::<AetherApp>::new(env!("CARGO_PKG_VERSION"))
        .expect("Failed to initialize application")
        .build_with_bevy(|app, ctx| {
            let mut app = app.use_aether_server_settings(ctx, None);
            build_bevy_app(&mut app);
            app
        });
    
    app.run();
}
