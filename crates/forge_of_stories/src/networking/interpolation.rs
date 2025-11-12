//! Client-side interpolation for smooth replicated entity movement.
//!
//! The server runs at 20Hz (50ms updates), but the client renders at 60+ FPS.
//! This module implements smooth interpolation with position correction to eliminate
//! jittering and snapping when server updates arrive.
//!
//! Technique: Exponential smoothing with velocity-based prediction
//! - Predict movement using velocity between server updates
//! - When server correction arrives, smoothly lerp to correct position
//! - Avoids hard "snaps" on direction changes or stop/start

use bevy::prelude::*;
use bevy_replicon::prelude::ClientSystems;
use game_server::{Player, Velocity};

/// Plugin that adds client-side interpolation for replicated entities.
pub struct InterpolationPlugin;

impl Plugin for InterpolationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (capture_server_updates, interpolate_to_server_position)
                .chain()
                .after(ClientSystems::Receive),
        );
    }
}

/// Component that tracks interpolation state for smooth corrections.
#[derive(Component)]
pub struct InterpolationState {
    /// The authoritative server position we're interpolating towards
    pub server_position: Vec3,
    /// Previous frame's position to detect server updates
    pub previous_position: Vec3,
    /// How fast to correct position errors (0.0 = no correction, 1.0 = instant snap)
    /// Typical values: 0.1-0.3 for smooth correction
    pub correction_speed: f32,
}

impl Default for InterpolationState {
    fn default() -> Self {
        Self {
            server_position: Vec3::ZERO,
            previous_position: Vec3::ZERO,
            correction_speed: 0.15, // 15% per frame = smooth correction over ~10-15 frames
        }
    }
}

/// Captures server Transform updates and stores them as interpolation targets.
///
/// This runs immediately after ClientSet::Receive, so Changed<Transform> will only
/// detect server updates, not our own interpolation changes (which happen later).
fn capture_server_updates(
    mut commands: Commands,
    mut updated_players: Query<
        (Entity, &Transform, Option<&mut InterpolationState>),
        (With<Player>, Changed<Transform>),
    >,
) {
    for (entity, transform, maybe_interp) in &mut updated_players {
        if let Some(mut interp) = maybe_interp {
            // Server sent an update - store it as our new target
            interp.server_position = transform.translation;
            info!(
                "Server update received - new target: {:?}, current: {:?}, error: {:.3}",
                interp.server_position,
                interp.previous_position,
                (interp.server_position - interp.previous_position).length()
            );
        } else {
            // First time seeing this entity, initialize interpolation state
            commands.entity(entity).insert(InterpolationState {
                server_position: transform.translation,
                previous_position: transform.translation,
                ..default()
            });
            info!(
                "Initialized interpolation for new player at {:?}",
                transform.translation
            );
        }
    }
}

/// Smoothly interpolates player positions towards server-authoritative position.
///
/// This system runs every frame to:
/// 1. Apply velocity-based prediction for smooth continuous movement
/// 2. Smoothly correct towards server position to fix prediction errors
///
/// This eliminates both:
/// - Jittering from low server tickrate (via velocity prediction)
/// - Snapping from server corrections (via exponential smoothing)
fn interpolate_to_server_position(
    mut players: Query<(&mut Transform, &Velocity, &mut InterpolationState), With<Player>>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();

    for (mut transform, velocity, mut interp_state) in &mut players {
        // Store current position before we modify it
        let position_before = transform.translation;

        // Calculate new position
        let mut new_position = transform.translation;

        // 1. Apply velocity-based prediction for smooth continuous movement
        if velocity.linear.length_squared() > 0.001 {
            let predicted_movement = velocity.linear * delta;
            new_position += predicted_movement;
        }

        // 2. Smoothly correct towards server position (exponential smoothing)
        // Calculate error between where we are and where server says we should be
        let error = interp_state.server_position - new_position;
        let error_magnitude = error.length();

        // Only apply correction if error is significant
        if error_magnitude > 0.001 {
            // Exponential smoothing: move X% of the way to target each frame
            // This creates a smooth "rubber band" effect that gradually pulls us back
            let correction = error * interp_state.correction_speed;
            new_position += correction;

            // Log significant corrections (useful for debugging)
            if error_magnitude > 0.5 {
                info!(
                    "Large position error: {:.3} units, applying correction: {:?}",
                    error_magnitude, correction
                );
            }
        }

        // Apply the new position WITHOUT triggering change detection
        // This prevents our interpolation from being detected as a "server update"
        // in the next frame's capture_server_updates system
        transform.bypass_change_detection().translation = new_position;

        // Store position for next frame's server update detection
        interp_state.previous_position = new_position;
    }
}
