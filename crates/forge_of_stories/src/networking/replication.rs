//! Replication setup - registers all components that should be replicated from server.

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use game_server::{GroundPlane, GroundPlaneSize, Player, PlayerIdentity, Position, Velocity};

/// Plugin that registers all replicated components.
///
/// **IMPORTANT**: This must match exactly what the server registers!
/// See `game_server::plugin::ServerPlugin` for the server-side registration.
pub struct ReplicationPlugin;

impl Plugin for ReplicationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Player components
            .replicate::<Player>()
            .replicate::<PlayerIdentity>()
            .replicate::<Position>()
            .replicate::<Velocity>()
            // World components
            .replicate::<GroundPlane>()
            .replicate::<GroundPlaneSize>();
    }
}
