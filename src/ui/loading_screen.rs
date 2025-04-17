// src/ui_components/loading_screen.rs
use crate::initialization::{
    // Add other necessary imports like LoadingScreenDelayTimer, AssetsLoadedEvent etc.
    AppState, // Assuming AppState is here
    AssetLoadingTracker,
    AssetsLoadedEvent,
    EssentialAssets,
    GameAssets, // Assuming these are here
};
use bevy::{input::keyboard::KeyboardInput, prelude::*};

// --- Marker Component for the Progress Text ---
#[derive(Component)]
pub struct LoadingProgressText;

// --- Timer Resource ---
#[derive(Resource)]
pub struct LoadingScreenDelayTimer(pub Timer); // Ensure this is defined

#[derive(Component)]
struct LoadingScreenUIRoot;

#[derive(Component)]
struct LoadingText;

// Plugin Definition (unverändert)
pub struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Loading), setup_loading_ui) // Bleibt OnEnter(Loading)
            .add_systems(
                Update,
                (
                    update_loading_progress, // Überwacht jetzt GameAssets Fortschritt
                    check_loading_complete_and_transition, // Reagiert auf AssetLoadingTracker
                )
                    .run_if(in_state(AppState::Loading)),
            )
            .add_systems(
                OnExit(AppState::Loading),
                (cleanup_loading_ui, remove_delay_timer),
            );
    }
}

// System zum Erstellen der Lade-UI (verwendet jetzt EssentialAssets)
fn setup_loading_ui(
    mut commands: Commands,
    essential_assets: Res<EssentialAssets>, // Greife auf die vorab geladenen Assets zu
) {
    info!("Setting up loading screen UI using preloaded essential assets...");

    // --- Hole Handles aus der Ressource ---
    let font_handle = essential_assets.font.clone();
    let logo_handle = essential_assets.logo.clone();
    // --- Kein asset_server.load() mehr hier! ---

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
            BackgroundColor(Color::BLACK),
            // BorderColor(Color::srgb(1.0, 0.0, 1.0)), // Optional: Rand entfernen/ändern
            LoadingScreenUIRoot,
        ))
        .with_children(|parent| {
            // --- Logo ---
            parent
                .spawn(Node {
                    width: Val::Auto,
                    height: Val::Auto,
                    margin: UiRect::bottom(Val::Px(30.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,

                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        ImageNode::new(logo_handle.clone()), // Lade das Logo
                        Node {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            width: Val::Px(200.),

                            ..default()
                        },
                        // Uses the transform to rotate the logo image by 45 degrees
                    ));
                });

            // --- Lade-Text ("Loading...") ---
            parent.spawn((
                Text::new("Loading..."),
                TextFont {
                    font: font_handle.clone(),
                    font_size: 40.0,
                    ..default()
                },
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor(Color::WHITE),
                LoadingText,
                Label, // Label hier ist korrekt für Text Accessibility
            ));

            // --- Fortschritts-Text ("Progress: X / Y") ---
            parent.spawn((
                Text::new("Progress: 0 / ?"),
                TextFont {
                    font: font_handle,
                    font_size: 20.0,
                    ..default()
                },
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor(Color::srgb(0.5, 0.5, 0.5)), // Korrigiert zu Color::GRAY
                LoadingProgressText,
                Label, // Label hier ist korrekt für Text Accessibility
            ));
        });
}

// System zum Aktualisieren des Fortschrittstextes (basiert auf GameAssets)
fn update_loading_progress(
    asset_server: Res<AssetServer>,
    game_assets: Res<GameAssets>, // Ist jetzt da, wenn wir in Loading sind
    mut progress_text_query: Query<&mut Text, With<LoadingProgressText>>,
    // tracker: Res<AssetLoadingTracker>, // Alternative: Direkt Tracker verwenden
) {
    // Zähle die Assets in GameAssets
    let total_assets =
        game_assets.species_templates.len() + game_assets.fonts.len() + game_assets.textures.len();

    if total_assets == 0 {
        // Fall: Keine Spiel-Assets zu laden (unwahrscheinlich, aber sicher ist sicher)
        for mut text in progress_text_query.iter_mut() {
            text.0 = "Progress: 0 / 0".to_string();
        }
        return;
    }

    let mut loaded_count = 0;

    // Count loaded species templates
    loaded_count += game_assets
        .species_templates
        .iter()
        .filter(|handle| asset_server.is_loaded_with_dependencies(*handle))
        .count();

    // Count loaded fonts by iterating over the HashMap's *values*
    loaded_count += game_assets
        .fonts // The HashMap<String, Handle<Font>>
        .values() // <<< Use .values() to get an iterator over &Handle<Font>
        .filter(|handle| asset_server.is_loaded_with_dependencies(*handle))
        .count();

    // Count loaded textures
    loaded_count += game_assets
        .textures
        .iter()
        .filter(|handle| asset_server.is_loaded_with_dependencies(*handle))
        .count();

    for mut text in progress_text_query.iter_mut() {
        // Annahme: Text hat genau eine Section
        text.0 = format!("Progress: {} / {}", loaded_count, total_assets);
    }
}

