// src/main.rs
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use forge_ui::prelude::*;
use forge_ui::showcase::plugin::ShowcasePlugin;

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
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(WorldInspectorPlugin::new())
        .init_state::<AppState>()
        // --- UI Plugin mit eigener UiState-Zustandsmaschine ---
        .add_plugins(ForgeUiPlugin::new())
        .add_plugins(ShowcasePlugin)
        .add_systems(
            Update,
            transition_to_main_menu_when_ui_ready.run_if(in_state(AppState::Loading)), // nur solange wir noch laden
        )
        // --- Wenn wir in MainMenu landen, baue das UI auf ---
        .add_systems(
            OnEnter(AppState::MainMenu),
            setup_main_menu.run_if(|res: Option<Res<UiTheme>>| resource_exists(res)), // optional: warte auf Theme
        )
        .add_event::<ButtonClickedEvent<MyGameAction>>()
        .add_event::<ToggleGroupChangedEvent<SwitchMode>>()
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
fn setup_main_menu(mut commands: Commands, theme: Res<UiTheme>, font: Res<FontAssets>) {
    let font_handle = theme.font.family.default.clone();
    let ui_root_entity = UiRoot::spawn(&mut commands, &theme);

    commands
        .entity(ui_root_entity)
        .insert(Name::new("Main Menu UI Root"))
        .with_children(|parent| {
            // Wichtig: Zuerst den Entity-Wert des Vertical Stacks erhalten
            let vertical_stack_entity = VerticalStackBuilder::new("Main Menu Stack")
                .gap(Val::Px(theme.layout.gap.sm))
                .spawn(parent)
                .id(); // <-- Hier .id() verwenden, um die Entity-ID zu bekommen!

            // Dann die Kinder zum Stack hinzufügen
            parent
                .commands()
                .entity(vertical_stack_entity)
                .with_children(|stack_parent| {
                    LabelBuilder::new("Main Menu Controls")
                        .color(theme.color.gray.step11)
                        .align(JustifyText::Center)
                        .spawn(stack_parent, &theme, &font_handle);

                    ButtonBuilder::<MyGameAction>::new_for_action()
                        .text("Spiel starten")
                        .disabled(true)
                        .variant(ButtonVariant::Solid)
                        .color(theme.color.lime.clone())
                        .action(MyGameAction::StartGame)
                        .spawn(stack_parent, &theme, &font.default);
                });
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
