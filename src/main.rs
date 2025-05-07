// src/main.rs
use bevy::prelude::*;

use forge_ui::{
    components::{button::ButtonBuilder, label::LabelBuilder, switch::ToggleSwitchBuilder},
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
        .add_plugins(ForgeUiPlugin::new().with_font_size(90.0))
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
                    .color(theme.color.gray.step11) // Andere Farbe zum Testen
                    .align(JustifyText::Center) // Zentrieren
                    .spawn(button_parent, &theme, &font_handle);

                // Default Button
                let _ = ButtonBuilder::new()
                    .with_text("Play Game")
                    .border_radius(theme.layout.radius.xs) // Beispiel: BorderRadius aus Theme
                    .spawn(button_parent, &theme, &font_handle); // <<< Pass theme

                // <<< Pass theme
            });
        let _ = ToggleSwitchBuilder::new().spawn(parent, &theme /*, &font_handle*/); // font_handle auskommentiert, da nicht direkt verwendet

        // Anfangs aktivierter Switch
        let _ = ToggleSwitchBuilder::new()
            .checked(true)
            .with_color(theme.color.green.step10)
            .spawn(parent, &theme /*, &font_handle*/);

        // Deaktivierter Switch
        let _ = ToggleSwitchBuilder::new()
            .disabled(true)
            .spawn(parent, &theme /*, &font_handle*/);

        // Deaktivierter und aktivierter Switch
        let _ = ToggleSwitchBuilder::new()
            .checked(true)
            .disabled(true)
            .with_radius(50.0)
            .with_color(theme.color.red.step05) // Beispiel: Rote Farbe
            .spawn(parent, &theme /*, &font_handle*/);
    });

    info!("Main menu UI setup complete.");
}

// --- Marker Components for Specific Buttons ---
#[derive(Component, Default)] // Default hinzugefügt für .mark()
struct DeleteSaveButton;

#[derive(Component, Default)] // Default hinzugefügt für .mark()
struct SettingsButton;
