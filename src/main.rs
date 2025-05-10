// src/main.rs
use bevy::prelude::*;

use forge_ui::*;

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

// Eigene Aktions-Enum
#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub enum MyGameAction {
    StartGame,
    OpenSettings,
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
        .add_event::<ButtonClickedEvent<MyGameAction>>()
        .add_systems(
            Update,
            (
                handle_button_press::<MyGameAction>.run_if(in_state(AppState::MainMenu)), // nur in MainMenu
                handle_button_clicks.run_if(in_state(AppState::MainMenu)), // nur in MainMenu
            ),
        )
        .run();
}

// --- UI Setup System ---
// Runs once when AppState::MainMenu is entered, AFTER setup_theme_resource
fn setup_main_menu(
    mut commands: Commands,
    theme: Res<UiTheme>, // Theme Ressource wird hier benötigt
    icons: Res<IconAssets>,
    global_portal_root_opt: Option<Res<ForgeUiPortalRoot>>,
) {
    // Spawn Camera
    commands.spawn(Camera2d::default());

    // Get the font handle (either from theme or fallback)
    let font_handle = theme.font.font_family.default.clone(); // Fallback, falls Theme keine Font hat

    // Create root UI nodre
    let ui_root_entity = UiRoot::spawn(&mut commands, &theme);
    let dialog_id = DialogId::new_unique();

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

                let _ = ButtonBuilder::<MyGameAction>::new_for_action()
                    .text("Spiel starten")
                    .variant(ButtonVariant::Default) // Annahme: Primary ist eine Variante
                    .action(MyGameAction::StartGame)
                    .spawn(button_parent, &theme, &font_handle);
                let _ = DialogTriggerBuilder::new(dialog_id.clone())
                    .text("Dialog öffnen")
                    .variant(ButtonVariant::Default)
                    .spawn(button_parent, &theme, &font_handle);
            });
    });
    let my_header = DialogHeaderBuilder::new()
        .title("Wichtige Entscheidung")
        .subtitle("Bitte wähle eine Option.");

    let my_body = DialogBodyBuilder::new()
        .add_content(|parent, theme, font_handle| {
            parent.spawn((
                Text::new("Dies ist der Haupttext des Dialogs. Überlege gut!"),
                TextFont {
                    font: font_handle.clone(),
                    font_size: theme.font.font_size.base,
                    ..default()
                },
                TextColor(theme.color.gray.step12),
            ));
        })
        .add_content(|parent, theme, font_handle| {
            // Beispiel: Button im Body hinzufügen
            let _ = ButtonBuilder::<NoAction>::new().text("Zusatzinfo").spawn(
                parent,
                theme,
                font_handle,
            );
        });

    let my_footer = DialogFooterBuilder::new()
        .justify_content(JustifyContent::SpaceBetween) // Beispiel: Buttons verteilen
        .add_custom_content(move |parent, theme, font_handle| {
            let _ = ButtonBuilder::<DialogAction>::new_for_action()
                .text("Abbrechen")
                .action(DialogAction::Close(dialog_id.clone()))
                .variant(ButtonVariant::Outline)
                .spawn(parent, theme, font_handle);
        })
        .add_custom_content(move |parent, theme, font_handle| {
            let _ = ButtonBuilder::<DialogAction>::new_for_action() // Hier ggf. eigene Action
                .text("Bestätigen")
                .action(DialogAction::Close(dialog_id.clone()))
                .spawn(parent, theme, font_handle);
        });
    // Einzigartige ID für den Dialog
    // Dann den DialogBuilder verwenden
    let _ = DialogBuilder::new(dialog_id)
        .header(my_header)
        .body(my_body)
        .footer(my_footer)
        .width(Val::Px(400.0)) // Dialog-spezifische Einstellungen
        .spawn(
            &mut commands,
            &theme,
            &font_handle,
            global_portal_root_opt,
            Some(icons.cross_1.clone()),
        );

    info!("Main menu UI setup complete.");
}

fn handle_button_clicks(mut events: EventReader<ButtonClickedEvent<MyGameAction>>) {
    for event in events.read() {
        if let Some(action) = &event.action_id {
            match action {
                MyGameAction::StartGame => {
                    info!("Button zum Starten des Spiels geklickt!");
                }
                MyGameAction::OpenSettings => {
                    info!("Button zum Öffnen der Einstellungen geklickt!");
                }
            }
        }
    }
}
