//! Player movement and physics systems.

use bevy::prelude::*;
use bevy_replicon::prelude::*;

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
/// Runs in PreUpdate after ServerSystems::Receive to ensure all messages are available.
// NEUES Observer-Pattern
pub fn process_player_input(
    trigger: On<FromClient<PlayerInput>>,
    mut players: Query<(&PlayerOwner, &mut Velocity)>,
) {
    let FromClient { client_id, message } = trigger.event();
    // Get the client entity from the ClientId
    let client_entity = match client_id.entity() {
        Some(e) => e,
        None => {
            warn!("Received input from invalid client: {:?}", client_id);
            return;
        }
    };

    // Find the player entity for this client
    for (owner, mut velocity) in &mut players {
        if owner.client_entity == client_entity {
            // Convert 2D input to 3D velocity (XZ plane)
            let move_vec = Vec3::new(message.direction.x, 0.0, message.direction.y);

            // Apply speed
            velocity.linear = if move_vec.length() > 0.01 {
                move_vec.normalize() * MOVE_SPEED
            } else {
                Vec3::ZERO
            };

            // TODO: Jump handling
            // if message.jump {
            //     velocity.linear.y = JUMP_FORCE;
            // }

            return;
        }
    }

    warn!("No player found for client {:?}", client_id);
}
