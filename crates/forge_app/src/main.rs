// ############ crates/forge_app/src/main.rs (NEU) ############
use bevy::prelude::*; // <-- Import für RonAssetPlugin von Bevy selbst
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin; // <-- Import für RonAssetPlugin
use std::collections::HashMap; // <-- Import für HashMap

use forge_app::attributes::{self, AttributesPlugin}; // Ihr attributes Modul
use forge_ui::{
    button::{ButtonBuilder, ButtonSize, ButtonVariant}, // Nur Button-Sachen die wir brauchen
    card::{CardBuilder, ElementStyle, NodeElement},     // << Builder und Helfer importieren
    theme::UiTheme,
    ButtonClickedEvent, // Event bleibt wichtig
    ForgeUiPlugin,
}; // Ihre UI Elemente

// Define States for Loading etc.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum AppState {
    #[default]
    Loading, // Start in loading state
    MainMenu, // Example game state
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
            RonAssetPlugin::<UiThemeData>::new(&["theme.ron"]), // Aktivieren, wenn Sie Theme laden
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
        .add_plugins(AttributesPlugin) // Add your attributes logic
        // --- UI Plugin ---
        .add_plugins(ForgeUiPlugin) // Add the UI plugin HERE
        // --- Game Setup & Systems ---
        .add_systems(
            OnEnter(AppState::MainMenu),
            (
                setup_theme_resource, // <<< Lädt das Theme in eine Ressource
                apply_deferred,       // Stellt sicher, dass Ressource vor nächstem System da ist
                setup_main_menu,      // <<< Baut das UI MIT Theme auf
            )
                .chain(), // Sorgt für korrekte Reihenfolge
        )
        .add_systems(
            Update,
            handle_button_clicks.run_if(in_state(AppState::MainMenu)), // Handle clicks in menu state
        )
        // Optional: Wenn Sie den fn-Callback-Handler brauchen:
        // .add_systems(Update, forge_ui::button::handle_button_clicks_fn.run_if(in_state(AppState::MainMenu)))
        // Add other game systems...
        .run();
}

// --- Asset Collection Definition ---
#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    // #[asset(key = "font.main")] pub main_font: Handle<Font>, // Wenn über Key geladen
    #[asset(path = "fonts/Roboto-Regular.ttf")]
    pub main_font: Handle<Font>, // Fallback, wenn nicht im Theme/Key

    #[asset(key = "icon.settings")]
    pub settings_icon: Handle<Image>,
    #[asset(key = "icon.delete")]
    pub delete_icon: Handle<Image>,

    #[asset(key = "species.elf", typed)] // 'typed' wichtig für RonAssetPlugin via Key
    pub elf_species: Handle<SpeciesData>,
    #[asset(key = "species.human", typed)]
    pub human_species: Handle<SpeciesData>,
    #[asset(key = "species.ork", typed)]
    pub ork_species: Handle<SpeciesData>,

    #[asset(key = "theme.default", typed)]
    pub theme_data: Handle<UiThemeData>,
    #[asset(key = "texture.loading_icon")]
    pub loading_icon: Handle<Image>,
}

// --- RON Data Structures ---
#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
pub struct SpeciesData {
    pub species_name: String,
    // Verwenden Sie den tatsächlichen Typ aus Ihrem attributes-Modul
    pub attribute_distributions:
        HashMap<attributes::AttributeType, attributes::AttributeDistribution>,
}

#[derive(serde::Deserialize, Asset, TypePath, Debug, Clone)]
pub struct UiThemeData {
    // Felder müssen exakt denen in default.theme.ron entsprechen!
    pub name: Option<String>,
    pub background: [f32; 4],
    pub foreground: [f32; 4],
    pub card: [f32; 4],
    pub card_foreground: [f32; 4],
    pub popover: [f32; 4],
    pub popover_foreground: [f32; 4],
    pub primary: [f32; 4],
    pub primary_foreground: [f32; 4],
    pub secondary: [f32; 4],
    pub secondary_foreground: [f32; 4],
    pub muted: [f32; 4],
    pub muted_foreground: [f32; 4],
    pub accent: [f32; 4],
    pub accent_foreground: [f32; 4],
    pub destructive: [f32; 4],
    pub destructive_foreground: [f32; 4],
    pub border: [f32; 4],
    pub input: [f32; 4],
    pub ring: [f32; 4],
    pub radius: f32,
    pub default_font_path: Option<String>,
}

