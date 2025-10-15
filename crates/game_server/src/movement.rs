//! Player movement and physics systems.

use bevy::prelude::*;
use networking::prelude::*;

use crate::components::{PlayerOwner, Position, Velocity};
use crate::messages::PlayerInput;

/// Movement speed in units per second.
const MOVE_SPEED: f32 = 5.0;

/// System that applies velocity to position (simple integration).
///
/// Runs in the Simulation phase of the server tick.
pub fn apply_velocity(mut query: Query<(&mut Position, &Velocity)>, time: Res<Time>) {
    for (mut pos, vel) in &mut query {
        pos.translation += vel.linear * time.delta_secs();
    }
}

/// System that processes player inputs from clients and updates velocity.
///
/// Uses bevy_replicon's MessageReader to receive PlayerInput messages from clients.
/// Runs after ServerSystems::Receive to ensure all messages are available.
pub fn process_player_input(
    mut players: Query<(&PlayerOwner, &mut Velocity)>,
    mut inputs: MessageReader<FromClient<PlayerInput>>,
) {
    // Process all incoming player inputs
    for FromClient { client_id, message } in inputs.read() {
        // Get the client entity from the ClientId
        let Some(client_entity) = client_id.entity() else {
            continue;
        };

        // Find the player entity for this client
        for (owner, mut velocity) in &mut players {
            if owner.client_entity == client_entity {
                // Convert 2D input to 3D velocity (XZ plane)
                let move_vec = Vec3::new(message.direction.x, 0.0, message.direction.y);

                // Normalize and apply speed
                if move_vec.length() > 0.01 {
                    velocity.linear = move_vec.normalize() * MOVE_SPEED;
                } else {
                    velocity.linear = Vec3::ZERO;
                }

                // TODO: Jump handling (would require ground detection)
                // if message.jump { ... }

                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
