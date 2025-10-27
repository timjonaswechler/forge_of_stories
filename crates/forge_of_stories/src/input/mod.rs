use crate::{
    GameState,
    ui::{
        cameras::{CameraMode, CameraModeChangeEvent, InGameCameraMode},
        components::InGameMenuState,
    },
};
use anyhow::Result;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use keymap::{
    ActionDescriptor, ActionId, BindingDescriptor, BindingInputDescriptor, KeyBindingMetaIndex,
    KeymapSpec, KeymapStore,
    enhanced::{ConversionError, binding_descriptor_to_binding},
    parse_keystroke_sequence,
};
use paths::PathContext;
use tracing::warn;

// ============================================================================
// Action Definitions
// ============================================================================

/// Toggle the in-game menu
#[derive(Debug, Component, InputAction)]
#[action_output(bool)]
struct ToggleMenu;

/// Switch camera to Pan/Orbit mode
#[derive(Debug, Component, InputAction)]
#[action_output(bool)]
struct SwitchToPanOrbit;

/// Switch camera to First Person mode
#[derive(Debug, Component, InputAction)]
#[action_output(bool)]
struct SwitchToFirstPerson;

/// Resource wrapper around [`KeymapStore`] so Bevy can manage it.
#[derive(Resource)]
pub struct KeymapStoreResource(KeymapStore);

impl KeymapStoreResource {
    pub fn new(store: KeymapStore) -> Self {
        Self(store)
    }
}

impl std::ops::Deref for KeymapStoreResource {
    type Target = KeymapStore;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Extract and convert bindings for a specific action from the keymap store.
fn extract_bindings_for_action(
    store: &KeymapStore,
    action_id: &str,
) -> Vec<bevy_enhanced_input::binding::Binding> {
    let mut bindings = Vec::new();

    store.with_spec(|spec| {
        for descriptor in &spec.bindings {
            if let Some(id) = &descriptor.action_id {
                if id.as_str() == action_id {
                    match binding_descriptor_to_binding(descriptor) {
                        Ok(Some(binding)) => bindings.push(binding),
                        Ok(None) => {}
                        Err(err) => {
                            warn!("Failed to convert binding for {}: {}", action_id, err);
                        }
                    }
                }
            }
        }
    });

    bindings
}

/// Build a keymap store using the default descriptors bundled with the game.
///
/// The store is configured to read user overrides from the project's keybinding file
/// (see [`PathContext::keybinding_file`]). Invalid user files are ignored with a warning.
pub fn create_keymap_store(path_context: &PathContext) -> Result<KeymapStore> {
    let spec = default_keymap_spec();
    let keymap_path = path_context.keybinding_file();

    let store = KeymapStore::builder()
        .with_default_spec(spec)
        .with_user_keymap_path(keymap_path)
        .build()?;

    if let Err(err) = store.load_user_bindings() {
        warn!("Failed to load user keymap overrides: {err:#}");
    }

    Ok(store)
}

// ============================================================================
// Plugin
// ============================================================================

/// Marker component for the global input context.
#[derive(Component)]
struct GlobalInputContext;

pub struct KeymapInputPlugin;

impl Plugin for KeymapInputPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register the global input context
            .add_input_context::<GlobalInputContext>()
            // Setup systems
            .add_systems(Startup, (log_loaded_keymap, spawn_global_input_context))
            // Observers for input actions
            .add_observer(handle_toggle_menu)
            .add_observer(handle_switch_to_pan_orbit)
            .add_observer(handle_switch_to_first_person);
    }
}

fn log_loaded_keymap(store: Res<KeymapStoreResource>) {
    store.with_spec(|spec| {
        info!(
            "Loaded keymap with {} actions, {} contexts, {} bindings",
            spec.actions.len(),
            spec.contexts.len(),
            spec.bindings.len()
        );

        for descriptor in &spec.bindings {
            let action = descriptor
                .action_id
                .as_ref()
                .map(|id| id.as_str())
                .unwrap_or("<disabled>");
            let context = descriptor
                .context_id
                .as_ref()
                .map(|id| id.as_str())
                .unwrap_or("<global>");

            match binding_descriptor_to_binding(descriptor) {
                Ok(Some(binding)) => {
                    info!("  [{context}] {action} => {binding:?}");
                }
                Ok(None) => {
                    info!("  [{context}] {action} => (no binding)");
                }
                Err(err) => log_conversion_error(context, action, err),
            }
        }
    });
}

fn log_conversion_error(context: &str, action: &str, err: ConversionError) {
    warn!("  [{context}] {action} => conversion failed: {err}");
}

// ============================================================================
// Systems
// ============================================================================

/// Spawn the global input context with actions loaded from the keymap.
fn spawn_global_input_context(mut commands: Commands, store: Res<KeymapStoreResource>) {
    // Extract bindings for all actions from the keymap
    let toggle_menu_bindings = extract_bindings_for_action(&store, "ui::toggle_menu");
    let switch_to_pan_orbit_bindings =
        extract_bindings_for_action(&store, "camera::switch_to_pan_orbit");
    let switch_to_first_person_bindings =
        extract_bindings_for_action(&store, "camera::switch_to_first_person");

    info!(
        "Spawning global input context with {} toggle_menu, {} pan_orbit, {} first_person bindings",
        toggle_menu_bindings.len(),
        switch_to_pan_orbit_bindings.len(),
        switch_to_first_person_bindings.len()
    );

    // Spawn the global input context with all actions
    commands.spawn((
        GlobalInputContext,
        actions!(GlobalInputContext[
            (
                Action::<ToggleMenu>::new(),
                Bindings::spawn(toggle_menu_bindings),
            ),
            (
                Action::<SwitchToPanOrbit>::new(),
                Bindings::spawn(switch_to_pan_orbit_bindings),
            ),
            (
                Action::<SwitchToFirstPerson>::new(),
                Bindings::spawn(switch_to_first_person_bindings),
            ),
        ]),
    ));
}

