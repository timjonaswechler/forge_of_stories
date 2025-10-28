//! Splashscreen 2D UI Layer
//!
//! Contains all 2D UI elements for the splashscreen (text, overlays).

use crate::GameState;
use bevy::prelude::*;

/// Plugin for splashscreen UI elements
pub(super) struct SplashscreenUIPlugin;

impl Plugin for SplashscreenUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Splashscreen), spawn_ui);
    }
}

/// Marker component for splashscreen UI entities
#[derive(Component)]
pub(super) struct SplashscreenUI;

/// Spawns the 2D UI overlay for the splashscreen
fn spawn_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            SplashscreenUI,
            Name::new("Splashscreen UI Root"),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("SPLASHSCREEN"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Subtitle/Instructions
            parent.spawn((
                Text::new("Press any key to continue..."),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}
