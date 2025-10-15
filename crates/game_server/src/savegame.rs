//! World save/load functionality.
//!
//! Provides serialization and deserialization of the server world state
//! to/from files on disk.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::components::{Player, Position, Velocity};
use crate::world::GroundPlane;

/// Serializable snapshot of the entire world state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    /// Format version (for future migration compatibility).
    pub version: u32,
    /// Timestamp when this save was created.
    pub timestamp: u64,
    /// All players in the world.
    pub players: Vec<PlayerData>,
    /// Ground plane (always exists, mostly for validation).
    pub has_ground_plane: bool,
}

impl Default for WorldSnapshot {
    fn default() -> Self {
        Self {
            version: 1,
            timestamp: 0,
            players: Vec::new(),
            has_ground_plane: true,
        }
    }
}

/// Serializable player entity data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    pub color: [f32; 4], // RGBA
    pub position: [f32; 3], // XYZ
    pub velocity: [f32; 3], // XYZ
}

/// Extracts a world snapshot from the server world.
pub fn extract_world_snapshot(world: &mut World) -> WorldSnapshot {
    let mut snapshot = WorldSnapshot::default();

    // Set timestamp (using system time)
    snapshot.timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Check for ground plane
    snapshot.has_ground_plane = world.query::<&GroundPlane>().iter(world).count() > 0;

    // Extract all players
    for (player, position, velocity) in world
        .query::<(&Player, &Position, &Velocity)>()
        .iter(world)
    {
        snapshot.players.push(PlayerData {
            color: player.color.to_srgba().to_f32_array(),
            position: position.translation.to_array(),
            velocity: velocity.linear.to_array(),
        });
    }

    info!(
        "Extracted world snapshot: {} players",
        snapshot.players.len()
    );
    snapshot
}

/// Saves a world snapshot to a file (RON format).
pub fn save_world_to_file<P: AsRef<Path>>(world: &mut World, path: P) -> Result<(), SaveError> {
    let snapshot = extract_world_snapshot(world);

    let ron_string = ron::ser::to_string_pretty(&snapshot, ron::ser::PrettyConfig::default())
        .map_err(|e| SaveError::Serialization(e.to_string()))?;

    std::fs::write(path.as_ref(), ron_string).map_err(|e| SaveError::Io(e.to_string()))?;

    info!("World saved to {}", path.as_ref().display());
    Ok(())
}

/// Loads a world snapshot from a file and restores it to the world.
pub fn load_world_from_file<P: AsRef<Path>>(world: &mut World, path: P) -> Result<(), SaveError> {
    let ron_string =
        std::fs::read_to_string(path.as_ref()).map_err(|e| SaveError::Io(e.to_string()))?;

    let snapshot: WorldSnapshot =
        ron::from_str(&ron_string).map_err(|e| SaveError::Deserialization(e.to_string()))?;

    restore_world_snapshot(world, snapshot)?;

    info!("World loaded from {}", path.as_ref().display());
    Ok(())
}

/// Restores a world snapshot into the server world.
///
/// This clears all existing players and spawns new ones from the snapshot.
pub fn restore_world_snapshot(world: &mut World, snapshot: WorldSnapshot) -> Result<(), SaveError> {
    // Despawn all existing players
    let player_entities: Vec<_> = world
        .query_filtered::<Entity, With<Player>>()
        .iter(world)
        .collect();

    for entity in player_entities {
        world.despawn(entity);
    }

    // Spawn players from snapshot
    let player_count = snapshot.players.len();
    for player_data in snapshot.players {
        world.spawn((
            Player {
                color: Color::srgba(
                    player_data.color[0],
                    player_data.color[1],
                    player_data.color[2],
                    player_data.color[3],
                ),
            },
            Position {
                translation: Vec3::from_array(player_data.position),
            },
            Velocity {
                linear: Vec3::from_array(player_data.velocity),
            },
            Name::new("Player"),
        ));
    }

    info!("Restored world snapshot: {} players", player_count);
    Ok(())
}

/// Error types for save/load operations.
#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Deserialization error: {0}")]
    Deserialization(String),
}
