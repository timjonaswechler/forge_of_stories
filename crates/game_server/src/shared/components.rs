//! Shared replicated components for bevy_replicon.
//!
//! These components are automatically synchronized from server to clients.
//! Both server and client must register these for replication to work.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Player entity marker component.
///
/// This component marks an entity as a player and stores their color.
/// The relationship between ConnectedClient and Player is maintained separately.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Player {
    /// Player's assigned color.
    pub color: Color,
}

/// Identifies which client owns a replicated player entity.
///
/// The contained `client_id` is the unique identifier negotiated during the
/// netcode handshake so that clients can recognise their own replicated entity.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct PlayerIdentity {
    /// Globally unique identifier of the owning client (Renet client id).
    pub client_id: u64,
}

/// Movement velocity component.
///
/// Replicated from server to clients for smooth interpolation.
#[derive(Component, Debug, Clone, Copy, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Velocity {
    pub linear: Vec3,
}

/// Links a player entity to its ConnectedClient entity on the server.
///
/// **Server-only** - This is NOT replicated to clients!
/// Used only on the server to track which ConnectedClient entity owns which player entity.
/// This is needed for cleanup when clients disconnect.
#[derive(Component, Debug, Clone, Copy)]
pub struct ServerPlayerConnection {
    /// The ConnectedClient entity that owns this player.
    pub connected_client_entity: Entity,
}

// Compatibility alias for gradual migration
pub type PlayerOwner = ServerPlayerConnection;
