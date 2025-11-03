//! Physics simulation systems.

use bevy::prelude::*;

use crate::shared::{Position, Velocity};

/// System that applies velocity to position (simple integration).
///
/// Runs in FixedUpdate to ensure consistent physics simulation.
pub fn apply_velocity(mut query: Query<(&mut Position, &Velocity)>, time: Res<Time>) {
    for (mut pos, vel) in &mut query {
        pos.translation += vel.linear * time.delta_secs();
    }
}

/// Placeholder for future physics simulation (collisions, gravity, etc.).
pub fn simulate_physics() {
    // TODO: Implement collision detection, gravity, etc.
}
