pub mod in_game;
mod in_game_menu;
mod main_menu;
mod splashscreen;

use bevy::prelude::*;

pub use in_game::InGameScenePlugin;
pub use in_game_menu::InGameMenuScenePlugin;
pub use main_menu::MainMenuScenePlugin;
pub use splashscreen::SplashscreenScenePlugin;

/// Main scene plugin that coordinates all scene sub-plugins
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SplashscreenScenePlugin,
            MainMenuScenePlugin,
            InGameScenePlugin,
            InGameMenuScenePlugin,
        ));
    }
}
