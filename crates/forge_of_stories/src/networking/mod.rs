//! Client networking layer.
//!
//! This module handles:
//! - Client connection to server (embedded or dedicated)
//! - Replication setup (registering replicated components)
//! - Local player tracking

pub mod client;
pub mod replication;

pub use client::*;
pub use replication::*;

use bevy::prelude::*;

/// Main networking plugin for the client.
///
/// This plugin sets up:
/// - Replicon client networking
/// - Component replication registration
/// - Local player tracking
pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ClientPlugin, ReplicationPlugin));
    }
}
