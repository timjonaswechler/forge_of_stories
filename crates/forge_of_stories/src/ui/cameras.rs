mod first_person;
mod main_menu;
mod pan_orbit;
mod splashscreen;
mod transition;

use crate::GameState;
use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

pub use first_person::FirstPersonCameraPlugin;
pub use main_menu::MainMenuCameraPlugin;
pub use pan_orbit::PanOrbitCameraPlugin;
pub use splashscreen::SplashscreenCameraPlugin;
pub use transition::CameraTransitionPlugin;

/// Marker component for splashscreen camera
#[derive(Component)]
pub struct SplashscreenCamera;

/// Marker component for main menu camera
#[derive(Component)]
pub struct MainMenuCamera;

/// Marker component for first person camera
#[derive(Component)]
pub struct FirstPersonCamera;

/// Tracks whether a camera transition animation is currently running.
#[derive(Resource, Default)]
pub struct CameraTransitionState {
    pub active: bool,
}

/// Event emitted when the active in-game camera should toggle.
#[derive(Message, Default)]
pub struct ToggleCameraEvent;

/// Resource tracking which in-game camera is currently active
#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum ActiveInGameCamera {
    #[default]
    FirstPerson,
    PanOrbit,
}

impl ActiveInGameCamera {
    pub fn toggle(&mut self) {
        *self = match self {
            Self::FirstPerson => Self::PanOrbit,
            Self::PanOrbit => Self::FirstPerson,
        };
    }
}

/// Main camera plugin that coordinates all camera sub-plugins
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            SplashscreenCameraPlugin,
            MainMenuCameraPlugin,
            FirstPersonCameraPlugin,
            PanOrbitCameraPlugin,
            CameraTransitionPlugin,
        ))
        .add_message::<ToggleCameraEvent>()
        .init_resource::<CameraTransitionState>()
        .init_resource::<ActiveInGameCamera>()
        .add_systems(
            Update,
            toggle_active_camera.run_if(in_state(GameState::InGame)),
        )
        .add_systems(OnEnter(GameState::InGame), apply_cursor_for_active_camera)
        .add_systems(OnExit(GameState::InGame), release_cursor);
    }
}

fn toggle_active_camera(
    keys: Res<ButtonInput<KeyCode>>,
    transition_state: Res<CameraTransitionState>,
    mut toggle_events: MessageWriter<ToggleCameraEvent>,
) {
    if transition_state.active {
        return;
    }

    if keys.just_pressed(KeyCode::KeyC) {
        toggle_events.write(ToggleCameraEvent);
    }
}

fn apply_cursor_for_active_camera(
    active_camera: Res<ActiveInGameCamera>,
    mut window_query: Query<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    let grab = *active_camera == ActiveInGameCamera::FirstPerson;
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
