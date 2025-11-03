//! Network messages for client-server communication.

use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};

/// Player input message sent from client to server.
#[derive(MapEntities, Debug, Clone, Event, Serialize, Deserialize)]
pub struct PlayerInput {
    /// Movement direction in 2D (XZ plane), normalized by the client.
    pub direction: Vec2,
    /// Whether the player is trying to jump.
    pub jump: bool,
}
