//! Network protocol for gameplay synchronization.
//!
//! This module defines the messages exchanged between client and server for
//! multiplayer gameplay (player spawning, movement, state updates).
//!
//! Uses `network_shared` transport primitives and `ClientSideConnection::send_message()`
//! for serialization and transmission.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::world::PlayerShape;

// ============================================================================
// Server → Client Messages
// ============================================================================

/// Message sent when a new player joins the game.
///
/// The server broadcasts this to all connected clients so they can spawn
/// the player entity locally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSpawnMessage {
    /// Unique player ID (matches ClientId from networking).
    pub player_id: u64,
    /// Player's assigned color.
    pub color: SerializableColor,
    /// Player's shape type.
    pub shape: PlayerShape,
    /// Initial spawn position.
    pub position: SerializableVec3,
}

/// Message sent when a player leaves the game.
///
/// The server broadcasts this to all clients so they can despawn the player entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDespawnMessage {
    /// Player ID that left.
    pub player_id: u64,
}

/// Bulk state update for all entities in the world.
///
/// The server broadcasts this every tick (or at a lower rate) to sync
/// all entity positions and velocities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldStateMessage {
    /// Server tick number (for client interpolation/extrapolation).
    pub tick: u64,
    /// All player states.
    pub players: Vec<PlayerStateSnapshot>,
}

/// Individual player state (position, velocity).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStateSnapshot {
    pub player_id: u64,
    pub position: SerializableVec3,
    pub velocity: SerializableVec3,
}

// ============================================================================
// Client → Server Messages
// ============================================================================

/// Player input sent from client to server.
///
/// The client sends this every frame (or when input changes) so the server
/// can update the player's authoritative state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInputMessage {
    /// Movement direction (normalized, -1.0 to 1.0 per axis).
    pub movement: SerializableVec3,
    /// Client tick when this input was generated (for lag compensation).
    pub client_tick: u64,
}

// ============================================================================
// Message Envelope
// ============================================================================

/// Top-level message envelope for gameplay messages.
///
/// All gameplay messages are wrapped in this enum for routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameplayMessage {
    // Server → Client
    PlayerSpawn(PlayerSpawnMessage),
    PlayerDespawn(PlayerDespawnMessage),
    WorldState(WorldStateMessage),

    // Client → Server
    PlayerInput(PlayerInputMessage),
}

// ============================================================================
// Serializable Types (for Bevy types that don't impl Serialize)
// ============================================================================

/// Serializable wrapper for Vec3.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SerializableVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for SerializableVec3 {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<SerializableVec3> for Vec3 {
    fn from(s: SerializableVec3) -> Self {
        Vec3::new(s.x, s.y, s.z)
    }
}

/// Serializable wrapper for Color.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SerializableColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<Color> for SerializableColor {
    fn from(c: Color) -> Self {
        let linear = c.to_linear();
        Self {
            r: linear.red,
            g: linear.green,
            b: linear.blue,
            a: linear.alpha,
        }
    }
}

impl From<SerializableColor> for Color {
    fn from(s: SerializableColor) -> Self {
        Color::linear_rgba(s.r, s.g, s.b, s.a)
    }
}

// ============================================================================
// Channel Configuration
// ============================================================================

/// Network channel IDs for gameplay messages.
pub mod channels {
    use network_shared::channels::{ChannelId, ChannelKind, ChannelsConfiguration};

    /// Reliable ordered channel for critical gameplay events (spawn, despawn).
    pub const GAMEPLAY_EVENTS: ChannelId = 0;

    /// Reliable ordered channel for player input (client → server).
    pub const PLAYER_INPUT: ChannelId = 1;

    /// Reliable ordered channel for world state updates (server → client).
    /// TODO: Consider unreliable channel for high-frequency updates.
    pub const WORLD_STATE: ChannelId = 2;

    /// Creates the standard channel configuration for gameplay.
    ///
    /// This must match the channel IDs defined above.
    pub fn create_gameplay_channels() -> ChannelsConfiguration {
        ChannelsConfiguration::from_types(vec![
            // Channel 0: GAMEPLAY_EVENTS - Critical events like spawn/despawn
            ChannelKind::OrderedReliable {
                max_frame_size: 10 * 1024, // 10 KB
            },
            // Channel 1: PLAYER_INPUT - Player input from client to server
            ChannelKind::OrderedReliable {
                max_frame_size: 1024, // 1 KB (small messages)
            },
            // Channel 2: WORLD_STATE - Position/velocity updates
            ChannelKind::OrderedReliable {
                max_frame_size: 64 * 1024, // 64 KB (larger for bulk updates)
            },
        ])
        .expect("Failed to create gameplay channels configuration")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_conversion() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let s: SerializableVec3 = v.into();
        assert_eq!(s.x, 1.0);
        assert_eq!(s.y, 2.0);
        assert_eq!(s.z, 3.0);

        let v2: Vec3 = s.into();
        assert_eq!(v2, v);
    }

    #[test]
    fn test_color_conversion() {
        let c = Color::srgb(1.0, 0.5, 0.0);
        let s: SerializableColor = c.into();
        let c2: Color = s.into();

        // Colors should be approximately equal (accounting for sRGB → linear conversion)
        let diff = (c.to_linear().red - c2.to_linear().red).abs();
        assert!(diff < 0.01, "Color conversion failed");
    }

    #[test]
    fn test_message_types() {
        // Just verify that messages can be constructed
        let _spawn = GameplayMessage::PlayerSpawn(PlayerSpawnMessage {
            player_id: 42,
            color: Color::srgb(1.0, 0.0, 0.0).into(),
            shape: PlayerShape::Capsule,
            position: Vec3::new(10.0, 5.0, 20.0).into(),
        });

        let _input = GameplayMessage::PlayerInput(PlayerInputMessage {
            movement: Vec3::new(1.0, 0.0, 0.0).into(),
            client_tick: 100,
        });

        let _despawn = GameplayMessage::PlayerDespawn(PlayerDespawnMessage { player_id: 42 });

        let _state = GameplayMessage::WorldState(WorldStateMessage {
            tick: 100,
            players: vec![],
        });
    }
}
