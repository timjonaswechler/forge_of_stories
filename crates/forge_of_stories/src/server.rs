//! Bevy integration for the GameServer running in a separate thread.
//!
//! This module provides helper functions and systems to integrate the thread-based
//! GameServer with the Bevy app. It handles:
//! - Starting the server in Singleplayer or Multiplayer mode
//! - Managing the server lifecycle via ServerHandle
//! - Extracting the loopback client for the host player

use bevy::prelude::*;
use game_server::{ExternalTransport, ServerHandle};
use shared::transport::{LoopbackClientTransport, TransportOrchestrator};
use std::io::ErrorKind;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

/// Resource containing the loopback client transport for the host player.
///
/// This is inserted into the Bevy app after starting an embedded server,
/// allowing the host to communicate with their own server.
#[derive(Resource)]
pub struct LoopbackClient(pub LoopbackClientTransport);

const PORT_PROBE_LIMIT: u16 = 16;

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
    let base_addr: SocketAddr = bind_address
        .parse()
        .map_err(|e| format!("Invalid bind address: {e}"))?;
    let resolved_addr = resolve_bind_addr(base_addr)?;

    info!(
        "Starting embedded server in Multiplayer mode on {}",
        resolved_addr
    );

    // Create loopback transport pair for the host
    let loopback_pair = TransportOrchestrator::create_loopback_pair();

    // Create QUIC transport for remote clients
    let endpoint_config = server::ServerEndpointConfiguration::from_addr(resolved_addr);

    let channels = game_server::protocol::channels::create_gameplay_channels();

    let capabilities = shared::TransportCapabilities::new(
        true, // reliable_streams
        true, // unreliable_streams
        true, // datagrams
        8,    // max_channels
    );

    let quic = match game_server::QuicTransport::new(endpoint_config, channels, capabilities) {
        Ok(quic) => quic,
        Err(e) => {
            error!("Failed to create QUIC transport: {}", e);
            return Err(format!("Failed to create QUIC transport: {e}"));
        }
    };
    let external = ExternalTransport::Quic(quic);

    info!("QUIC transport bound to {}", resolved_addr);

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
    info!("Opening server to LAN...");

    match create_default_quic_transport() {
        Ok((external, addr)) => {
            if let Err(e) = server_handle.add_external(external) {
                error!("Failed to add external transport: {}", e);
            } else {
                info!("Server successfully opened to LAN on {}", addr);
            }
        }
        Err(e) => error!("Failed to create external transport: {}", e),
    }
}

fn resolve_bind_addr(base_addr: SocketAddr) -> Result<SocketAddr, String> {
    if base_addr.port() == 0 {
        return Ok(base_addr);
    }

    for offset in 0..=PORT_PROBE_LIMIT {
        let candidate_port = base_addr.port() as u32 + offset as u32;
        if candidate_port > u16::MAX as u32 {
            break;
        }

        let candidate = SocketAddr::new(base_addr.ip(), candidate_port as u16);

        match UdpSocket::bind(candidate) {
            Ok(socket) => {
                drop(socket);

                if offset > 0 {
                    info!(
                        "Requested port {} unavailable, using {} instead",
                        base_addr.port(),
                        candidate.port()
                    );
                }

                return Ok(candidate);
            }
            Err(err) if err.kind() == ErrorKind::AddrInUse => continue,
            Err(err) => {
                return Err(format!(
                    "Failed to bind UDP port {} on {}: {err}",
                    candidate.port(),
                    candidate.ip()
                ));
            }
        }
    }

    Err(format!(
        "No free UDP port available near {}:{}",
        base_addr.ip(),
        base_addr.port()
    ))
}

pub fn create_default_quic_transport() -> Result<(ExternalTransport, SocketAddr), String> {
    let base_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 7777));
    let resolved_addr = resolve_bind_addr(base_addr)?;
    info!("Preparing default QUIC transport on {}", resolved_addr);

    let endpoint_config = server::ServerEndpointConfiguration::from_addr(resolved_addr);

    let channels = game_server::protocol::channels::create_gameplay_channels();
    let capabilities = shared::TransportCapabilities::new(
        true, // reliable_streams
        true, // unreliable_streams
        true, // datagrams
        8,    // max_channels
    );

    let quic = game_server::QuicTransport::new(endpoint_config, channels, capabilities)
        .map_err(|e| format!("Failed to create QUIC transport: {e}"))?;

    Ok((ExternalTransport::Quic(quic), resolved_addr))
}
