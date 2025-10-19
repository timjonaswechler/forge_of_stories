mod client;
mod fos_app;
mod server;
mod ui;
mod utils;

use crate::client::*;
use crate::fos_app::FOSApp;
use app::AppBuilder;
use bevy::{log::LogPlugin, prelude::*};
use game_server::ServerHandle;
use ui::UIMenuPlugin;

/// Game state tracking where we are in the application flow.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Splashscreen,
    MainMenu,
    InGame,
}

fn main() {
    let mut app = AppBuilder::<FOSApp>::new(env!("CARGO_PKG_VERSION"))
        .expect("Failed to initialize application")
        .build_with_bevy(|mut app, _ctx| {
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

            // Add UiPlugin
            app.add_plugins(UIMenuPlugin);

            app
        });

    app.run();
}
