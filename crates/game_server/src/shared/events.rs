//! Network events for client-server communication.
//!
//! These events are sent between client and server using bevy_replicon's event system.

use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};

/// Player input event sent from client to server.
///
/// Sent with `Channel::Unreliable` since newer inputs supersede older ones.
#[derive(MapEntities, Debug, Clone, Event, Serialize, Deserialize)]
pub struct PlayerInput {
    /// Movement direction in 2D (XZ plane), normalized by the client.
    pub direction: Vec2,
    /// Whether the player is trying to jump.
    pub jump: bool,
}
