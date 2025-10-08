//! World setup and initialization systems.
//!
//! This module handles spawning the initial world state when the server starts.

use bevy::prelude::*;
use tracing::info;

use crate::world::{GroundPlane, PlayerColorAssigner, Position};
/// Resource to track if the world has been initialized.
#[derive(Resource, Default)]
pub struct WorldInitialized(pub bool);

/// Direct world initialization (for EmbeddedServer).
///
/// This is called manually when creating an EmbeddedServer, bypassing Bevy's Startup schedule.
pub fn initialize_world_direct(world: &mut World) {
    info!("Server: Initializing world resources...");

    world.insert_resource(PlayerColorAssigner::default());
    world.insert_resource(WorldInitialized(false));

    info!("Server: World resources initialized");
}

/// Direct world spawning (for EmbeddedServer).
///
/// This is called manually when creating an EmbeddedServer, bypassing Bevy's Startup schedule.
pub fn spawn_world_direct(world: &mut World) {
    info!("Server: Spawning world...");

    // Spawn ground plane at origin
    world.spawn((
        GroundPlane,
        Position {
            translation: Vec3::ZERO,
        },
        Name::new("Ground Plane"),
    ));

    // Note: Players are spawned when clients connect, not during world initialization
    // The client will trigger a connection event which spawns the player

    info!("Server: World spawned successfully");
}

/// System that spawns the initial world (ground plane).
///
/// Runs once on server startup (for ServerLogicPlugin).
pub fn spawn_world(mut commands: Commands) {
    info!("Server: Spawning world...");

    // Spawn ground plane at origin
    commands.spawn((
        GroundPlane,
        Position {
            translation: Vec3::ZERO,
        },
        Name::new("Ground Plane"),
    ));

    info!("Server: World spawned successfully");
}

/// Startup system that initializes world resources (for ServerLogicPlugin).
pub fn initialize_world(mut commands: Commands) {
    info!("Server: Initializing world resources...");

    commands.insert_resource(PlayerColorAssigner::default());
    commands.insert_resource(WorldInitialized(false));

    info!("Server: World resources initialized");
}

/// System that marks world as initialized after spawn.
pub fn mark_world_initialized(mut initialized: ResMut<WorldInitialized>) {
    if !initialized.0 {
        initialized.0 = true;
        info!("Server: World marked as initialized");
    }
}
