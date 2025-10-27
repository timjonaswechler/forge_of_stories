use super::{CameraMode, CameraTransitionState, InGameCameraMode, SceneCamera};
use crate::client::LocalPlayer;
use bevy::{
    input::mouse::MouseMotion,
    math::EulerRot,
    prelude::{MessageReader, *},
};
use game_server::components::Position;
use std::f32::consts::{FRAC_PI_2, TAU};

/// Maximum pitch angle (in radians) to avoid flipping the camera upside down.
const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;

/// Component describing how the first-person camera should follow the local player.
#[derive(Component)]
pub struct FollowLocalPlayer {
    pub offset: Vec3,
}

/// Stores current yaw/pitch for the first-person camera.
#[derive(Component)]
pub struct FirstPersonView {
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: f32,
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

pub(super) fn follow_local_player(
    local_player: Query<
        (Option<&Transform>, Option<&Position>),
        (With<LocalPlayer>, Without<SceneCamera>),
    >,
    mut camera_query: Query<(&FollowLocalPlayer, &mut Transform), With<SceneCamera>>,
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

pub(super) fn update_first_person_view_from_input(
    mut mouse_motion: MessageReader<MouseMotion>,
    camera_mode: Res<CameraMode>,
    mut view_query: Query<&mut FirstPersonView, With<SceneCamera>>,
    transition_state: Res<CameraTransitionState>,
) {
    // Nur aktiv im FirstPerson-Mode
    let is_active = matches!(
        *camera_mode,
        CameraMode::InGame(InGameCameraMode::FirstPerson)
    );

    if transition_state.active || !is_active {
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

pub(super) fn apply_first_person_orientation(
    mut query: Query<(&FirstPersonView, &mut Transform), With<SceneCamera>>,
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
