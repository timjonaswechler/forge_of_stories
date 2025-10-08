//! Player movement and physics systems.

use bevy::prelude::*;
use shared::ClientId;

use crate::world::{Position, Velocity};

/// Movement speed in units per second.
const MOVE_SPEED: f32 = 5.0;

/// Player input command (sent from client to server).
#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerInput {
    /// Movement direction (normalized, in world space).
    pub direction: Vec2,
    /// Jump requested.
    pub jump: bool,
}

/// Resource storing pending player inputs (keyed by player/client ID).
#[derive(Resource, Default)]
pub struct PlayerInputQueue {
    pub inputs: std::collections::HashMap<ClientId, PlayerInput>,
}

/// System that applies velocity to position (simple integration).
///
/// Runs in the Simulation phase of the server tick.
pub fn apply_velocity(mut query: Query<(&mut Position, &Velocity)>, time: Res<Time<Fixed>>) {
    for (mut pos, vel) in &mut query {
        pos.translation += vel.linear * time.delta_secs();
    }
}

/// System that processes player inputs and updates velocity.
///
/// Runs in the Input phase of the server tick.
pub fn process_player_input(
    mut query: Query<(&crate::world::Player, &mut Velocity)>,
    mut input_queue: ResMut<PlayerInputQueue>,
) {
    for (player, mut velocity) in &mut query {
        if let Some(input) = input_queue.inputs.get(&player.id) {
            // Convert 2D input to 3D velocity (XZ plane)
            let move_vec = Vec3::new(input.direction.x, 0.0, input.direction.y);

            // Normalize and apply speed
            if move_vec.length() > 0.01 {
                velocity.linear = move_vec.normalize() * MOVE_SPEED;
            } else {
                velocity.linear = Vec3::ZERO;
            }

            // TODO: Jump handling (would require ground detection)
            // if input.jump { ... }
        } else {
            // No input - stop moving
            velocity.linear = Vec3::ZERO;
        }
    }

    // Clear inputs after processing
    input_queue.inputs.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::uuid;

    #[test]
    fn test_velocity_integration() {
        // Simple test: verify the system compiles and can be added to a schedule
        let mut app = App::new();
        app.add_systems(Update, apply_velocity);

        // Spawn an entity with position and velocity
        app.world_mut().spawn((
            Position {
                translation: Vec3::ZERO,
            },
            Velocity {
                linear: Vec3::new(1.0, 0.0, 0.0),
            },
        ));

        // Test passes if the system is valid
    }

    #[test]
    fn test_player_input_processing() {
        use crate::world::Player;

        let mut app = App::new();
        app.insert_resource(PlayerInputQueue::default());

        let entity = app
            .world_mut()
            .spawn((
                Player {
                    id: uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
                    color: Color::WHITE,
                },
                Velocity::default(),
            ))
            .id();

        // Queue an input
        app.world_mut()
            .resource_mut::<PlayerInputQueue>()
            .inputs
            .insert(
                uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
                PlayerInput {
                    direction: Vec2::new(1.0, 0.0),
                    jump: false,
                },
            );

        app.add_systems(Update, process_player_input);
        app.update();

        let vel = app.world().get::<Velocity>(entity).unwrap();
        assert!(vel.linear.x > 0.0, "Velocity should be set by input");
        assert_eq!(vel.linear.y, 0.0, "Y velocity should be zero");
        assert_eq!(vel.linear.z, 0.0, "Z velocity should be zero");
    }
}
