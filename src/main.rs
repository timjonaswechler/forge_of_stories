// ############ crates/forge_app/src/main.rs (NEU) ############
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use std::collections::HashMap;

use forge_of_stories::*;

use forge_ui::{
    components::{button::ButtonBuilder, label::LabelBuilder},
    layout::UiRoot,
    theme::*,
    ForgeUiPlugin, UiState,
};

fn transition_to_main_menu_when_ui_ready(
    ui_state: Res<State<UiState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    // wenn die UI-Plugin-State Ready ist, schalte in MainMenu
    if ui_state.get() == &UiState::Ready {
        next_app_state.set(AppState::MainMenu);
    }
}

// Define States for Loading etc.
#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    InGame,
    Paused,
    Cleanup,
    Ready,
}

fn main() {
    App::new()
        // --- Game-State initialisieren ---
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
                    file_path: "assets".into(),
                    watch_for_changes_override: Some(true),
                    ..Default::default()
                }),
        )
        .init_state::<AppState>()
        // --- UI Plugin mit eigener UiState-Zustandsmaschine ---
        .add_plugins(ForgeUiPlugin::new())
        // --- Brücke zwischen UiState und AppState ---
        .add_systems(
            Update,
            transition_to_main_menu_when_ui_ready.run_if(in_state(AppState::Loading)), // nur solange wir noch laden
        )
        // --- Wenn wir in MainMenu landen, baue das UI auf ---
        .add_systems(
            OnEnter(AppState::MainMenu),
            (ApplyDeferred, setup_main_menu)
                .chain()
                .run_if(|res: Option<Res<UiTheme>>| resource_exists(res)), // optional: warte auf Theme
        )
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
