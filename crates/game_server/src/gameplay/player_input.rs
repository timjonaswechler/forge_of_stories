//! Player input processing from clients.

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use std::collections::HashMap;

use crate::shared::{Player, PlayerIdentity, PlayerMovement, Velocity};

/// Resource to track message counts per client between server updates
#[derive(Resource, Default)]
pub struct ClientMessageStats {
    /// Count of messages received from each client since last server update (20Hz)
    pub message_counts: HashMap<u64, usize>,
}

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
    mut stats: ResMut<ClientMessageStats>,
) {
    let FromClient { client_id, message } = trigger.event();

    // Get the NetworkId (u64) from the ClientId
    let network_id = network_ids
        .get(client_id.entity().expect("REASON")) // ClientId hat eine .entity() methode
        .expect("ClientId missing NetworkId");
    let incoming_client_id = network_id.get();

    // Track message count for this client
    *stats.message_counts.entry(incoming_client_id).or_insert(0) += 1;

    for (player_identity, mut velocity, mut transform) in players.iter_mut() {
        if player_identity.client_id == incoming_client_id {
            // Rotation vom Client übernehmen (Kamera-basiert)
            transform.rotation = message.transform.rotation;

            // Setze Velocity basierend auf Bewegungsrichtung
            // Das System apply_velocity wird dann im Server-Update (20Hz) die Position aktualisieren
            if message.movement.length() > 0.01 {
                velocity.linear = message.movement.normalize() * MOVE_SPEED;
                info!("Server setting velocity: {:?}", velocity.linear);
            } else {
                velocity.linear = Vec3::ZERO;
            }

            // TODO: Jump handling
            // if message.jump {
            //     velocity.linear.y = JUMP_FORCE;
            // }

            return;
        }
    }

    warn!("No player found for client {:?}", client_id);
}

/// System that logs message statistics per client between each server update (20Hz)
pub fn log_client_message_stats(
    mut stats: ResMut<ClientMessageStats>,
) {
    if !stats.message_counts.is_empty() {
        info!("=== Messages since last server update (20Hz) ===");
        for (client_id, count) in stats.message_counts.iter() {
            info!("  Client {}: {} messages", client_id, count);
        }

        // Reset counts for next server update
        stats.message_counts.clear();
    }
}
