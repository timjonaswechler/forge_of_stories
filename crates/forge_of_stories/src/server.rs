//! Bevy integration for the GameServer running in a separate thread.
//!
//! This module provides helper functions and systems to integrate the thread-based
//! GameServer with the Bevy app. It handles:
//! - Starting the server in Singleplayer or Multiplayer mode
//! - Managing the server lifecycle via ServerHandle
//! - Extracting the loopback client for the host player

use bevy::prelude::*;
use game_server::{ExternalTransport, ServerHandle, ServerModeInfo};
use shared::transport::{LoopbackClientTransport, TransportOrchestrator};

/// Resource containing the loopback client transport for the host player.
///
/// This is inserted into the Bevy app after starting an embedded server,
/// allowing the host to communicate with their own server.
#[derive(Resource)]
pub struct LoopbackClient(pub LoopbackClientTransport);

/// Starts an embedded server in Singleplayer mode.
///
/// This creates a GameServer with only loopback transport (no network overhead).
/// The server runs in a separate thread at 20 TPS.
///
/// # Returns
/// A tuple of (ServerHandle, LoopbackClient) to insert as Bevy resources.
pub fn start_singleplayer_server() -> (ServerHandle, LoopbackClient) {
    info!("Starting embedded server in Singleplayer mode");

    // Create loopback transport pair
    let loopback_pair = TransportOrchestrator::create_loopback_pair();

    // Start the server with only loopback transport
    let mut handle = ServerHandle::start_embedded(
        loopback_pair.client,
        loopback_pair.server,
        None, // No external transport
    );

    // Extract the loopback client for the host player
    let loopback_client = handle
        .take_loopback_client()
        .expect("Loopback client should be available");

    (handle, LoopbackClient(loopback_client))
}

/// Starts an embedded server in LAN/WAN multiplayer mode.
///
/// This creates a GameServer with loopback transport for the host player
/// and QUIC transport for remote clients.
///
/// # Arguments
/// * `bind_address` - The address to bind the QUIC server to (e.g., "0.0.0.0:7777")
///
/// # Returns
/// A tuple of (ServerHandle, LoopbackClient) to insert as Bevy resources.
pub fn start_multiplayer_server(
    bind_address: &str,
) -> Result<(ServerHandle, LoopbackClient), String> {
    info!(
        "Starting embedded server in Multiplayer mode on {}",
        bind_address
    );

    // Create loopback transport pair for the host
    let loopback_pair = TransportOrchestrator::create_loopback_pair();

    // Create QUIC transport for remote clients
    let endpoint_config = server::ServerEndpointConfiguration::from_string(bind_address)
        .map_err(|e| format!("Invalid bind address: {}", e))?;

    let channels = game_server::protocol::channels::create_gameplay_channels();

    let capabilities = shared::TransportCapabilities::new(
        true, // reliable_streams
        true, // unreliable_streams
        true, // datagrams
        8,    // max_channels
    );

    let quic = game_server::QuicTransport::new(endpoint_config, channels, capabilities);
    let external = ExternalTransport::Quic(quic);

    // Start the server with loopback + QUIC
    let mut handle =
        ServerHandle::start_embedded(loopback_pair.client, loopback_pair.server, Some(external));

    // Extract the loopback client for the host player
    let loopback_client = handle
        .take_loopback_client()
        .expect("Loopback client should be available");

    Ok((handle, LoopbackClient(loopback_client)))
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

/// Example: System to open the server to LAN.
///
/// This can be called from a UI button or hotkey to dynamically open
/// a singleplayer game to multiplayer.
pub fn open_to_lan(server_handle: Res<ServerHandle>) {
    info!("Opening server to LAN on port 7777...");

    // Create QUIC transport configuration
    let endpoint_config = match server::ServerEndpointConfiguration::from_string("0.0.0.0:7777") {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to create endpoint config: {}", e);
            return;
        }
    };

    let channels = game_server::protocol::channels::create_gameplay_channels();
    let capabilities = shared::TransportCapabilities::new(
        true, // reliable_streams
        true, // unreliable_streams
        true, // datagrams
        8,    // max_channels
    );

    let quic = game_server::QuicTransport::new(endpoint_config, channels, capabilities);

    if let Err(e) = server_handle.add_external(ExternalTransport::Quic(quic)) {
        error!("Failed to add external transport: {}", e);
    } else {
        info!("Server successfully opened to LAN!");
    }
}
