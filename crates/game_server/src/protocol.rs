//! Network protocol for gameplay synchronization.
//!
//! This module defines the messages exchanged between client and server for
//! multiplayer gameplay (player spawning, movement, state updates).
//!
//! Uses `shared` transport primitives and `ClientSideConnection::send_message()`
//! for serialization and transmission.

use crate::world::Player;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use shared::channels::ChannelId;

// ============================================================================
// Shared Game Types
// ============================================================================

/// Player shape type (for rendering).
///
/// Defines the visual representation of a player entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerShape {
    Cube,
    Capsule,
}

impl Default for PlayerShape {
    fn default() -> Self {
        Self::Capsule
    }
}

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
    pub player_id: uuid::Uuid,
    /// Player's assigned color.
    pub color: SerializableColor,
    /// Player's shape type.
    pub shape: PlayerShape,
    /// Initial spawn position.
    pub position: SerializableVec3,
}

impl PlayerSpawnMessage {
    /// Create a spawn message from a Player component and position.
    ///
    /// This is used when sending existing players to newly connected clients.
    pub fn from_player(player: &Player, position: Vec3, shape: PlayerShape) -> Self {
        Self {
            player_id: player.id,
            color: player.color.into(),
            shape,
            position: position.into(),
        }
    }

    /// Create a new spawn message with default shape.
    ///
    /// This is used when spawning a new player for the first time.
    pub fn new(player_id: uuid::Uuid, color: Color, position: Vec3) -> Self {
        Self {
            player_id,
            color: color.into(),
            shape: PlayerShape::default(),
            position: position.into(),
        }
    }
}

/// Message sent when a player leaves the game.
///
/// The server broadcasts this to all clients so they can despawn the player entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDespawnMessage {
    /// Player ID that left.
    pub player_id: uuid::Uuid,
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
    pub player_id: uuid::Uuid,
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

impl GameplayMessage {
    pub const CHANNELS: [(ChannelId, &'static str); 3] = [
        (channels::GAMEPLAY_EVENTS, "PlayerSpawn|PlayerDespawn"),
        (channels::PLAYER_INPUT, "PlayerInput"),
        (channels::WORLD_STATE, "WorldState"),
    ];
    pub fn channel(&self) -> ChannelId {
        match self {
            Self::PlayerSpawn(_) | Self::PlayerDespawn(_) => channels::GAMEPLAY_EVENTS,
            Self::PlayerInput(_) => channels::PLAYER_INPUT,
            Self::WorldState(_) => channels::WORLD_STATE,
        }
    }
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
    use shared::channels::{ChannelId, ChannelKind, ChannelsConfiguration};

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
        use uuid::uuid;
        // Just verify that messages can be constructed
        let _spawn = GameplayMessage::PlayerSpawn(PlayerSpawnMessage {
            player_id: uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
            color: Color::srgb(1.0, 0.0, 0.0).into(),
            shape: PlayerShape::Capsule,
            position: Vec3::new(10.0, 5.0, 20.0).into(),
        });

        let _input = GameplayMessage::PlayerInput(PlayerInputMessage {
            movement: Vec3::new(1.0, 0.0, 0.0).into(),
            client_tick: 100,
        });

        let _despawn = GameplayMessage::PlayerDespawn(PlayerDespawnMessage {
            player_id: uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
        });

        let _state = GameplayMessage::WorldState(WorldStateMessage {
            tick: 100,
            players: vec![],
        });
    }
}
