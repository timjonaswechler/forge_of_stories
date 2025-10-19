//! Server initialization for embedded game server.
//!
//! This module provides helper functions to start the embedded game server
//! that runs in a separate thread with bevy_replicon.

use bevy::prelude::*;
use game_server::{Port, ServerHandle};

const DEFAULT_PORT: Port = Port(7777);

/// Starts an embedded server in Singleplayer mode.
///
/// This creates a GameServer with loopback transport only (localhost TCP connection).
/// The server runs in a separate thread with its own Bevy App.
///
/// # Returns
/// A `ServerHandle` to control the server thread.
pub fn start_singleplayer_server() -> ServerHandle {
    info!(
        "Starting embedded server in Singleplayer mode on port {}",
        DEFAULT_PORT.0
    );

    ServerHandle::start_embedded(DEFAULT_PORT)
}