// --- Theme-Setup-System ---
fn setup_theme_resource(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    theme_data_handles: Res<Assets<UiThemeData>>,
    game_assets: Res<GameAssets>, // GameAssets MUSS hier verfügbar sein
) {
    if let Some(theme_data) = theme_data_handles.get(&game_assets.theme_data) {
        let theme_font = theme_data
            .default_font_path
            .as_ref()
            // Lade Font Handle ODER nutze das bereits geladene Fallback
            .map_or_else(
                || {
                    info!("Theme has no font path, using fallback main_font.");
                    game_assets.main_font.clone() // <-- Fallback nutzen
                },
                |path| {
                    info!("Loading font from theme path: {}", path);
                    asset_server.load(path) // <-- Aus Theme laden
                },
            );

        let ui_theme = UiTheme {
            background: Color::srgba(
                theme_data.background[0],
                theme_data.background[1],
                theme_data.background[2],
                theme_data.background[3],
            ),
            foreground: Color::srgba(
                theme_data.foreground[0],
                theme_data.foreground[1],
                theme_data.foreground[2],
                theme_data.foreground[3],
            ),
            card: Color::srgba(
                theme_data.card[0],
                theme_data.card[1],
                theme_data.card[2],
                theme_data.card[3],
            ),
            card_foreground: Color::srgba(
                theme_data.card_foreground[0],
                theme_data.card_foreground[1],
                theme_data.card_foreground[2],
                theme_data.card_foreground[3],
            ),
            popover: Color::srgba(
                theme_data.popover[0],
                theme_data.popover[1],
                theme_data.popover[2],
                theme_data.popover[3],
            ),
            popover_foreground: Color::srgba(
                theme_data.popover_foreground[0],
                theme_data.popover_foreground[1],
                theme_data.popover_foreground[2],
                theme_data.popover_foreground[3],
            ),
            primary: Color::srgba(
                theme_data.primary[0],
                theme_data.primary[1],
                theme_data.primary[2],
                theme_data.primary[3],
            ),
            primary_foreground: Color::srgba(
                theme_data.primary_foreground[0],
                theme_data.primary_foreground[1],
                theme_data.primary_foreground[2],
                theme_data.primary_foreground[3],
            ),
            secondary: Color::srgba(
                theme_data.secondary[0],
                theme_data.secondary[1],
                theme_data.secondary[2],
                theme_data.secondary[3],
            ),
            secondary_foreground: Color::srgba(
                theme_data.secondary_foreground[0],
                theme_data.secondary_foreground[1],
                theme_data.secondary_foreground[2],
                theme_data.secondary_foreground[3],
            ),
            muted: Color::srgba(
                theme_data.muted[0],
                theme_data.muted[1],
                theme_data.muted[2],
                theme_data.muted[3],
            ),
            muted_foreground: Color::srgba(
                theme_data.muted_foreground[0],
                theme_data.muted_foreground[1],
                theme_data.muted_foreground[2],
                theme_data.muted_foreground[3],
            ),
            accent: Color::srgba(
                theme_data.accent[0],
                theme_data.accent[1],
                theme_data.accent[2],
                theme_data.accent[3],
            ),
            accent_foreground: Color::srgba(
                theme_data.accent_foreground[0],
                theme_data.accent_foreground[1],
                theme_data.accent_foreground[2],
                theme_data.accent_foreground[3],
            ),
            destructive: Color::srgba(
                theme_data.destructive[0],
                theme_data.destructive[1],
                theme_data.destructive[2],
                theme_data.destructive[3],
            ),
            destructive_foreground: Color::srgba(
                theme_data.destructive_foreground[0],
                theme_data.destructive_foreground[1],
                theme_data.destructive_foreground[2],
                theme_data.destructive_foreground[3],
            ),
            border: Color::srgba(
                theme_data.border[0],
                theme_data.border[1],
                theme_data.border[2],
                theme_data.border[3],
            ),
            input: Color::srgba(
                theme_data.input[0],
                theme_data.input[1],
                theme_data.input[2],
                theme_data.input[3],
            ),
            ring: Color::srgba(
                theme_data.ring[0],
                theme_data.ring[1],
                theme_data.ring[2],
                theme_data.ring[3],
            ),
            radius: Val::Px(theme_data.radius),
            default_font: Some(theme_font), // Setze das geladene Handle
        };

        commands.insert_resource(ui_theme);
        info!("UiTheme resource created and inserted from theme data.");
    } else {
        error!("UiThemeData asset not found AFTER loading state. Cannot create UiTheme resource! Check asset path and loading logic.");
        // Eventuell hier Default Theme einfügen, falls das oft passiert
        // commands.insert_resource(UiTheme::default());
        // warn!("Inserted default fallback UiTheme.");
    }
}

