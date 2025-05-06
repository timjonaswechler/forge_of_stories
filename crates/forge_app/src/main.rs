// ############ crates/forge_app/src/main.rs (NEU) ############
use bevy::prelude::*; // <-- Import für RonAssetPlugin von Bevy selbst
use bevy_asset_loader::prelude::StandardDynamicAssetCollection;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin; // <-- Import für RonAssetPlugin
use std::collections::HashMap; // <-- Import für HashMap

use forge_of_stories::*; // Importieren Sie Ihr attributes Modul
                         // Ihr attributes Modul
use forge_ui::{
    components::{
        button::{
            handle_button_clicks_event, update_button_visuals, ButtonBuilder, ButtonClickedEvent,
            ButtonSize,
        },
        label::LabelBuilder,
    },

    layout::UiRoot,

    theme::*,
    // Event bleibt wichtig
    ForgeUiPlugin,
}; // Ihre UI Elemente

// Define States for Loading etc.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum AppState {
    #[default]
    Loading, // Start in loading state
    MainMenu, // Example game state
    SettingsMenu,
    // Settings, // Optional: Example für anderen State
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Forge of Stories".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    // Adjust this path if your assets folder is elsewhere relative to the executable
                    file_path: "../../assets".into(),
                    watch_for_changes_override: Some(true),
                    ..Default::default()
                }),
        )
        // --- Asset Loading Setup ---
        .init_state::<AppState>() // Manage game states
        // Add RonAssetPlugin FÜR JEDEN Typ, den Sie aus .ron/.species.ron/.theme.ron laden wollen
        .add_plugins((
            RonAssetPlugin::<SpeciesData>::new(&["species.ron"]),
            // Aktivieren, wenn Sie Theme laden
        ))
        .add_loading_state(
            LoadingState::new(AppState::Loading)
                .continue_to_state(AppState::MainMenu)
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "game_assets.assets.ron",
                ) // Transition after loading
                .load_collection::<GameAssets>(), // Load our defined assets
        )
        // --- Your Game Plugins ---
        // --- UI Plugin ---
        .add_plugins(ForgeUiPlugin::new()) // Add the UI plugin HERE
        // --- Game Setup & Systems ---
        .add_systems(
            OnEnter(AppState::MainMenu),
            (
                ApplyDeferred, // Stellt sicher, dass Ressource vor nächstem System da ist
                setup_main_menu.run_if(resource_exists::<UiTheme>), // <<< Baut das UI MIT Theme auf
            )
                .chain(), // Sorgt für korrekte Reihenfolge
        )
        .add_systems(
            Update,
            (
                // Button Systeme
                update_button_visuals,
                handle_button_clicks_event,
                handle_button_clicks, // <- Ihr Button-Event-Handler
            )
                .run_if(in_state(AppState::MainMenu)), // << Bedingung HIER setzen
        )
        // Optional: Wenn Sie den fn-Callback-Handler brauchen:
        // .add_systems(Update, forge_ui::button::handle_button_clicks_fn.run_if(in_state(AppState::MainMenu)))
        // Add other game systems...
        .run();
}

// --- Asset Collection Definition ---
#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(key = "species.elf", typed)] // 'typed' wichtig für RonAssetPlugin via Key
    pub elf_species: Handle<SpeciesData>,
    #[asset(key = "species.human", typed)]
    pub human_species: Handle<SpeciesData>,
    #[asset(key = "species.ork", typed)]
    pub ork_species: Handle<SpeciesData>,
}

// --- RON Data Structures ---
#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
pub struct SpeciesData {
    pub species_name: String,
    // Verwenden Sie den tatsächlichen Typ aus Ihrem attributes-Modul
    pub attribute_distributions:
        HashMap<attributes::AttributeType, attributes::AttributeDistribution>,
}

// --- UI Setup System ---
// Runs once when AppState::MainMenu is entered, AFTER setup_theme_resource
fn setup_main_menu(
    mut commands: Commands,
    theme: Res<UiTheme>, // Theme Ressource wird hier benötigt
) {
    // Spawn Camera
    commands.spawn(Camera2d::default());

    // Get the font handle (either from theme or fallback)
    let font_handle = theme.font.font_family.default.clone(); // Fallback, falls Theme keine Font hat

    // Create root UI nodre
    let ui_root_entity = UiRoot::spawn(&mut commands, &theme);

    commands.entity(ui_root_entity).with_children(|parent| {
        // --- Badges Gruppe ---
        parent.spawn(Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center, // Badges zentrieren
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0), // Abstand zwischen Badges
            margin: UiRect::top(Val::Px(20.0)),

            ..default()
        });

        // --- Button Gruppe ---
        parent
            .spawn(Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                align_items: AlignItems::Center, // Zentriert die Buttons
                ..default()
            })
            .with_children(|button_parent| {
                // === Spawn Buttons using the Builder ===

                // --- Label hinzufügen ---
                let _ = LabelBuilder::new("Main Menu Controls")
                    .font_size(18.0) // Etwas größer
                    .color(theme.color.gray.text_primary) // Andere Farbe zum Testen
                    .align(JustifyText::Center) // Zentrieren
                    .spawn(button_parent, &theme, &font_handle);

                // Default Button
                let _ = ButtonBuilder::new()
                    .with_text("Play Game")
                    .border_radius(theme.layout.radius.xs) // Beispiel: BorderRadius aus Theme
                    .spawn(button_parent, &theme, &font_handle); // <<< Pass theme

                // <<< Pass theme
            });
    });

    info!("Main menu UI setup complete.");
}

// --- Marker Components for Specific Buttons ---
#[derive(Component, Default)] // Default hinzugefügt für .mark()
struct DeleteSaveButton;

#[derive(Component, Default)] // Default hinzugefügt für .mark()
struct SettingsButton;

// --- Event Handling System ---
fn handle_button_clicks(
    // mut commands: Commands, // Nur wenn nötig, z.B. um State zu ändern
    mut events: EventReader<ButtonClickedEvent>,
    // Query for markers to identify which button was clicked
    delete_button_query: Query<(), With<DeleteSaveButton>>,
    settings_button_query: Query<(), With<SettingsButton>>,
    // Query für andere Buttons, falls sie keine speziellen Marker haben
    // button_query: Query<&ButtonMarker>,
    // Optional: Zugriff auf andere Ressourcen
    // mut next_state: ResMut<NextState<AppState>>,
) {
    for event in events.read() {
        info!("-> Button Clicked Event: {:?}", event.button_entity);

        if delete_button_query.get(event.button_entity).is_ok() {
            warn!("--> Delete Save button pressed!");
            // Implement save deletion logic here
        } else if settings_button_query.get(event.button_entity).is_ok() {
            info!("--> Settings button pressed!");
            // Navigate to settings menu, e.g.:
            // next_state.set(AppState::Settings);
        } else {
            // Handle other buttons if necessary
            info!("--> A generic button was pressed (no specific marker found).");
        }
    }
}
