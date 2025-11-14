mod app;
mod input;
mod ui;
mod utils;

use crate::input::InputPlugin;
use crate::ui::UIPlugin;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

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
    let mut app = app::init();

    // Initialize GameState
    app.init_state::<GameState>();

    // Add EnhancedInputPlugin BEFORE KeymapInputPlugin
    app.add_plugins((
        EnhancedInputPlugin, // TODO: in port in keymap plugin
        InputPlugin,
        UIPlugin,
    ));

    app.run();
}
