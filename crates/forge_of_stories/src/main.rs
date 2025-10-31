mod client;
mod fos_app;
mod ui;
mod utils;

use crate::client::ClientPlugin;
use crate::fos_app::FOSApp;
use crate::ui::UIPlugin;
use app::AppBuilder;
use bevy::{log::LogPlugin, prelude::*};
use bevy_enhanced_input::prelude::*;
use game_server::settings::Network;
use keymap::KeymapPlugin;
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

            // Add EnhancedInputPlugin BEFORE KeymapInputPlugin
            app.add_plugins((
                KeymapPlugin::with_config_path(ctx.path_context().keybinding_file()),
                EnhancedInputPlugin, // TODO: in port in keymap plugin
                UIPlugin,
                ClientPlugin,
            ));

            app
        });

    app.run();
}
