//! Player input processing from clients.

use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::shared::{Player, PlayerIdentity, PlayerMovement, Velocity};

/// Movement speed in units per second.
const MOVE_SPEED: f32 = 5.0;

/// Observer that processes player inputs from clients and updates velocity.
///
/// This is triggered automatically when a client sends a PlayerInput event.
/// Uses the Observer pattern (bevy_replicon 0.36+) instead of MessageReader.
pub fn process_player_input(
    trigger: On<FromClient<PlayerMovement>>,
    mut players: Query<(&PlayerIdentity, &mut Velocity, &mut Transform), With<Player>>,
    network_ids: Query<&bevy_replicon::shared::backend::connected_client::NetworkId>,
) {
    let FromClient { client_id, message } = trigger.event();

    // Get the NetworkId (u64) from the ClientId
    let network_id = network_ids
        .get(client_id.entity().expect("REASON")) // ClientId hat eine .entity() methode
        .expect("ClientId missing NetworkId");
    let incoming_client_id = network_id.get();

    for (player_identity, mut velocity, mut transform) in players.iter_mut() {
        if player_identity.client_id == incoming_client_id {
            *transform = message.transform;
            // Convert camera-relative input to world-relative movement

            // // Apply speed
            // velocity.linear = if move_vec.length() > 0.01 {
            //     move_vec.normalize() * MOVE_SPEED
            // } else {
            //     Vec3::ZERO
            // };
            // TODO: Acceleration handling
            // TODO: Jump handling
            // if message.jump {
            //     velocity.linear.y = JUMP_FORCE;
            // }

            return;
        }
    }

    warn!("No player found for client {:?}", client_id);
}
