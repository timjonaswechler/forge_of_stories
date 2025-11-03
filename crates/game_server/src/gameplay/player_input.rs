//! Player input processing from clients.

use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::shared::{PlayerInput, PlayerOwner, Velocity};

/// Movement speed in units per second.
const MOVE_SPEED: f32 = 5.0;

/// Observer that processes player inputs from clients and updates velocity.
///
/// This is triggered automatically when a client sends a PlayerInput event.
/// Uses the Observer pattern (bevy_replicon 0.36+) instead of MessageReader.
pub fn process_player_input(
    trigger: On<FromClient<PlayerInput>>,
    mut players: Query<(&PlayerOwner, &mut Velocity)>,
) {
    let FromClient { client_id, message } = trigger.event();

    // Get the client entity from the ClientId
    let Some(client_entity) = client_id.entity() else {
        warn!("Received input from invalid client: {:?}", client_id);
        return;
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
