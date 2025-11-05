//! Client-side interpolation for smooth replicated entity movement.
//!
//! The server runs at 20Hz (50ms updates), but the client renders at 60+ FPS.
//! Without interpolation, movement appears jittery. This module uses velocity-based
//! extrapolation to smoothly predict entity positions between server updates.

use bevy::prelude::*;
use bevy_replicon::prelude::ClientSet;
use game_server::{Player, Velocity};

/// Plugin that adds client-side interpolation for replicated entities.
pub struct InterpolationPlugin;

impl Plugin for InterpolationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            interpolate_player_movement
                // Run after receiving server updates
                .after(ClientSet::Receive),
        );
    }
}

/// Smoothly interpolates player positions using their replicated velocity.
///
/// This system runs every frame on the client to extrapolate movement between
/// server updates (20Hz → 60+ FPS), eliminating jitter.
///
/// How it works:
/// 1. Server sends Transform + Velocity updates at 20Hz
/// 2. Client applies velocity * delta_time every frame
/// 3. When next server update arrives, position is corrected
///
/// This is a simple form of client-side prediction that works well for
/// continuous movement but may overshoot on sudden direction changes.
fn interpolate_player_movement(
    mut players: Query<(&mut Transform, &Velocity), With<Player>>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();

    for (mut transform, velocity) in &mut players {
        // Only interpolate if there's meaningful velocity
        if velocity.linear.length_squared() > 0.001 {
            // Extrapolate position based on velocity
            let predicted_movement = velocity.linear * delta;
            transform.translation += predicted_movement;
        }
    }
}
