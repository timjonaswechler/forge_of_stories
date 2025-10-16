//! Server initialization for embedded game server.
//!
//! This module provides helper functions to start the embedded game server
//! that runs in a separate thread with bevy_replicon.

use bevy::prelude::*;
use game_server::ServerHandle;

pub const DEFAULT_PORT: u16 = 7777;

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
        DEFAULT_PORT
    );

    ServerHandle::start_embedded(DEFAULT_PORT)
}

/// System that monitors the server state and logs changes.
///
/// Add this to your app for debugging server lifecycle.
pub fn monitor_server_state(handle: Option<Res<ServerHandle>>) {
    if let Some(handle) = handle {
        let state = handle.state();
        debug!("Server state: {:?}", state);
    }
}

/// System to gracefully shutdown the server when the app exits.
pub fn shutdown_server(mut commands: Commands, handle: Option<Res<ServerHandle>>) {
    if let Some(handle) = handle {
        info!("Shutting down server...");
        handle.shutdown();
        commands.remove_resource::<ServerHandle>();
    }
}
