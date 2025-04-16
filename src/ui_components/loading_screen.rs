// src/ui_components/loading_screen.rs
use bevy::prelude::*;

use crate::{
    app_setup::{AssetLoadingTracker, GameAssets}, // Zugriff auf Tracker und Asset Handles
    AppState,
};

// Plugin für den Ladebildschirm
pub struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app
            // Füge Systeme hinzu, die nur im Loading-Zustand laufen
            .add_systems(OnEnter(AppState::Loading), setup_loading_ui)
            .add_systems(
                Update,
                (
                    update_loading_progress,
                    check_loading_complete_and_transition, // Prüft, ob Assets geladen sind
                )
                    .run_if(in_state(AppState::Loading)),
            )
            .add_systems(OnExit(AppState::Loading), cleanup_loading_ui);
    }
}

// Marker-Komponente für den Root-Node des Ladebildschirms
#[derive(Component)]
struct LoadingScreenUIRoot;

// Marker-Komponente für das Haupt-Ladestatus-Textfeld
#[derive(Component)]
struct LoadingText;

// Marker für den Fortschrittstext
#[derive(Component)]
struct LoadingProgressText;

// System zum Erstellen der Lade-UI (fügt Logo hinzu)
fn setup_loading_ui(mut commands: Commands, game_assets: Option<Res<GameAssets>>) {
    info!("Setting up loading screen UI with Logo...");

    // Hole Font und Logo Handles
    let font_handle = game_assets
        .as_ref() // Nimm eine Referenz, um den Ownership nicht zu verschieben
        .and_then(|ga| ga.fonts.first().cloned())
        .unwrap_or_else(|| {
            warn!("Font not available for loading screen setup, using default font.");
            Default::default()
        });

    // Hole den Logo-Handle. Wenn GameAssets noch nicht da ist, können wir das Logo nicht anzeigen.
    let logo_handle = game_assets.map(|ga| ga.logo.clone()); // Klonen

    commands
        .spawn((
            Node {
                // Style für Layout direkt in Node oder via Style-Komponente
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0), // Etwas mehr Abstand für das Logo
                ..default()
            },
            BackgroundColor(Color::BLACK),
            LoadingScreenUIRoot,
        ))
        .with_children(|parent| {
            // --- Logo ---
            // Nur spawnen, wenn der Handle verfügbar ist
            if let Some(handle) = logo_handle {
                parent.spawn(ImageNode {
                    // Node für Größe und ggf. weiteres Styling
                    image: handle,
                    ..default()
                });
            } else {
                warn!("Logo handle not available during loading screen setup.");
            }

            // --- Lade-Text ("Loading...") ---
            parent.spawn((
                Text::new("Loading..."),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                LoadingText,
                Label,
            ));

            // --- Fortschritts-Text ("Progress: X / Y") ---
            parent.spawn((
                Text::new("Progress: 0 / ?"),
                TextFont {
                    font: font_handle,
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)), // Korrigiert zu Color::GRAY
                LoadingProgressText,
                Label,
            ));
        });
}

// System zum Aktualisieren des Fortschrittstextes (Angepasst an Bevy 0.15 Text API)
fn update_loading_progress(
    asset_server: Res<AssetServer>,
    game_assets: Option<Res<GameAssets>>, // Behalte Option<Res> für Robustheit
    mut progress_text_query: Query<&mut Text, With<LoadingProgressText>>,
) {
    let Some(game_assets) = game_assets else {
        // Frühzeitiger Ausstieg, wenn Assets noch nicht geladen/Ressource nicht verfügbar
        return;
    };

    let total_assets =
        game_assets.species_templates.len() + game_assets.fonts.len() + game_assets.textures.len();

    let mut loaded_count = 0;

    // Zähle geladene Templates
    loaded_count += game_assets
        .species_templates
        .iter()
        .filter(|handle| asset_server.is_loaded_with_dependencies(*handle))
        .count();

    // Zähle geladene Fonts
    loaded_count += game_assets
        .fonts
        .iter()
        .filter(|handle| asset_server.is_loaded_with_dependencies(*handle))
        .count();

    // Zähle geladene Texturen
    loaded_count += game_assets
        .textures
        .iter()
        .filter(|handle| asset_server.is_loaded_with_dependencies(*handle))
        .count();

    // Aktualisiere den Fortschrittstext
    for mut text in progress_text_query.iter_mut() {
        // --- KORREKTER ZUGRIFF für Bevy 0.15 ---
        // Greife direkt auf den String-Inhalt über den Index .0 zu
        text.0 = format!("Progress: {} / {}", loaded_count, total_assets);
        // -----------------------------------------
    }
}

// --- Angepasst für Übergang zum MainMenu ---
fn check_loading_complete_and_transition(
    tracker: Res<AssetLoadingTracker>,
    mut next_state: ResMut<NextState<AppState>>, // Bleibt NextState
    mut ev_assets_loaded: EventWriter<crate::app_setup::events::AssetsLoadedEvent>,
) {
    // Prüft, ob ALLE Assets geladen sind (inkl. Logo via textures_loaded)
    if tracker.species_templates_loaded && tracker.fonts_loaded && tracker.textures_loaded {
        info!("Loading complete. Transitioning to AppState::MainMenu"); // Ziel geändert!
        next_state.set(AppState::MainMenu); // <<<----- HIER ÄNDERN
        ev_assets_loaded.send(crate::app_setup::events::AssetsLoadedEvent);
    }
}

// --- cleanup_loading_ui bleibt gleich ---
fn cleanup_loading_ui(mut commands: Commands, query: Query<Entity, With<LoadingScreenUIRoot>>) {
    info!("Cleaning up loading screen UI...");
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
