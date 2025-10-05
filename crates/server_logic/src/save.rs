//! World save/load functionality.
//!
//! Provides serialization and deserialization of the server world state
//! to/from files on disk.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

use crate::world::{GroundPlane, Player, PlayerShape, Position, Velocity};

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
    pub id: u64,
    pub color: [f32; 4], // RGBA
    pub shape: PlayerShapeData,
    pub position: [f32; 3], // XYZ
    pub velocity: [f32; 3], // XYZ
}

/// Serializable player shape enum.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PlayerShapeData {
    Cube,
    Sphere,
    Capsule,
}

impl From<PlayerShape> for PlayerShapeData {
    fn from(shape: PlayerShape) -> Self {
        match shape {
            PlayerShape::Cube => PlayerShapeData::Cube,
            PlayerShape::Sphere => PlayerShapeData::Sphere,
            PlayerShape::Capsule => PlayerShapeData::Capsule,
        }
    }
}

impl From<PlayerShapeData> for PlayerShape {
    fn from(data: PlayerShapeData) -> Self {
        match data {
            PlayerShapeData::Cube => PlayerShape::Cube,
            PlayerShapeData::Sphere => PlayerShape::Sphere,
            PlayerShapeData::Capsule => PlayerShape::Capsule,
        }
    }
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
    for (player, position, velocity, shape) in world
        .query::<(&Player, &Position, &Velocity, &PlayerShape)>()
        .iter(world)
    {
        snapshot.players.push(PlayerData {
            id: player.id,
            color: player.color.to_srgba().to_f32_array(),
            shape: (*shape).into(),
            position: position.translation.to_array(),
            velocity: velocity.linear.to_array(),
        });
    }

    info!("Extracted world snapshot: {} players", snapshot.players.len());
    snapshot
}

/// Saves a world snapshot to a file (RON format).
pub fn save_world_to_file<P: AsRef<Path>>(
    world: &mut World,
    path: P,
) -> Result<(), SaveError> {
    let snapshot = extract_world_snapshot(world);

    let ron_string = ron::ser::to_string_pretty(&snapshot, ron::ser::PrettyConfig::default())
        .map_err(|e| SaveError::Serialization(e.to_string()))?;

    std::fs::write(path.as_ref(), ron_string)
        .map_err(|e| SaveError::Io(e.to_string()))?;

    info!("World saved to {}", path.as_ref().display());
    Ok(())
}

/// Loads a world snapshot from a file and restores it to the world.
pub fn load_world_from_file<P: AsRef<Path>>(
    world: &mut World,
    path: P,
) -> Result<(), SaveError> {
    let ron_string = std::fs::read_to_string(path.as_ref())
        .map_err(|e| SaveError::Io(e.to_string()))?;

    let snapshot: WorldSnapshot = ron::from_str(&ron_string)
        .map_err(|e| SaveError::Deserialization(e.to_string()))?;

    restore_world_snapshot(world, snapshot)?;

    info!("World loaded from {}", path.as_ref().display());
    Ok(())
}

/// Restores a world snapshot into the server world.
///
/// This clears all existing players and spawns new ones from the snapshot.
pub fn restore_world_snapshot(
    world: &mut World,
    snapshot: WorldSnapshot,
) -> Result<(), SaveError> {
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
                id: player_data.id,
                color: Color::srgba(
                    player_data.color[0],
                    player_data.color[1],
                    player_data.color[2],
                    player_data.color[3],
                ),
            },
            PlayerShape::from(player_data.shape),
            Position {
                translation: Vec3::from_array(player_data.position),
            },
            Velocity {
                linear: Vec3::from_array(player_data.velocity),
            },
            Name::new(format!("Player {}", player_data.id)),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_and_restore_snapshot() {
        let mut world = World::new();

        // Spawn a test player
        world.spawn((
            Player {
                id: 1,
                color: Color::srgb(1.0, 0.0, 0.0),
            },
            PlayerShape::Capsule,
            Position {
                translation: Vec3::new(1.0, 2.0, 3.0),
            },
            Velocity {
                linear: Vec3::new(0.5, 0.0, 0.0),
            },
        ));

        // Extract snapshot
        let snapshot = extract_world_snapshot(&mut world);
        assert_eq!(snapshot.players.len(), 1);
        assert_eq!(snapshot.players[0].id, 1);
        assert_eq!(snapshot.players[0].position, [1.0, 2.0, 3.0]);

        // Clear world
        let entities: Vec<_> = world
            .query_filtered::<Entity, With<Player>>()
            .iter(&world)
            .collect();
        for entity in entities {
            world.despawn(entity);
        }

        // Restore snapshot
        restore_world_snapshot(&mut world, snapshot).unwrap();

        // Verify restored
        let player_count = world.query::<&Player>().iter(&world).count();
        assert_eq!(player_count, 1);
    }

    #[test]
    fn test_save_and_load_file() {
        use std::fs;
        use tempfile::NamedTempFile;

        let mut world = World::new();

        // Spawn test player
        world.spawn((
            Player {
                id: 42,
                color: Color::srgb(0.5, 0.5, 1.0),
            },
            PlayerShape::Cube,
            Position {
                translation: Vec3::new(5.0, 10.0, 15.0),
            },
            Velocity::default(),
        ));

        // Save to temp file
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        save_world_to_file(&mut world, path).unwrap();

        // Verify file exists and has content
        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("version"));
        assert!(content.contains("players"));

        // Clear world
        let entities: Vec<_> = world
            .query_filtered::<Entity, With<Player>>()
            .iter(&world)
            .collect();
        for entity in entities {
            world.despawn(entity);
        }

        // Load from file
        load_world_from_file(&mut world, path).unwrap();

        // Verify loaded
        let player = world.query::<&Player>().iter(&world).next().unwrap();
        assert_eq!(player.id, 42);
    }
}