/// Observer that handles the ToggleMenu action.
/// Triggers only once when the button is first pressed (not continuously while held).
fn handle_toggle_menu(
    _trigger: On<Start<ToggleMenu>>,
    mut menu: ResMut<InGameMenuState>,
    game_state: Res<State<GameState>>,
) {
    // Only toggle menu when in-game
    if *game_state.get() == GameState::InGame {
        menu.toggle();
        info!("Menu toggled via enhanced input");
    }
}

/// Observer that handles switching to Pan/Orbit camera mode.
fn handle_switch_to_pan_orbit(
    _trigger: On<Start<SwitchToPanOrbit>>,
    current_mode: Res<CameraMode>,
    mut events: MessageWriter<CameraModeChangeEvent>,
    game_state: Res<State<GameState>>,
) {
    if *game_state.get() != GameState::InGame {
        return;
    }

    // Only switch if not already in PanOrbit mode
    if !matches!(
        *current_mode,
        CameraMode::InGame(InGameCameraMode::PanOrbit)
    ) {
        events.write(CameraModeChangeEvent {
            new_mode: CameraMode::InGame(InGameCameraMode::PanOrbit),
            animate: true,
        });
        info!("Switching to Pan/Orbit camera via enhanced input");
    }
}

/// Observer that handles switching to First Person camera mode.
fn handle_switch_to_first_person(
    _trigger: On<Start<SwitchToFirstPerson>>,
    current_mode: Res<CameraMode>,
    mut events: MessageWriter<CameraModeChangeEvent>,
    game_state: Res<State<GameState>>,
) {
    if *game_state.get() != GameState::InGame {
        return;
    }

    // Only switch if not already in FirstPerson mode
    if !matches!(
        *current_mode,
        CameraMode::InGame(InGameCameraMode::FirstPerson)
    ) {
        events.write(CameraModeChangeEvent {
            new_mode: CameraMode::InGame(InGameCameraMode::FirstPerson),
            animate: true,
        });
        info!("Switching to First Person camera via enhanced input");
    }
}

// ============================================================================
// Default Keymap Specification
// ============================================================================

fn default_keymap_spec() -> KeymapSpec {
    let actions = vec![
        ActionDescriptor {
            id: ActionId::from("ui::toggle_menu"),
            output: Some("bool".into()),
            modifiers: Vec::new(),
            conditions: Vec::new(),
            settings: None,
        },
        ActionDescriptor {
            id: ActionId::from("camera::switch_to_pan_orbit"),
            output: Some("bool".into()),
            modifiers: Vec::new(),
            conditions: Vec::new(),
            settings: None,
        },
        ActionDescriptor {
            id: ActionId::from("camera::switch_to_first_person"),
            output: Some("bool".into()),
            modifiers: Vec::new(),
            conditions: Vec::new(),
            settings: None,
        },
    ];

    let contexts = vec![ContextDescriptor {
        id: ContextId::from("global"),
        predicate: None,
        priority: Some(0.0),
        schedule: None,
        settings: None,
    }];

    let bindings = vec![
        BindingDescriptor {
            action_id: Some(ActionId::from("ui::toggle_menu")),
            meta: Some(KeyBindingMetaIndex::DEFAULT),
            modifiers: Vec::new(),
            conditions: Vec::new(),
            settings: None,
            input: Some(BindingInputDescriptor::keyboard(
                parse_keystroke_sequence("escape").expect("static key sequence"),
            )),
        },
        BindingDescriptor {
            action_id: Some(ActionId::from("ui::toggle_menu")),
            meta: Some(KeyBindingMetaIndex::DEFAULT),
            modifiers: Vec::new(),
            conditions: Vec::new(),
            settings: None,
            input: Some(BindingInputDescriptor::GamepadButton {
                button: "start".into(),
                threshold: None,
            }),
        },
        // Camera switching
        BindingDescriptor {
            action_id: Some(ActionId::from("camera::switch_to_pan_orbit")),
            meta: Some(KeyBindingMetaIndex::DEFAULT),
            modifiers: Vec::new(),
            conditions: Vec::new(),
            settings: None,
            input: Some(BindingInputDescriptor::keyboard(
                parse_keystroke_sequence("c").expect("static key sequence"),
            )),
        },
        BindingDescriptor {
            action_id: Some(ActionId::from("camera::switch_to_first_person")),
            meta: Some(KeyBindingMetaIndex::DEFAULT),
            modifiers: Vec::new(),
            conditions: Vec::new(),
            settings: None,
            input: Some(BindingInputDescriptor::keyboard(
                parse_keystroke_sequence("c").expect("static key sequence"),
            )),
        },
    ];

    KeymapSpec { actions, bindings }
}

pub use KeymapInputPlugin as InputPlugin;
pub use KeymapStoreResource as StoreResource;
