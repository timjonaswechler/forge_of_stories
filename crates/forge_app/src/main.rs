// ############ crates/forge_app/src/main.rs (NEU) ############
use bevy::prelude::*; // <-- Import für RonAssetPlugin von Bevy selbst
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin; // <-- Import für RonAssetPlugin
use std::collections::HashMap; // <-- Import für HashMap

use forge_of_stories::*; // Importieren Sie Ihr attributes Modul
                         // Ihr attributes Modul
use forge_ui::{
    badge::{BadgeBuilder, BadgeVariant}, // Importieren Sie den TabBuilder

    card::{CardBuilder, ElementStyle, NodeElement},
    checkbox::{
        handle_checkbox_clicks, update_checkbox_visuals,
        update_checkmark_visibility_on_state_change,
    },
    checkbox::{CheckboxBuilder, CheckboxChangedEvent}, // << Builder und Helfer importieren
    components::button::{
        handle_button_clicks_event, update_button_visuals, ButtonBuilder, ButtonClickedEvent,
        ButtonSize, ButtonVariant,
    },
    dialog::{DialogBuilder, DialogCloseTrigger, DialogId, OpenDialogEvent},
    label::LabelBuilder,
    layout::UiRoot,
    tabs::handle_tab_triggers,
    tabs::{TabId, TabsBuilder},
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
        .add_plugins(ForgeUiPlugin) // Add the UI plugin HERE
        // --- Game Setup & Systems ---
        .add_systems(
            OnEnter(AppState::MainMenu),
            (
                apply_deferred, // Stellt sicher, dass Ressource vor nächstem System da ist
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
                // Checkbox Systeme
                update_checkbox_visuals, // Den umbenannten Systemnamen verwenden, falls update_button_visuals auch so heißt
                handle_checkbox_clicks,
                update_checkmark_visibility_on_state_change,
                // Handler aus der App
                handle_button_clicks,         // <- Ihr Button-Event-Handler
                handle_checkbox_changes,      // <- Ihr Checkbox-Event-Handler
                handle_tab_triggers::<TabId>, // <- Ihr Tab-Event-Handler
                handle_dialog_trigger_buttons,
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
    // #[asset(key = "font.main")] pub main_font: Handle<Font>, // Wenn über Key geladen
    #[asset(path = "fonts/Roboto-Regular.ttf")]
    pub main_font: Handle<Font>, // Fallback, wenn nicht im Theme/Key

    #[asset(key = "icon.settings")]
    pub settings_icon: Handle<Image>,
    #[asset(key = "icon.delete")]
    pub delete_icon: Handle<Image>,
    #[asset(key = "icon.checkmark")]
    pub checkmark_icon: Handle<Image>,

    #[asset(key = "species.elf", typed)] // 'typed' wichtig für RonAssetPlugin via Key
    pub elf_species: Handle<SpeciesData>,
    #[asset(key = "species.human", typed)]
    pub human_species: Handle<SpeciesData>,
    #[asset(key = "species.ork", typed)]
    pub ork_species: Handle<SpeciesData>,

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
    let font_handle = theme.font.font_family.default.clone(); // Fallback, falls Theme keine Font hat

    // --- Trigger-Button für den Dialog ---
    let profile_dialog_id = DialogId::new_unique();
    let profile_dialog_id_for_button = profile_dialog_id.clone(); // ID für den Dialog

    // Create root UI nodre
    let ui_root_entity = UiRoot::spawn(&mut commands, &theme);

    commands.entity(ui_root_entity).with_children(|parent| {
        // --- Badges Gruppe ---
        parent
            .spawn(Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center, // Badges zentrieren
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0), // Abstand zwischen Badges
                margin: UiRect::top(Val::Px(20.0)),

                ..default()
            })
            .with_children(|badge_row| {
                // Verschiedene Badge-Varianten erstellen
                let _ = BadgeBuilder::new("Default").spawn(badge_row, &theme, &font_handle);

                let _ = BadgeBuilder::new("Secondary")
                    .variant(BadgeVariant::Secondary)
                    .spawn(badge_row, &theme, &font_handle);

                let _ = BadgeBuilder::new("Outline")
                    .variant(BadgeVariant::Outline)
                    .spawn(badge_row, &theme, &font_handle);

                let _ = BadgeBuilder::new("Destructive")
                    .variant(BadgeVariant::Destructive)
                    .spawn(badge_row, &theme, &font_handle);
            }); // Ende Badge Row Children
                // --- Badges ---

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
                    .size(ButtonSize::Default)
                    .border_radius(theme.layout.radius.xs) // Beispiel: BorderRadius aus Theme
                    .spawn(button_parent, &theme, &font_handle); // <<< Pass theme

                // Destructive Button with Icon
                let _ = ButtonBuilder::new()
                    .variant(ButtonVariant::Destructive)
                    .with_icon(assets.delete_icon.clone())
                    .with_text("Delete Save")
                    .add_marker(|cmd| {
                        cmd.insert(DeleteSaveButton);
                    })
                    .spawn(button_parent, &theme, &font_handle); // <<< Pass theme

                // Outline Button with Callback
                let _ = ButtonBuilder::new()
                    .variant(ButtonVariant::Outline)
                    .with_text("Options")
                    .on_click(|| {
                        println!("Options button clicked (direct callback)!");
                    })
                    .spawn(button_parent, &theme, &font_handle); // <<< Pass theme

                // Icon-only button
                let _ = ButtonBuilder::new()
                    .size(ButtonSize::Icon)
                    .variant(ButtonVariant::Secondary)
                    .with_icon(assets.settings_icon.clone())
                    .add_marker(|cmd| {
                        cmd.insert(SettingsButton);
                    })
                    .spawn(button_parent, &theme, &font_handle); // <<< Pass theme

                // Disabled button
                let _ = ButtonBuilder::new()
                    .with_text("Continue (Disabled)")
                    .disabled(true)
                    .spawn(button_parent, &theme, &font_handle); // <<< Pass theme

                // <<< Pass theme
            });
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
                NodeElement::Button(
                    ButtonBuilder::new()
                        .variant(ButtonVariant::Outline)
                        .with_text("Cancel"),
                ),
                NodeElement::Button(
                    ButtonBuilder::new()
                        // Kein Variant -> Default-Button
                        .with_text("Confirm"),
                ),
            ])
            .spawn(parent, &theme, &font_handle);

        // --- Checkbox-Beispiel mit Label ---
        // Container für Checkbox + Label nebeneinander
        parent
            .spawn(Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row, // Nebeneinander
                align_items: AlignItems::Center,    // Vertikal zentrieren
                column_gap: Val::Px(8.0),           // Abstand dazwischen
                margin: UiRect::top(Val::Px(20.0)), // Etwas Abstand nach oben

                ..default()
            })
            .with_children(|cb_row| {
                // Checkbox spawnen
                let _ = CheckboxBuilder::new()
                    .checked(true) // Startet ausgewählt
                    .spawn(cb_row, &theme, &assets.checkmark_icon)
                    // Marker für spezifische Reaktion
                    .insert(TermsCheckbox) // Eigener Marker
                    .id(); // Die Entity ID speichern, falls wir sie brauchen

                // Zugehöriges Label spawnen
                let _ = LabelBuilder::new("Accept terms and conditions")
                    // Optional: Verknüpfung (in Bevy nicht direkt wie htmlFor, aber als Hinweis)
                    // .id("terms-label") // Weniger sinnvoll in Bevy
                    .spawn(cb_row, &theme, &font_handle);

                // -- Beispiel für disabled Checkbox --
                let _ = CheckboxBuilder::new().disabled(true).spawn(
                    cb_row,
                    &theme,
                    &assets.checkmark_icon,
                );
                let _ = LabelBuilder::new("Disabled Checkbox")
                    .color(theme.color.gray.border_secondary) // Ausgegraut
                    .spawn(cb_row, &theme, &font_handle);
            });
        parent
            .spawn(Node {
                margin: UiRect::top(Val::Px(20.0)),
                // Breite für das Tabs-Widget festlegen
                width: Val::Px(400.0),

                ..default()
            })
            .with_children(|tabs_parent| {
                // Tabs mit String-Werten erstellen

                let _ = TabsBuilder::<TabId>::new(TabId("account".to_string())) // Default-Tab 'account'
                    .add_tab(
                        TabId("account".to_string()), // Wert dieses Tabs
                        "Account",                    // Label für den Trigger
                        |content_parent, theme, font_handle| {
                            // Closure für den Content
                            // Inhalt für den Account-Tab (z.B. eine Karte)
                            let _ = CardBuilder::new()
                                .with_header(vec![
                                    NodeElement::Text {
                                        content: "Account Settings".into(),
                                        style: ElementStyle::Title,
                                        font_size: None,
                                    },
                                    NodeElement::Text {
                                        content: "Manage your account details.".into(),
                                        style: ElementStyle::Description,
                                        font_size: None,
                                    },
                                ])
                                .with_content(vec![
                                    NodeElement::Text {
                                        content: "Username: Placeholder".into(),
                                        style: ElementStyle::Normal,
                                        font_size: None,
                                    },
                                    // Hier könnten Inputs etc. hin
                                ])
                                .with_footer(vec![NodeElement::Button(
                                    ButtonBuilder::new().with_text("Save Account"),
                                )])
                                .spawn(content_parent, theme, font_handle);
                        },
                    )
                    .add_tab(
                        TabId("password".to_string()), // Wert dieses Tabs
                        "Password",                    // Label für den Trigger
                        |content_parent, theme, font_handle| {
                            // Closure für den Content
                            // Inhalt für den Password-Tab
                            let _ = CardBuilder::new()
                                .with_header(vec![NodeElement::Text {
                                    content: "Change Password".into(),
                                    style: ElementStyle::Title,
                                    font_size: None,
                                }])
                                .with_content(vec![
                                    NodeElement::Text {
                                        content: "Current Password: ***".into(),
                                        style: ElementStyle::Normal,
                                        font_size: None,
                                    },
                                    NodeElement::Text {
                                        content: "New Password: ***".into(),
                                        style: ElementStyle::Normal,
                                        font_size: None,
                                    },
                                ])
                                .with_footer(vec![NodeElement::Button(
                                    ButtonBuilder::new().with_text("Save Password"),
                                )])
                                .spawn(content_parent, theme, font_handle);
                        },
                    )
                    .add_disabled_tab(
                        TabId("security".to_string()), // Wert des deaktivierten Tabs
                        "Security",                    // Label des deaktivierten Tabs
                    )
                    .spawn(tabs_parent, &theme, &font_handle); // Das ganze Tabs-Widget spawnen
            }); // Ende Tabs Parent Children

        // Eindeutige ID erstellen
        let _ = ButtonBuilder::new()
            .with_text("Open Profile")
            .add_marker(move |cmd| {
                // Closure, um ID und Event zu verknüpfen
                cmd.insert(OpenProfileButton {
                    dialog_id_to_open: profile_dialog_id_for_button,
                });
            })
            .spawn(parent, &theme, &font_handle);
    });
    // --- Dialog spawnen (auf oberster Ebene, NICHT in main_ui_parent!) ---
    // let profile_dialog_id = DialogId::new_unique(); // Dieselbe ID verwenden oder neu generieren und im Button speichern
    let dialog_font_handle = font_handle.clone();
    let _ = DialogBuilder::new(profile_dialog_id.clone()) // Verwende die vorher erstellte ID
        .title("Edit Profile")
        .description("Make changes to your profile here. Click save when you're done.")
        .width(Val::Px(450.0))
        // Default close button verwenden (X oben rechts)
        .with_content(|content_builder, theme, font_handle| {
            // Inhalt des Dialogs definieren
            // Beispiel: Label + Text (Input fehlt noch)
            let _ = LabelBuilder::new("Name:").spawn(content_builder, theme, font_handle);
            // TODO: Input-Feld ersetzen
            content_builder.spawn((
                Node {
                    width: Val::Percent(100.),
                    height: Val::Px(30.),
                    border: UiRect::all(Val::Px(1.)),
                    ..default()
                },
                BorderColor(theme.color.gray.border_primary), // Beispiel-Farbe
            ));

            let _ = LabelBuilder::new("Username:").spawn(content_builder, theme, font_handle);
            content_builder.spawn((
                Node {
                    width: Val::Percent(100.),
                    height: Val::Px(30.),
                    border: UiRect::all(Val::Px(1.)),
                    ..default()
                },
                BorderColor(theme.color.gray.border_primary),
            ));

            // Beispiel: Eigener Schließen-Button im Footer
            content_builder
                .spawn(Node {
                    justify_content: JustifyContent::FlexEnd,
                    width: Val::Percent(100.),
                    margin: UiRect::top(Val::Px(20.)),

                    ..default()
                })
                .with_children(|footer| {
                    let _ = ButtonBuilder::new()
                        .with_text("Save Changes")
                        .add_marker(|cmd| {
                            cmd.insert(DialogCloseTrigger);
                        }) // <-- Schließt Dialog
                        .spawn(footer, theme, &font_handle);
                });
        })
        .spawn(
            &mut commands,
            &theme,
            &dialog_font_handle,
            Some(&assets.checkmark_icon.clone()),
            Some(ui_root_entity),
        );

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

