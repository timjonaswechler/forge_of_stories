// src/ui_components/main_menu.rs

use bevy::prelude::*;
// Entferne Farbkonstanten für Buttons, kommen jetzt aus dem Theme

use crate::{
    // Assets/Events kommen aus initialization
    initialization::AppStartupCompletedEvent,
    // UI Elemente kommen aus ui
    ui::{
        theme::UiTheme,                                             // Zugriff auf das Theme
        widgets::button::{ButtonPressedEvent, ButtonWidgetBuilder}, // Unser Button
    },
    // AppState kommt jetzt direkt aus crate::
    AppState,
};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), setup_main_menu_ui)
            .add_systems(OnExit(AppState::MainMenu), cleanup_main_menu_ui)
            .add_systems(
                Update,
                // Ändere das System, um auf unser Event zu reagieren
                (handle_start_button_press,).run_if(in_state(AppState::MainMenu)),
            );
        // Das allgemeine button_interaction_system wird vom UiPlugin hinzugefügt.
    }
}
// Marker (unverändert)
#[derive(Component)]
struct MainMenuUIRoot;

// Neuer Marker speziell für den Start-Button, um ihn im Event zu identifizieren
#[derive(Component, Default)]
struct StartGameButtonMarker;

// --- Setup mit direkter Komponentenstruktur für den Button ---
fn setup_main_menu_ui(mut commands: Commands, theme: Res<UiTheme>) {
    let root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
            MainMenuUIRoot,
        ))
        .id();

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Text::new("Forge of Stories"),
            TextFont {
                font: theme.default_font().clone(),
                font_size: 70.0,
                ..default()
            },
            TextLayout::new_with_justify(JustifyText::Center),
            TextColor(Color::WHITE),
        ));

        ButtonWidgetBuilder::new("Start Simulation")
            .with_width(Val::Px(300.0))
            .with_height(Val::Px(65.0))
            .with_marker::<StartGameButtonMarker>() // <--- Marker direkt beim Spawn!
            .spawn(parent, &theme);
    });
}

// System reagiert jetzt auf unser ButtonPressedEvent
fn handle_start_button_press(
    mut button_press_reader: EventReader<ButtonPressedEvent>,
    start_button_query: Query<Entity, With<StartGameButtonMarker>>, // Query nach unserem Marker
    mut next_state: ResMut<NextState<AppState>>,
    mut ev_startup_completed: EventWriter<AppStartupCompletedEvent>,
) {
    // Finde heraus, welche Entity unser Start-Button ist
    // (Geht davon aus, dass es nur einen gibt, sonst Resource oder andere Logik nötig)
    let Ok(start_button_entity) = start_button_query.get_single() else {
        // Button noch nicht gespawnt oder mehr als einer? Frühzeitig zurück.
        // Oder logge einen Fehler.
        // warn!("StartGameButtonMarker query did not return exactly one entity.");
        return;
    };

    for event in button_press_reader.read() {
        // Prüfe, ob das Event von unserem Start-Button kam
        if event.0 == start_button_entity {
            info!("Start button pressed (via ButtonPressedEvent)! Transitioning to AppState::Running.");
            next_state.set(AppState::Running);
            ev_startup_completed.send(AppStartupCompletedEvent);
            // Da wir das Event gelesen haben, müssen wir nicht mehr tun
            break; // Verlasse die Schleife, wenn der richtige Button gefunden wurde
        }
    }
}

// Cleanup (unverändert)
fn cleanup_main_menu_ui(mut commands: Commands, query: Query<Entity, With<MainMenuUIRoot>>) {
    info!("Cleaning up main menu UI...");
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
