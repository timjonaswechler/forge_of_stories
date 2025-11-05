//! Physics simulation systems.

use bevy::prelude::*;

use crate::shared::Velocity;

/// System that applies velocity to position (simple integration).
///
/// Runs in FixedUpdate to ensure consistent physics simulation at 20Hz.
pub fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, vel) in &mut query {
        if vel.linear.length() > 0.01 {
            let movement = vel.linear * time.delta_secs();
            transform.translation += movement;
            info!("Applying velocity: vel={:?}, delta={:.4}, movement={:?}", vel.linear, time.delta_secs(), movement);
        }
    }
}

/// Placeholder for future physics simulation (collisions, gravity, etc.).
pub fn simulate_physics() {
    // TODO: Implement collision detection, gravity, etc.
}
