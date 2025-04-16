// src/ui_components/main_menu.rs
use bevy::prelude::*;

use crate::{app_setup::GameAssets, AppState}; // Zugriff auf Assets für Fonts etc.

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Systeme für den MainMenu-Zustand
            .add_systems(OnEnter(AppState::MainMenu), setup_main_menu_ui)
            .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu_ui)
            .add_systems(
                Update,
                (handle_start_button_interaction,) // System für Button-Klicks
                    .run_if(in_state(AppState::MainMenu)),
            );
    }
}

// Marker für das Root-Node des Hauptmenüs
#[derive(Component)]
struct MainMenuUIRoot;

// Marker für den "Spiel starten"-Button
#[derive(Component)]
struct StartGameButton;

// System zum Erstellen der Hauptmenü-UI
fn setup_main_menu_ui(mut commands: Commands, game_assets: Option<Res<GameAssets>>) {
    info!("Setting up main menu UI...");

    // Hole Font Handle (oder Fallback)
    let font_handle = game_assets
        .and_then(|ga| ga.fonts.first().cloned())
        .unwrap_or_else(|| {
            warn!("Font not available for main menu setup, using default font.");
            Default::default()
        });

    // Farben für Buttons (Beispiel)
    let button_normal_color = Color::srgb(0.15, 0.15, 0.15);
    let button_hover_color = Color::srgb(0.25, 0.25, 0.25);
    let button_pressed_color = Color::srgb(0.35, 0.75, 0.35);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),

                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)), // Anderer Hintergrund für Menü
            MainMenuUIRoot,
        ))
        .with_children(|parent| {
            // --- Titel ---
            parent.spawn((
                Text::new("Forge of Stories"),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Label,
            ));

            // --- "Spiel starten" Button ---
            parent
                .spawn((
                    // Button ist eine Node mit Button-Komponente und Interaction
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center, // Text zentrieren
                        align_items: AlignItems::Center,         // Text zentrieren
                        padding: UiRect::all(Val::Px(10.0)),

                        ..default()
                    },
                    Button::default(),                    // Button-Komponente
                    Interaction::default(),               // Wichtig für Klick-Erkennung
                    BackgroundColor(button_normal_color), // Normale Farbe
                    StartGameButton,                      // Marker für diesen Button
                ))
                .with_children(|button_parent| {
                    // Text im Button
                    button_parent.spawn((
                        Text::new("Start Simulation"),
                        TextFont {
                            font: font_handle.clone(), // Kann derselbe Font sein
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Label,
                    ));
                });

            // --- Weitere Buttons (Platzhalter) ---
            // parent.spawn(... Button "Einstellungen" ...);
            // parent.spawn(... Button "Beenden" ...);
        });
}

// System zum Handhaben von Button-Interaktionen
fn handle_start_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<StartGameButton>), // Nur geänderte Interaktionen für unseren Button
    >,
    mut next_state: ResMut<NextState<AppState>>, // Zum Ändern des Spielzustands
    mut ev_startup_completed: EventWriter<crate::app_setup::events::AppStartupCompletedEvent>, // Optional: Event senden
) {
    // Farben aus setup_main_menu_ui wiederverwenden oder hier definieren
    let button_normal_color = Color::srgb(0.15, 0.15, 0.15);
    let button_hover_color = Color::srgb(0.25, 0.25, 0.25);
    let button_pressed_color = Color::srgb(0.35, 0.75, 0.35);

    for (interaction, mut background_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *background_color = BackgroundColor(button_pressed_color);
                info!("Start button pressed! Transitioning to AppState::Running.");
                next_state.set(AppState::Running); // Wechsle zum Spielzustand
                                                   // Optional: Event senden, dass das Spiel/die Simulation beginnt
                ev_startup_completed.send(crate::app_setup::events::AppStartupCompletedEvent);
            }
            Interaction::Hovered => {
                *background_color = BackgroundColor(button_hover_color);
            }
            Interaction::None => {
                *background_color = BackgroundColor(button_normal_color);
            }
        }
    }
}

// System zum Aufräumen der Hauptmenü-UI
fn cleanup_main_menu_ui(mut commands: Commands, query: Query<Entity, With<MainMenuUIRoot>>) {
    info!("Cleaning up main menu UI...");
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
