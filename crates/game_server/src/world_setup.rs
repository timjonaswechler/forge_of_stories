//! World setup and initialization systems.
//!
//! This module handles spawning the initial world state when the server starts.

use bevy::prelude::*;

use crate::components::Position;
use crate::world::GroundPlane;

/// System that spawns the initial world (ground plane).
///
/// This is called once during server initialization to set up the static world geometry.
pub fn spawn_world(world: &mut World) {
    info!("Server: Spawning world...");

    // Spawn ground plane at origin
    world.spawn((
        GroundPlane,
        Position {
            translation: Vec3::ZERO,
        },
        Name::new("Ground Plane"),
    ));

    // Note: Players are spawned when clients connect via the handle_client_connections system

    info!("Server: World spawned successfully");
}
