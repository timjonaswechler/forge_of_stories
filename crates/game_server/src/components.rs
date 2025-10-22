//! Replicated game components for bevy_replicon.
//!
//! These components are automatically synchronized from server to clients.

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

/// Position component (server authoritative).
///
/// The server maintains this, clients receive updates via replication.
#[derive(Component, Debug, Clone, Copy, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Position {
    pub translation: Vec3,
}

/// Movement velocity component.
///
/// Replicated from server to clients for smooth interpolation.
#[derive(Component, Debug, Clone, Copy, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Velocity {
    pub linear: Vec3,
}

/// Component that links a player entity to its owning client entity.
///
/// This is NOT replicated - it's only used on the server to track which client owns which player.
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerOwner {
    /// The ConnectedClient entity that owns this player.
    pub client_entity: Entity,
}
