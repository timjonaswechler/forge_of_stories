//! World setup and initialization systems.
//!
//! This module handles spawning the initial world state when the server starts.

use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;

use crate::components::Position;
use crate::world::{GroundPlane, GroundPlaneSize};

/// System that spawns the initial world (ground plane).
///
/// This is called once during server initialization to set up the static world geometry.
pub fn spawn_world(commands: &mut Commands) {
    // Spawn ground plane at origin with size
    commands.spawn((
        GroundPlane,
        Position {
            translation: Vec3::ZERO,
        },
        GroundPlaneSize {
            width: 40.0,
            height: 0.25,
            depth: 40.0,
        },
        Replicated,
    ));
}