// --- UI Setup System ---
// Runs once when AppState::MainMenu is entered, AFTER setup_theme_resource
fn setup_main_menu(
    mut commands: Commands,
    assets: Res<GameAssets>,
    theme: Res<UiTheme>, // Theme Ressource wird hier benötigt
) {
    // Spawn Camera
    commands.spawn(Camera2d::default());

    // Get the font handle (either from theme or fallback)
    let font_handle = theme
        .default_font
        .clone()
        .unwrap_or_else(|| assets.main_font.clone()); // Fallback, falls Theme keine Font hat

    // Create root UI node
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(theme.background), // Hintergrund aus Theme
        ))
        .with_children(|parent| {
            // --- Button Gruppe (wie bisher) ---
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

                    // Default Button
                    let _ = ButtonBuilder::new()
                        .with_text("Play Game")
                        .size(ButtonSize::Large)
                        .spawn(button_parent, font_handle.clone(), &theme); // <<< Pass theme

                    // Destructive Button with Icon
                    let _ = ButtonBuilder::new()
                        .variant(ButtonVariant::Destructive)
                        .with_icon(assets.delete_icon.clone())
                        .with_text("Delete Save")
                        .add_marker(|cmd| {
                            cmd.insert(DeleteSaveButton);
                        })
                        .spawn(button_parent, font_handle.clone(), &theme); // <<< Pass theme

                    // Outline Button with Callback
                    let _ = ButtonBuilder::new()
                        .variant(ButtonVariant::Outline)
                        .with_text("Options")
                        .on_click(|| {
                            println!("Options button clicked (direct callback)!");
                        })
                        .spawn(button_parent, font_handle.clone(), &theme); // <<< Pass theme

                    // Icon-only button
                    let _ = ButtonBuilder::new()
                        .size(ButtonSize::Icon)
                        .variant(ButtonVariant::Secondary)
                        .with_icon(assets.settings_icon.clone())
                        .add_marker(|cmd| {
                            cmd.insert(SettingsButton);
                        })
                        .spawn(button_parent, font_handle.clone(), &theme); // <<< Pass theme

                    // Disabled button
                    let _ = ButtonBuilder::new()
                        .with_text("Continue (Disabled)")
                        .disabled(true)
                        .spawn(button_parent, font_handle.clone(), &theme); // <<< Pass theme
                });
            // --- Beispiel-Karte mit Builder ---
            let _ = CardBuilder::new() // << CardBuilder starten
                .width(Val::Px(380.0)) // Beispiel: Breite setzen
                .with_header(vec![
                    // << Header definieren
                    NodeElement::Text {
                        content: "Notifications".to_string(),
                        style: ElementStyle::Title,
                        font_size: None, // Verwendet Default-Größe für Title
                    },
                    NodeElement::Text {
                        content: "You have 3 unread messages.".to_string(),
                        style: ElementStyle::Description,
                        font_size: None,
                    },
                ])
                .with_content(vec![
                    // << Content definieren
                    // Einfacher Text als Beispiel
                    NodeElement::Text {
                        content: "Main card content area.".to_string(),
                        style: ElementStyle::Normal,
                        font_size: None,
                    },
                    // Button direkt im Content einfügen
                    NodeElement::Button(
                        ButtonBuilder::new()
                            .variant(ButtonVariant::Secondary)
                            .with_icon(assets.settings_icon.clone()) // Beispiel-Icon verwenden
                            .with_text("Manage Settings")
                            .size(ButtonSize::Small),
                    ),
                ])
                .with_footer(vec![
                    // << Footer definieren
                    // Button direkt im Footer einfügen
                    NodeElement::Button(ButtonBuilder::new().with_text("Cancel")),
                    NodeElement::Button(
                        ButtonBuilder::new()
                            // Kein Variant -> Default-Button
                            .with_text("Confirm"),
                    ),
                ])
                .spawn(parent, &theme, &font_handle);
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
