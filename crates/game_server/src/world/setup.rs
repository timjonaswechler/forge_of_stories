//! World setup and initialization.

use crate::world::{GroundPlane, GroundPlaneSize};
use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;

/// Spawns the initial world (ground plane).
///
/// This is called on the first client connection to set up the static world geometry.
pub fn spawn_world(commands: &mut Commands) {
    // Spawn ground plane at origin
    commands.spawn((
        GroundPlane,
        Transform {
            translation: Vec3::new(0.0, -0.125, 0.0),
            ..Default::default()
        },
        GroundPlaneSize {
            width: 40.0,
            height: 0.25,
            depth: 40.0,
        },
        Replicated,
    ));
}