// System zum Überprüfen, ob ALLE Spiel-Assets geladen sind (nutzt Tracker) und zum Übergang
fn check_loading_complete_and_transition(
    mut commands: Commands,
    time: Res<Time>,
    tracker: Res<AssetLoadingTracker>, // Get the tracker resource
    timer: Option<ResMut<LoadingScreenDelayTimer>>, // Use mut timer for tick()
    mut next_state: ResMut<NextState<AppState>>,
    mut ev_assets_loaded: EventWriter<AssetsLoadedEvent>,
    mut evr_keyboard: EventReader<KeyboardInput>, // Use mut for .read()
) {
    // Prüfen, ob alle *Spiel*-Assets geladen UND verarbeitet sind (basierend auf dem Tracker)
    let assets_loaded_and_processed = tracker.species_templates_loaded
        && tracker.fonts_processed // <<< Check if fonts are processed (folder loaded AND handles extracted)
        && tracker.textures_loaded;

    // --- Restliche Logik (Timer, Tastendruck) ---
    let key_pressed_this_frame = evr_keyboard.read().next().is_some(); // Check if *any* key was pressed
    if key_pressed_this_frame {
        debug!("Key pressed this frame!"); // <-- DEBUG LOG
    }

    if assets_loaded_and_processed {
        if let Some(mut delay_timer) = timer {
            if key_pressed_this_frame {
                info!("Key pressed, skipping loading delay. Transitioning to AppState::MainMenu");
                next_state.set(AppState::MainMenu); // Or your target state
                ev_assets_loaded.send(AssetsLoadedEvent);
                // Optional: remove timer if skipping
                commands.remove_resource::<LoadingScreenDelayTimer>();
                return; // Exit early after transition
            }

            delay_timer.0.tick(time.delta());
            debug!(
                "Timer ticked. Remaining: {:.2}s",
                delay_timer.0.remaining_secs()
            ); // <-- DEBUG LOG
            if delay_timer.0.finished() {
                info!("Loading delay finished. Transitioning to AppState::MainMenu");
                next_state.set(AppState::MainMenu); // Or your target state
                ev_assets_loaded.send(AssetsLoadedEvent);
                // Optional: remove timer after it finishes
                commands.remove_resource::<LoadingScreenDelayTimer>();
                // No return here, let the system finish normally after transition set
            }
            // If timer is ticking but not finished, do nothing this frame
        } else {
            // Assets are loaded, but timer hasn't started yet. Insert it.
            info!("Game assets loaded. Starting delay timer (Press any key to skip)...");
            commands.insert_resource(LoadingScreenDelayTimer(Timer::from_seconds(
                2.0, // Your desired delay
                TimerMode::Once,
            )));
            debug!("Timer resource inserted."); // <-- DEBUG LOG
        }
    } else {
        // Optional: Log, dass Assets noch nicht fertig sind
        // info!("DEBUG: Waiting for assets...");
    }
    // If assets are not loaded yet, do nothing this frame
}

// Cleanup und Timer-Entfernen bleiben gleich
fn cleanup_loading_ui(mut commands: Commands, query: Query<Entity, With<LoadingScreenUIRoot>>) {
    info!("Cleaning up loading screen UI...");
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn remove_delay_timer(mut commands: Commands) {
    info!("Removing loading screen delay timer resource.");
    commands.remove_resource::<LoadingScreenDelayTimer>();
}
