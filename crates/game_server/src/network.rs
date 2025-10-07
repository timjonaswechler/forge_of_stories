//! Network event handling systems for server-side multiplayer.
//!
//! This module handles:
//! - Player connections/disconnections
//! - Broadcasting gameplay messages to all clients
//! - Processing client input messages

use crate::protocol::{
    GameplayMessage, PlayerDespawnMessage, PlayerShape, PlayerSpawnMessage, PlayerStateSnapshot,
    WorldStateMessage,
};
use crate::{
    movement::{PlayerInput, PlayerInputQueue},
    world::{Player, PlayerColorAssigner, Position, Velocity},
};
use bevy::prelude::*;
use shared::{ClientId, TransportEvent};
use tracing::{debug, error, info, warn};

/// Resource that holds network events from the transport layer.
///
/// The `EmbeddedServer` polls transport events and inserts them here,
/// then our systems process them.
#[derive(Resource, Default)]
pub struct NetworkEvents {
    pub events: Vec<TransportEvent>,
}

/// Resource that holds outgoing messages to be sent to clients.
///
/// Systems add messages here, then `EmbeddedServer` sends them.
#[derive(Resource, Default)]
pub struct OutgoingMessages {
    pub messages: Vec<(Option<ClientId>, GameplayMessage)>, // (target_client, message) - None = broadcast
}

/// Resource that tracks currently connected clients.
///
/// Updated immediately when clients connect/disconnect (before Commands are applied).
#[derive(Resource, Default)]
pub struct ConnectedClients {
    pub clients: Vec<ClientId>,
}

/// System that processes incoming network events (connections, disconnections).
///
/// Runs in `ServerSet::Input`.
pub fn process_network_events(
    mut events: ResMut<NetworkEvents>,
    mut outgoing: ResMut<OutgoingMessages>,
    mut connected_clients: ResMut<ConnectedClients>,
    mut commands: Commands,
    mut color_assigner: ResMut<PlayerColorAssigner>,
    mut input_queue: ResMut<PlayerInputQueue>,
    players: Query<(Entity, &Player)>,
) {
    for event in events.events.drain(..) {
        match event {
            TransportEvent::PeerConnected { client } => {
                info!("Client {} connected, spawning player", client);

                // Add to connected clients list immediately (before Commands are applied)
                connected_clients.clients.push(client);
                info!("Connected clients now: {:?}", connected_clients.clients);

                // First, send all existing players to the new client BEFORE spawning
                // (This ensures the new client sees existing players immediately)
                for (_entity, player) in &players {
                    // Send existing player to new client
                    let existing_spawn = GameplayMessage::PlayerSpawn(PlayerSpawnMessage {
                        player_id: player.id,
                        color: player.color.into(),
                        shape: PlayerShape::default(),
                        position: Vec3::new(0.0, 1.0, 0.0).into(), // TODO: Get actual position
                    });

                    info!(
                        "Sending existing player {} to new client {}",
                        player.id, client
                    );
                    outgoing.messages.push((Some(client), existing_spawn));
                }

                // Assign color
                let color = color_assigner.next_color();

                // Spawn player entity in server world
                let entity = commands.spawn((
                    Player { id: client, color },
                    PlayerShape::default(),
                    Position {
                        translation: Vec3::new(0.0, 1.0, 0.0),
                    },
                    Velocity::default(),
                ));

                info!(
                    "Spawned player entity {:?} for client {}",
                    entity.id(),
                    client
                );

                // Broadcast spawn message to ALL clients (including the new one)
                let spawn_msg = GameplayMessage::PlayerSpawn(PlayerSpawnMessage {
                    player_id: client,
                    color: color.into(),
                    shape: PlayerShape::default(),
                    position: Vec3::new(0.0, 1.0, 0.0).into(),
                });

                info!("Broadcasting spawn of player {} to all clients", client);
                outgoing.messages.push((None, spawn_msg)); // None = broadcast to all
            }
            TransportEvent::PeerDisconnected { client, .. } => {
                info!("Client {} disconnected", client);

                // Remove from connected clients list
                connected_clients.clients.retain(|&c| c != client);
                info!("Connected clients now: {:?}", connected_clients.clients);

                // TODO: Find player by client_id and despawn
                // TODO: Broadcast despawn message
            }
            TransportEvent::Message {
                client,
                channel,
                payload,
            } => {
                // Deserialize the gameplay message
                match bincode::serde::decode_from_slice::<GameplayMessage, _>(
                    &payload,
                    bincode::config::standard(),
                ) {
                    Ok((message, _)) => {
                        handle_gameplay_message(client, channel, message, &mut input_queue);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to deserialize message from client {}: {:?}",
                            client, e
                        );
                    }
                }
            }
            TransportEvent::Datagram { client, payload } => {
                info!(
                    "Received datagram from client {}: {} bytes",
                    client,
                    payload.len()
                );
            }
            TransportEvent::Error { error, .. } => {
                error!("Network error: {:?}", error);
            }
            TransportEvent::AuthResult { .. } => {
                // TODO: Handle Steam auth results (Phase 9)
                debug!("Auth result received (not yet implemented)");
            }
        }
    }
}

/// Handles incoming gameplay messages from clients.
fn handle_gameplay_message(
    client: ClientId,
    _channel: u8,
    message: GameplayMessage,
    input_queue: &mut PlayerInputQueue,
) {
    match message {
        GameplayMessage::PlayerInput(input_msg) => {
            // Convert network message to internal PlayerInput
            let movement_vec: Vec3 = input_msg.movement.into();
            let input = PlayerInput {
                direction: Vec2::new(movement_vec.x, movement_vec.z), // XZ plane
                jump: false, // Jump not yet implemented in protocol
            };

            // Queue input for this player
            input_queue.inputs.insert(client, input);
            debug!(
                "Received input from client {}: direction={:?}",
                client, input.direction
            );
        }
        GameplayMessage::PlayerSpawn(_)
        | GameplayMessage::PlayerDespawn(_)
        | GameplayMessage::WorldState(_) => {
            // These are server->client messages, ignore if received from client
            warn!("Client {} sent server-only message: {:?}", client, message);
        }
    }
}

/// System that broadcasts world state updates to clients.
///
/// Runs in `ServerSet::Replication` after simulation.
/// Uses change detection to only send entities that have moved.
pub fn broadcast_world_state(
    mut outgoing: ResMut<OutgoingMessages>,
    query: Query<(&Player, &Position, &Velocity), Changed<Position>>,
) {
    // Collect all players that have moved this frame
    let mut players = Vec::new();

    for (player, position, velocity) in &query {
        players.push(PlayerStateSnapshot {
            player_id: player.id,
            position: position.translation.into(),
            velocity: velocity.linear.into(),
        });
    }

    // Only broadcast if there are updates
    if !players.is_empty() {
        let player_count = players.len();
        let message = GameplayMessage::WorldState(WorldStateMessage {
            tick: 0, // TODO: Track server tick counter
            players,
        });

        // Broadcast to all clients
        outgoing.messages.push((None, message));
        debug!(
            "Broadcasting world state with {} player updates",
            player_count
        );
    }
}

/// System that handles player disconnections.
///
/// Despawns player entities and broadcasts despawn messages.
pub fn handle_disconnections(
    _outgoing: ResMut<OutgoingMessages>,
    _commands: Commands,
    // TODO: Track disconnected clients
) {
    // TODO: Implement once we have proper connection tracking
}
