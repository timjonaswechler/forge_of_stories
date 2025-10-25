mod client;
mod fos_app;
mod input;
mod rendering;
mod ui;
mod utils;

use crate::client::ClientPlugin;
use crate::fos_app::FOSApp;
use crate::rendering::RenderingPlugin;
use crate::ui::UIPlugin;
use app::AppBuilder;
use bevy::{log::LogPlugin, prelude::*};
use game_server::settings::Network;
use input::{
    InputPlugin as KeymapInputPlugin, StoreResource as KeymapStoreResource, create_keymap_store,
};
use settings::{AppSettingsExt, SettingsStore};

/// Game state tracking where we are in the application flow.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Splashscreen,
    MainMenu,
    ConnectingToServer,
    InGame,
}

fn main() {
    let mut app = AppBuilder::<FOSApp>::new(env!("CARGO_PKG_VERSION"))
        .expect("Failed to initialize application")
        .build_with_bevy(|mut app, ctx| {
            let keymap_store =
                create_keymap_store(ctx.path_context()).expect("failed to create keymap store");

            let settings_store = SettingsStore::builder("0.1.0")
                .with_settings_file(ctx.path_context().settings_file(Some(ctx.app_id())))
                .build()
                .expect("failed to build settings store");

            app = app
                .insert_settings_store(settings_store)
                .register_settings_section::<Network>();

            app.add_plugins(
                DefaultPlugins
                    .build()
                    .disable::<LogPlugin>()
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            title: "Forge of Stories".to_string(),
                            ..default()
                        }),
                        ..default()
                    }),
            );

            // Initialize GameState
            app.init_state::<GameState>();

            app.insert_resource(KeymapStoreResource::new(keymap_store));

            // Add plugins: UI (cameras + scenes), Rendering (lighting + visuals), Client (networking)
            // Server logic runs in embedded server thread (via ServerHandle)
            app.add_plugins((UIPlugin, RenderingPlugin, ClientPlugin, KeymapInputPlugin));

            // Optional: poll for settings changes on disk and update registered sections.
            // app.add_systems(
            //     Update,
            //     settings::settings_reload_system.run_if(on_timer(Duration::from_secs(1))),
            // );

            app
        });

    app.run();
}
