//! Player input processing from clients.

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use std::collections::HashMap;

use crate::shared::{Player, PlayerIdentity, PlayerMovement, Velocity};

/// Resource to buffer and track player input messages between server updates
#[derive(Resource, Default)]
pub struct ClientMessageBuffer {
    /// Buffered messages from each client since last server update (20Hz)
    pub buffered_messages: HashMap<u64, Vec<PlayerMovement>>,
}

/// Movement speed in units per second.
const MOVE_SPEED: f32 = 5.0;

/// Observer that buffers player inputs from clients.
///
/// This is triggered automatically when a client sends a PlayerInput event.
/// Messages are buffered and processed together in the FixedUpdate schedule.
/// Uses the Observer pattern (bevy_replicon 0.36+) instead of MessageReader.
pub fn buffer_player_input(
    trigger: On<FromClient<PlayerMovement>>,
    network_ids: Query<&bevy_replicon::shared::backend::connected_client::NetworkId>,
    mut buffer: ResMut<ClientMessageBuffer>,
) {
    let FromClient { client_id, message } = trigger.event();

    // Get the NetworkId (u64) from the ClientId
    let network_id = network_ids
        .get(client_id.entity().expect("REASON"))
        .expect("ClientId missing NetworkId");
    let incoming_client_id = network_id.get();

    // Buffer this message for processing in FixedUpdate
    buffer.buffered_messages
        .entry(incoming_client_id)
        .or_insert_with(Vec::new)
        .push(message.clone());
}

/// System that processes all buffered input messages in FixedUpdate (20Hz).
///
/// This runs once per server tick, processing all messages received since the last tick.
/// We use the most recent message from each client to determine the final state.
pub fn process_buffered_inputs(
    mut buffer: ResMut<ClientMessageBuffer>,
    mut players: Query<(&PlayerIdentity, &mut Velocity, &mut Transform), With<Player>>,
) {
    if buffer.buffered_messages.is_empty() {
        return;
    }

    info!("=== Processing buffered inputs (FixedUpdate @ 20Hz) ===");

    for (client_id, messages) in buffer.buffered_messages.iter() {
        let message_count = messages.len();
        info!("  Client {}: {} messages buffered", client_id, message_count);

        // Get the most recent message (last in the buffer)
        if let Some(latest_message) = messages.last() {
            // Find the player for this client
            for (player_identity, mut velocity, mut transform) in players.iter_mut() {
                if player_identity.client_id == *client_id {
                    // Update rotation from the latest message
                    transform.rotation = latest_message.transform.rotation;

                    // Update velocity based on movement direction
                    if latest_message.movement.length() > 0.01 {
                        velocity.linear = latest_message.movement.normalize() * MOVE_SPEED;
                        info!("    → Final velocity: {:?}", velocity.linear);
                    } else {
                        velocity.linear = Vec3::ZERO;
                        info!("    → Final velocity: ZERO (no movement)");
                    }

                    break;
                }
            }
        }
    }

    info!("=== Finished processing inputs ===");

    // Clear the buffer for the next server tick
    buffer.buffered_messages.clear();
}
