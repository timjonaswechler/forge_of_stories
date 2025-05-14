// src/main.rs
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use forge_ui::prelude::*;

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

/// Wir speichern einfach eine numerische ID in der Action
#[derive(Component, Clone, Debug)]
pub struct MyActionComponent {
    pub id: usize,
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
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(WorldInspectorPlugin::new())
        .init_state::<AppState>()
        // --- UI Plugin mit eigener UiState-Zustandsmaschine ---
        .add_plugins(ForgeUiPlugin::new()) // --- Brücke zwischen UiState und AppState ---
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
        .add_event::<ToggleGroupChangedEvent<SwitchMode>>()
        .add_systems(
            Update,
            (
                handle_button_press::<MyGameAction>.run_if(in_state(AppState::MainMenu)), // nur in MainMenu
                handle_button_clicks.run_if(in_state(AppState::MainMenu)), // nur in MainMenu
                handle_group_changed.run_if(in_state(AppState::MainMenu)), // nur in MainMenu
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
    // Get the font handle (either from theme or fallback)
    let font_handle = theme.font.font_family.default.clone(); // Fallback, falls Theme keine Font hat

    // Create root UI nodre
    let ui_root_entity = UiRoot::spawn(&mut commands, &theme);
    let dialog_id = DialogId::new_unique();

    commands.entity(ui_root_entity).with_children(|parent| {
        // --- Button Gruppe ---
        let _ = VerticalStackBuilder::new()
            .add_entity(
                LabelBuilder::new("Main Menu Controls")
                    .color(theme.color.gray.step11) // Andere Farbe zum Testen
                    .align(JustifyText::Center) // Zentrieren
                    .spawn(parent, &theme, &font_handle),
            )
            .add_entity(
                ButtonBuilder::<MyGameAction>::new_for_action()
                    .text("Spiel starten")
                    .variant(ButtonVariant::Default) // Annahme: Primary ist eine Variante
                    .action(MyGameAction::StartGame)
                    .spawn(parent, &theme, &font_handle)
                    .id(),
            )
            .add_entity(
                DialogTriggerBuilder::new(dialog_id.clone())
                    .text("Dialog öffnen")
                    .variant(ButtonVariant::Default)
                    .spawn(parent, &theme, &font_handle)
                    .id(),
            )
            .gap(Val::Px(theme.layout.gap.sm))
            .spawn(parent);
        ToggleBuilder::<MyActionComponent>::new_with_action_type()
            .variant(ToggleVariant::Primary)
            .action(MyActionComponent { id: 42 })
            .icon(icons.align_left.clone())
            .spawn_into(parent, &theme);

        // let my_header = DialogHeaderBuilder::new()
        //     .title("Wichtige Entscheidung")
        //     .subtitle("Bitte wähle eine Option.")
        //     .with_close_button(icons.cross_1.clone());

        // let my_body = DialogBodyBuilder::new()
        //     .add_content(|parent, theme, font_handle| {
        //         parent.spawn((
        //             Text::new("Dies ist der Haupttext des Dialogs. Überlege gut!"),
        //             TextFont {
        //                 font: font_handle.clone(),
        //                 font_size: theme.font.font_size.base,
        //                 ..default()
        //             },
        //             TextColor(theme.color.gray.step12),
        //         ));
        //     })
        //     .add_content(|parent, theme, font_handle| {
        //         // Beispiel: Button im Body hinzufügen
        //         let _ = ButtonBuilder::<NoAction>::new().text("Zusatzinfo").spawn(
        //             parent,
        //             theme,
        //             font_handle,
        //         );
        //     });

        // let my_footer = DialogFooterBuilder::new()
        //     .justify_content(JustifyContent::SpaceBetween) // Beispiel: Buttons verteilen
        //     .add_custom_content(move |parent, theme, font_handle| {
        //         let _ = ButtonBuilder::<DialogAction>::new_for_action()
        //             .text("Abbrechen")
        //             .action(DialogAction::Close(dialog_id.clone()))
        //             .variant(ButtonVariant::Outline)
        //             .spawn(parent, theme, font_handle);
        //     })
        //     .add_custom_content(move |parent, theme, font_handle| {
        //         let _ = ButtonBuilder::<DialogAction>::new_for_action() // Hier ggf. eigene Action
        //             .text("Bestätigen")
        //             .action(DialogAction::Close(dialog_id.clone()))
        //             .spawn(parent, theme, font_handle);
        //     });
        // let dialog_content = DialogContentBuilder::new()
        //     .header(my_header)
        //     .body(my_body)
        //     .footer(my_footer);
        // // Einzigartige ID für den Dialog
        // // Dann den DialogBuilder verwenden
        // let _ = DialogBuilder::new(dialog_id)
        //     .content(dialog_content)
        //     .width(Val::Px(400.0)) // Dialog-spezifische Einstellungen
        //     .spawn(
        //         &mut parent.commands(),
        //         &theme,
        //         &font_handle,
        //         global_portal_root_opt,
        //     );
    });
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

#[derive(Debug, Clone)]
pub enum Mode {
    Light,
    Dark,
}

#[derive(Component, Clone, Debug)]
pub struct SwitchMode(pub Mode);

fn handle_group_changed(mut events: EventReader<ToggleGroupChangedEvent<SwitchMode>>) {
    for evt in events.read() {
        if let Some(SwitchMode(mode)) = &evt.action_id {
            info!(
                "Gruppe {:?} schaltet auf {:?}, active_values={:?}",
                evt.source_entity, mode, evt.active_values
            );
        }
    }
}
