use crate::GameState;
use crate::ui::components::{HOVERED_BUTTON, InGameMenuState, NORMAL_BUTTON, PRESSED_BUTTON};

use app::LOG_CLIENT;
use bevy::color::palettes::basic::RED;
use bevy::prelude::*;

/// Plugin for managing the in-game ESC menu
pub struct InGameMenuScenePlugin;

impl Plugin for InGameMenuScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Note: ESC key handling is now done via Enhanced Input in the KeymapInputPlugin
                spawn_in_game_menu_ui,
                handle_in_game_menu_buttons,
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}

/// Marker component for in-game menu UI entities
#[derive(Component)]
struct InGameMenuUI;

/// Component identifying in-game menu button actions
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum InGameMenuAction {
    Resume,
    LeaveGame,
}

fn spawn_in_game_menu_ui(
    mut commands: Commands,
    menu: Res<InGameMenuState>,
    existing: Query<Entity, With<InGameMenuUI>>,
) {
    if !menu.is_changed() {
        return;
    }

    // Despawn existing menu if closed
    if !menu.is_open() {
        for entity in &existing {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Don't spawn if already exists
    if !existing.is_empty() {
        return;
    }

    // Spawn menu
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
            InGameMenuUI,
            Name::new("InGame Menu Overlay"),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(24.0)),
                        row_gap: Val::Px(16.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderRadius::all(Val::Px(12.0)),
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("In-Game Menu"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));

                    // Resume button
                    panel
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(260.0),
                                height: Val::Px(56.0),
                                border: UiRect::all(Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor::all(Color::WHITE),
                            BorderRadius::all(Val::Px(10.0)),
                            BackgroundColor(NORMAL_BUTTON),
                            InGameMenuAction::Resume,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Resume"),
                                TextFont {
                                    font_size: 22.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ));
                        });

                    // Leave game button
                    panel
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(260.0),
                                height: Val::Px(56.0),
                                border: UiRect::all(Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor::all(Color::WHITE),
                            BorderRadius::all(Val::Px(10.0)),
                            BackgroundColor(NORMAL_BUTTON),
                            InGameMenuAction::LeaveGame,
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new("Leave Game"),
                                TextFont {
                                    font_size: 22.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            ));
                        });
                });
        });
}

fn handle_in_game_menu_buttons(
    mut interaction_query: Query<
        (
            &Interaction,
            &InGameMenuAction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    mut menu: ResMut<InGameMenuState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, action, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                *border_color = BorderColor::all(RED);

                match action {
                    InGameMenuAction::Resume => menu.set_closed(),
                    InGameMenuAction::LeaveGame => {
                        info!(target: LOG_CLIENT, "Leaving game...");
                        menu.set_closed();
                        next_state.set(GameState::MainMenu);
                    }
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                *border_color = BorderColor::all(Color::WHITE);
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                *border_color = BorderColor::all(Color::BLACK);
            }
        }
    }
}
