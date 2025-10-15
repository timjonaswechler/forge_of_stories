//! Client messages for bevy_replicon.
//!
//! These messages are sent from clients to the server.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Player input message sent from client to server.
///
/// Clients send this message every frame with their current input state.
/// The server processes these inputs and updates player velocity accordingly.
#[derive(Message, Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PlayerInput {
    /// Movement direction (normalized, in world space XZ plane).
    pub direction: Vec2,
    /// Jump requested.
    pub jump: bool,
}
