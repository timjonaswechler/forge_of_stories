use super::{ActiveInGameCamera, CameraDefaults, CameraTransitionState, FirstPersonCamera};
use crate::{GameState, client::LocalPlayer, utils::cleanup};
use bevy::{
    input::mouse::MouseMotion,
    math::EulerRot,
    prelude::{MessageReader, *},
};
use bevy_panorbit_camera::PanOrbitCamera;
use game_server::components::Position;
use std::f32::consts::{FRAC_PI_2, TAU};

/// Maximum pitch angle (in radians) to avoid flipping the camera upside down.
const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;

/// Component describing how the first-person camera should follow the local player.
#[derive(Component)]
pub(super) struct FollowLocalPlayer {
    offset: Vec3,
}

/// Stores current yaw/pitch for the first-person camera.
#[derive(Component)]
pub(super) struct FirstPersonView {
    yaw: f32,
    pitch: f32,
    sensitivity: f32,
}

impl Default for FirstPersonView {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            sensitivity: 0.002,
        }
    }
}

/// Plugin that spawns and updates the first-person camera.
pub struct FirstPersonCameraPlugin;

impl Plugin for FirstPersonCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_first_person_camera)
            .add_systems(
                Update,
                (
                    update_first_person_view_from_input,
                    apply_first_person_orientation,
                    follow_local_player,
                    sync_camera_activation,
                )
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnExit(GameState::InGame), cleanup::<FirstPersonCamera>);
    }
}

fn spawn_first_person_camera(mut commands: Commands, defaults: Res<CameraDefaults>) {
    commands.spawn((
        Camera3d::default(),
        Transform::default(),
        FirstPersonCamera,
        FollowLocalPlayer {
            offset: Vec3::new(0.0, defaults.first_person.height_offset, 0.0),
        },
        FirstPersonView {
            sensitivity: defaults.first_person.mouse_sensitivity,
            ..Default::default()
        },
        Name::new("First-Person Camera"),
    ));
}

fn follow_local_player(
    local_player: Query<
        (Option<&Transform>, Option<&Position>),
        (With<LocalPlayer>, Without<FirstPersonCamera>),
    >,
    mut camera_query: Query<(&FollowLocalPlayer, &mut Transform), With<FirstPersonCamera>>,
    transition_state: Res<CameraTransitionState>,
) {
    if transition_state.active {
        return;
    }

    let Ok((transform, position)) = local_player.single() else {
        return;
    };
    let Ok((follow, mut camera_transform)) = camera_query.single_mut() else {
        return;
    };

    let base_translation = transform
        .map(|t| t.translation)
        .or_else(|| position.map(|p| p.translation))
        .unwrap_or(Vec3::ZERO);

    camera_transform.translation = base_translation + follow.offset;
}

fn update_first_person_view_from_input(
    mut mouse_motion: MessageReader<MouseMotion>,
    active_camera: Res<ActiveInGameCamera>,
    mut view_query: Query<&mut FirstPersonView, With<FirstPersonCamera>>,
    transition_state: Res<CameraTransitionState>,
) {
    if transition_state.active || *active_camera != ActiveInGameCamera::FirstPerson {
        // Drain events so they don't accumulate when camera is inactive.
        for _ in mouse_motion.read() {}
        return;
    }

    let Ok(mut view) = view_query.single_mut() else {
        // Consume events even if the camera entity is missing.
        for _ in mouse_motion.read() {}
        return;
    };

    for motion in mouse_motion.read() {
        let delta = motion.delta * view.sensitivity;
        view.yaw = (view.yaw - delta.x).rem_euclid(TAU);
        view.pitch = (view.pitch - delta.y).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }
}

fn apply_first_person_orientation(
    mut query: Query<(&FirstPersonView, &mut Transform), With<FirstPersonCamera>>,
    transition_state: Res<CameraTransitionState>,
) {
    if transition_state.active {
        return;
    }

    let Ok((view, mut transform)) = query.single_mut() else {
        return;
    };

    let yaw = Quat::from_rotation_y(view.yaw);
    let pitch = Quat::from_rotation_x(view.pitch);
    transform.rotation = yaw * pitch;
}

fn sync_camera_activation(
    active_camera: Res<ActiveInGameCamera>,
    mut first_person: Query<&mut Camera, With<FirstPersonCamera>>,
    mut pan_orbit: Query<&mut Camera, (With<PanOrbitCamera>, Without<FirstPersonCamera>)>,
    transition_state: Res<CameraTransitionState>,
) {
    if transition_state.active {
        return;
    }

    let activate_first_person = *active_camera == ActiveInGameCamera::FirstPerson;
    if let Ok(mut camera) = first_person.single_mut() {
        camera.is_active = activate_first_person;
    }

    let activate_pan_orbit = *active_camera == ActiveInGameCamera::PanOrbit;
    for mut camera in &mut pan_orbit {
        camera.is_active = activate_pan_orbit;
    }
}

pub(super) fn first_person_transform_from_view(
    player_translation: Vec3,
    follow: &FollowLocalPlayer,
    view: &FirstPersonView,
) -> Transform {
    let translation = player_translation + follow.offset;
    let rotation = Quat::from_rotation_y(view.yaw) * Quat::from_rotation_x(view.pitch);
    let mut transform = Transform::from_translation(translation);
    transform.rotation = rotation;
    transform
}

pub(super) fn set_view_from_rotation(view: &mut FirstPersonView, rotation: Quat) {
    let (yaw, pitch, _) = rotation.to_euler(EulerRot::YXZ);
    view.yaw = yaw;
    view.pitch = pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
}
