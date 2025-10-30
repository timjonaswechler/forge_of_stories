// scenes/in_game/cameras/first_person.rs

use super::InGameCamera;
use crate::client::LocalPlayer;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use game_server::components::Position;
use std::f32::consts::{FRAC_PI_2, TAU};

const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;

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

pub fn handle_mouse_look(
    mut mouse_motion: EventReader<MouseMotion>,
    mut camera: Query<&mut FirstPersonView, With<InGameCamera>>,
) {
    let Ok(mut view) = camera.single_mut() else {
        for _ in mouse_motion.read() {}
        return;
    };

    for motion in mouse_motion.read() {
        let delta = motion.delta * view.sensitivity;
        view.yaw = (view.yaw - delta.x).rem_euclid(TAU);
        view.pitch = (view.pitch - delta.y).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }
}

pub fn follow_player(
    local_player: Query<
        (Option<&Transform>, Option<&Position>),
        (With<LocalPlayer>, Without<InGameCamera>),
    >,
    mut camera: Query<&mut Transform, With<InGameCamera>>,
) {
    let Ok((player_transform, player_position)) = local_player.single() else {
        return;
    };

    let Ok(mut camera_transform) = camera.single_mut() else {
        return;
    };

    let player_pos = player_transform
        .map(|t| t.translation)
        .or_else(|| player_position.map(|p| p.translation))
        .unwrap_or(Vec3::ZERO);

    camera_transform.translation = player_pos + Vec3::new(0.0, 1.7, 0.0);
}

pub fn apply_orientation(
    mut camera: Query<(&FirstPersonView, &mut Transform), With<InGameCamera>>,
) {
    let Ok((view, mut transform)) = camera.single_mut() else {
        return;
    };

    let yaw = Quat::from_rotation_y(view.yaw);
    let pitch = Quat::from_rotation_x(view.pitch);
    transform.rotation = yaw * pitch;
}
