// scenes/in_game/cameras/mod.rs

pub(super) mod cursor;
mod first_person;
mod pan_orbit;

use crate::GameState;
use crate::ui::components::InGameMenuState;
use crate::utils::cleanup;
use bevy::prelude::*;
use bevy::transform::TransformSystems;
use cursor::apply_cursor_state;

// Re-export CursorState for use in input module
pub use cursor::CursorState;

pub struct InGameCamerasPlugin;

/// Camera mode enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    FirstPerson,
    PanOrbit,
}

/// Component that tracks the active camera mode on the camera entity
#[derive(Component, Debug, Clone, Copy)]
pub struct ActiveCameraMode {
    pub mode: CameraMode,
}

impl Default for ActiveCameraMode {
    fn default() -> Self {
        Self {
            mode: CameraMode::FirstPerson,
        }
    }
}

impl Plugin for InGameCamerasPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register PanOrbit resources
            .init_resource::<pan_orbit::ActiveCameraData>()
            .init_resource::<pan_orbit::MouseKeyTracker>()
            .init_resource::<pan_orbit::TouchTracker>()
            .init_resource::<CursorState>()
            // Setup systems
            .add_systems(OnEnter(GameState::InGame), spawn_camera)
            .add_systems(
                Update,
                (
                    apply_cursor_state,
                    // FirstPerson systems
                    (
                        first_person::handle_mouse_look,
                        first_person::follow_player,
                        first_person::apply_orientation,
                    )
                        .chain()
                        .run_if(in_state(GameState::InGame))
                        .run_if(is_first_person_active)
                        .run_if(is_menu_inactive),
                    // Toggle system
                    toggle_camera_mode
                        .run_if(in_state(GameState::InGame))
                        .run_if(is_menu_inactive),
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    // PanOrbit systems
                    (
                        pan_orbit::active_viewport_data.run_if(
                            |active_cam: Res<pan_orbit::ActiveCameraData>| !active_cam.manual,
                        ),
                        (pan_orbit::mouse_key_tracker, pan_orbit::touch_tracker),
                        pan_orbit::pan_orbit_camera,
                    )
                        .chain()
                        .run_if(in_state(GameState::InGame))
                        .run_if(is_pan_orbit_active)
                        .before(TransformSystems::Propagate),
                    // Follow player focus for PanOrbit
                    pan_orbit::follow_local_player_focus
                        .run_if(in_state(GameState::InGame))
                        .run_if(is_pan_orbit_active),
                ),
            )
            .add_systems(
                OnExit(GameState::InGame),
                (cleanup::<InGameCamera>, release_cursor),
            );
    }
}

#[derive(Component)]
pub struct InGameCamera;

/// Run condition: Check if FirstPerson mode is active
fn is_first_person_active(camera_query: Query<&ActiveCameraMode, With<InGameCamera>>) -> bool {
    camera_query
        .iter()
        .next()
        .map(|mode| mode.mode == CameraMode::FirstPerson)
        .unwrap_or(false)
}
fn is_menu_inactive(menu: Res<InGameMenuState>) -> bool {
    menu.is_closed()
}

/// Run condition: Check if PanOrbit mode is active
fn is_pan_orbit_active(camera_query: Query<&ActiveCameraMode, With<InGameCamera>>) -> bool {
    camera_query
        .iter()
        .next()
        .map(|mode| mode.mode == CameraMode::PanOrbit)
        .unwrap_or(false)
}

fn spawn_camera(mut commands: Commands, mut cursor: ResMut<CursorState>) {
    // Spawn a single camera with both FirstPerson and PanOrbit components
    // Start in FirstPerson mode
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.7, 0.0).looking_at(Vec3::new(0.0, 1.7, -1.0), Vec3::Y),
        InGameCamera,
        ActiveCameraMode::default(), // Starts in FirstPerson
        first_person::FirstPersonView::default(),
        pan_orbit::PanOrbitCamera::default(),
        Name::new("InGame Camera"),
    ));

    // Start with locked cursor for FirstPerson
    *cursor = CursorState::LOCKED;
}

fn toggle_camera_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<
        (
            &mut ActiveCameraMode,
            &mut Transform,
            &mut first_person::FirstPersonView,
            &mut pan_orbit::PanOrbitCamera,
        ),
        With<InGameCamera>,
    >,
    mut cursor: ResMut<CursorState>,
) {
    if keyboard.just_pressed(KeyCode::KeyC) {
        let Some((mut active_mode, mut transform, mut fp_view, mut pan_orbit)) =
            camera_query.iter_mut().next()
        else {
            return;
        };

        // Toggle mode
        match active_mode.mode {
            CameraMode::FirstPerson => {
                // Switch to PanOrbit
                active_mode.mode = CameraMode::PanOrbit;

                // Reset PanOrbit to default values
                *pan_orbit = pan_orbit::PanOrbitCamera::default();
                pan_orbit.focus = Vec3::new(0.0, 1.0, 0.0);
                pan_orbit.target_focus = Vec3::new(0.0, 1.0, 0.0);
                pan_orbit.target_radius = 5.0;
                pan_orbit.radius = Some(5.0);

                // Set initial PanOrbit transform
                *transform = Transform::from_xyz(0.0, 3.0, 5.0)
                    .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y);

                // Free cursor for PanOrbit
                *cursor = CursorState::FREE;
            }
            CameraMode::PanOrbit => {
                // Switch to FirstPerson
                active_mode.mode = CameraMode::FirstPerson;

                // Reset FirstPerson to default values
                *fp_view = first_person::FirstPersonView::default();

                // Set initial FirstPerson transform
                *transform = Transform::from_xyz(0.0, 1.7, 0.0)
                    .looking_at(Vec3::new(0.0, 1.7, -1.0), Vec3::Y);

                // Lock cursor for FirstPerson
                *cursor = CursorState::LOCKED;
            }
        }
    }
}

fn release_cursor(mut cursor: ResMut<CursorState>) {
    *cursor = CursorState::FREE;
}
