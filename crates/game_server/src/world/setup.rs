//! World setup and initialization.

use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;

use crate::shared::Position;
use crate::world::{GroundPlane, GroundPlaneSize};

/// Spawns the initial world (ground plane).
///
/// This is called on the first client connection to set up the static world geometry.
pub fn spawn_world(commands: &mut Commands) {
    // Spawn ground plane at origin
    commands.spawn((
        GroundPlane,
        Position {
            translation: Vec3::new(0.0, -0.125, 0.0),
        },
        GroundPlaneSize {
            width: 40.0,
            height: 0.25,
            depth: 40.0,
        },
        Replicated,
    ));
}
