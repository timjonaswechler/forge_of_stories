//! In-game HUD (Heads-Up Display)

use crate::{GameState, utils::cleanup};
use bevy::prelude::*;

pub(super) struct InGameHUDPlugin;

impl Plugin for InGameHUDPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_hud)
            .add_systems(OnExit(GameState::InGame), cleanup::<InGameHUD>);
    }
}

/// Marker component for HUD entities
#[derive(Component)]
pub(super) struct InGameHUD;

fn spawn_hud(mut commands: Commands) {
    let ui_text = "Singleplayer\nPress ESC for menu\nPress C to toggle camera";

    commands.spawn((
        Text::new(ui_text),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        InGameHUD,
        Name::new("InGame HUD"),
    ));
}
