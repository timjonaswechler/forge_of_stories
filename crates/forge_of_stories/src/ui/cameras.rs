mod defaults;
mod first_person;
mod main_menu;
mod mode_handler;
mod pan_orbit;
mod transition;
mod ui_camera;

use crate::GameState;
use bevy::{
    camera::CameraUpdateSystems,
    prelude::*,
    transform::TransformSystems,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
pub use defaults::CameraDefaults;
pub use mode_handler::{handle_camera_mode_changes, handle_transition_completion};
pub use pan_orbit::ActiveCameraData;
pub use ui_camera::UiCameraPlugin;

// Re-export for use in other modules
use pan_orbit::PanOrbitCamera;

/// Marker component for splashscreen camera
#[derive(Resource, Clone, Debug, PartialEq)]
pub enum CameraMode {
    Splashscreen,
    MainMenu,
    InGame(InGameCameraMode),
}

impl Default for CameraMode {
    fn default() -> Self {
        Self::Splashscreen
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum InGameCameraMode {
    FirstPerson,
    PanOrbit,
    // ThirdPerson,  // später
}

#[derive(Component)]
pub struct SceneCamera;

/// Tracks whether a camera transition animation is currently running.
#[derive(Resource, Default)]
pub struct CameraTransitionState {
    pub active: bool,
}

/// Main camera plugin that coordinates all camera sub-plugins
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraDefaults>()
            .init_resource::<CameraMode>()
            .init_resource::<CameraTransitionState>()
            // PanOrbit Resources
            .init_resource::<ActiveCameraData>()
            .init_resource::<pan_orbit::MouseKeyTracker>()
            .init_resource::<pan_orbit::TouchTracker>()
            .add_message::<CameraModeChangeEvent>()
            // SceneCamera beim Start spawnen
            .add_systems(Startup, spawn_scene_camera)
            // State-Transitions → Mode-Changes
            .add_systems(
                OnEnter(GameState::Splashscreen),
                switch_to_splashscreen_mode,
            )
            .add_systems(OnEnter(GameState::MainMenu), switch_to_main_menu_mode)
            .add_systems(OnEnter(GameState::InGame), switch_to_ingame_mode)
            // Mode-Switching-Logik
            .add_systems(
                Update,
                (
                    handle_camera_mode_changes,
                    // Läuft während Transition, updated Translation
                    handle_transition_completion,
                ),
            )
            // InGame Toggle
            .add_systems(
                Update,
                toggle_ingame_camera_mode.run_if(in_state(GameState::InGame)),
            )
            // FirstPerson Mode Update-Systems
            .add_systems(
                Update,
                (
                    first_person::update_first_person_view_from_input,
                    first_person::apply_first_person_orientation,
                    first_person::follow_local_player,
                )
                    .run_if(in_state(GameState::InGame))
                    .run_if(resource_exists::<CameraMode>)
                    .run_if(|mode: Res<CameraMode>| {
                        matches!(*mode, CameraMode::InGame(InGameCameraMode::FirstPerson))
                    }),
            )
            // PanOrbit Mode Update-Systems
            .add_systems(
                PostUpdate,
                (
                    (
                        pan_orbit::active_viewport_data,
                        pan_orbit::mouse_key_tracker,
                        pan_orbit::touch_tracker,
                    ),
                    pan_orbit::pan_orbit_camera,
                )
                    .chain()
                    .before(TransformSystems::Propagate)
                    .before(CameraUpdateSystems)
                    .run_if(in_state(GameState::InGame))
                    .run_if(resource_exists::<CameraMode>)
                    .run_if(|mode: Res<CameraMode>| {
                        matches!(*mode, CameraMode::InGame(InGameCameraMode::PanOrbit))
                    })
                    // WICHTIG: Während Transition nicht laufen lassen!
                    .run_if(|transition: Res<CameraTransitionState>| !transition.active),
            )
            .add_systems(
                Update,
                pan_orbit::follow_local_player_focus
                    .run_if(in_state(GameState::InGame))
                    .run_if(resource_exists::<CameraMode>)
                    .run_if(|mode: Res<CameraMode>| {
                        matches!(*mode, CameraMode::InGame(InGameCameraMode::PanOrbit))
                    }),
            )
            // Plugins
            .add_plugins((
                UiCameraPlugin,
                bevy_tweening::TweeningPlugin, // Für Transitions
            ))
            // Cursor-Management
            .add_systems(OnEnter(GameState::InGame), apply_cursor_for_active_camera)
            .add_systems(OnExit(GameState::InGame), release_cursor);
    }
}

// Event für Mode-Wechsel
#[derive(Message)]
pub struct CameraModeChangeEvent {
    pub new_mode: CameraMode,
    pub animate: bool, // true = mit Transition, false = instant
}

// State-Transitions triggern Mode-Changes
fn switch_to_splashscreen_mode(mut events: MessageWriter<CameraModeChangeEvent>) {
    events.write(CameraModeChangeEvent {
        new_mode: CameraMode::Splashscreen,
        animate: false, // kein Transition zwischen States
    });
}

fn switch_to_main_menu_mode(mut events: MessageWriter<CameraModeChangeEvent>) {
    events.write(CameraModeChangeEvent {
        new_mode: CameraMode::MainMenu,
        animate: false,
    });
}

fn switch_to_ingame_mode(mut events: MessageWriter<CameraModeChangeEvent>) {
    events.write(CameraModeChangeEvent {
        new_mode: CameraMode::InGame(InGameCameraMode::FirstPerson),
        animate: false,
    });
}

// Toggle zwischen FirstPerson/PanOrbit
fn toggle_ingame_camera_mode(
    keys: Res<ButtonInput<KeyCode>>,
    transition_state: Res<CameraTransitionState>,
    current_mode: Res<CameraMode>,
    mut events: MessageWriter<CameraModeChangeEvent>,
) {
    if transition_state.active {
        return;
    }

    if keys.just_pressed(KeyCode::KeyC) {
        if let CameraMode::InGame(ingame_mode) = current_mode.as_ref() {
            let new_ingame_mode = match ingame_mode {
                InGameCameraMode::FirstPerson => InGameCameraMode::PanOrbit,
                InGameCameraMode::PanOrbit => InGameCameraMode::FirstPerson,
            };

            events.write(CameraModeChangeEvent {
                new_mode: CameraMode::InGame(new_ingame_mode),
                animate: true, // MIT Transition innerhalb InGame
            });
        }
    }
}

fn apply_cursor_for_active_camera(
    camera_mode: Res<CameraMode>,
    mut window_query: Query<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    let grab = matches!(
        *camera_mode,
        CameraMode::InGame(InGameCameraMode::FirstPerson)
    );
    set_cursor_state(grab, &mut window_query);
}

fn release_cursor(mut window_query: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    if let Ok(mut cursor) = window_query.single_mut() {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }
}

pub(super) fn set_cursor_state(
    grab: bool,
    window_query: &mut Query<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    if let Ok((mut window, mut cursor)) = window_query.single_mut() {
        if grab {
            cursor.grab_mode = CursorGrabMode::Locked;
            cursor.visible = false;
            window.focused = true;
        } else {
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
        }
    }
}

fn spawn_scene_camera(mut commands: Commands, defaults: Res<CameraDefaults>) {
    // Initial im Splashscreen-Mode starten
    let transform = Transform::from_translation(defaults.splashscreen.position)
        .looking_at(defaults.splashscreen.look_at, Vec3::Y);

    commands.spawn((
        Camera3d::default(),
        transform,
        SceneCamera,
        Name::new("Scene Camera"),
    ));
}