// --- Marker für Checkbox ---
#[derive(Component)]
struct TermsCheckbox;

// --- Event Handler für Checkbox-Änderungen ---
fn handle_checkbox_changes(
    mut events: EventReader<CheckboxChangedEvent>,
    terms_query: Query<(), With<TermsCheckbox>>, // Prüfen, ob es unsere Terms-Checkbox ist
) {
    for event in events.read() {
        if terms_query.get(event.checkbox_entity).is_ok() {
            info!("Terms checkbox changed! New state: {}", event.is_checked);
            // Hier Logik ausführen, z.B. einen "Weiter"-Button aktivieren/deaktivieren
        } else {
            info!(
                "Another checkbox (Entity {:?}) changed to {}",
                event.checkbox_entity, event.is_checked
            );
        }
    }
}

// --- Marker + Zustand für den Trigger-Button ---
#[derive(Component)]
struct OpenProfileButton {
    dialog_id_to_open: DialogId,
}

// --- System, das auf den Trigger-Button reagiert ---
fn handle_dialog_trigger_buttons(
    mut interactions: Query<
        (&Interaction, &OpenProfileButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut ev_open_dialog: EventWriter<OpenDialogEvent>,
) {
    for (interaction, button_data) in interactions.iter_mut() {
        if *interaction == Interaction::Pressed {
            info!("Opening dialog: {:?}", button_data.dialog_id_to_open);
            ev_open_dialog.send(OpenDialogEvent(button_data.dialog_id_to_open.clone()));
        }
    }
}
