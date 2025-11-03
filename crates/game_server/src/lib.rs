//! Shared server-side game logic for Forge of Stories.
//!
//! This crate contains gameplay systems, world simulation, and server authority logic
//! that is shared between:
//! - Dedicated server binary (future `aether` crate)
//! - Embedded server (client-hosted server in `forge_of_stories`)
//!
//! # Architecture
//!
//! The server uses bevy_replicon for automatic server-authoritative replication:
//! - **shared/** - Components and events shared between client and server
//! - **network/** - Server networking setup and connection handling
//! - **gameplay/** - Core game logic (physics, input processing)
//! - **world/** - World management and initialization
//!
//! # Usage
//!
//! ## Embedded Server Mode
//! ```ignore
//! let server_handle = ServerHandle::start_embedded(Port(5000));
//! // Wait for server to be ready
//! while !server_handle.is_ready() {
//!     std::thread::sleep(Duration::from_millis(10));
//! }
//! ```
//!
//! ## Dedicated Server Mode
//! ```ignore
//! App::new()
//!     .add_plugins((MinimalPlugins, StatesPlugin, RepliconPlugins))
//!     .add_plugins(ServerPlugin { port: 5000 })
//!     .run();
//! ```

use app::LOG_SERVER;
use bevy::{prelude::*, state::app::StatesPlugin};
use bevy_replicon::prelude::*;

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

// Public module exports
pub mod gameplay;
pub mod network;
pub mod plugin;
pub mod settings;
pub mod shared;
pub mod world;

// Re-exports for convenience
pub use network::Port;
pub use plugin::{GameplayState, ServerPlugin};

// Re-export shared types (components and events)
pub use shared::{Player, PlayerIdentity, PlayerInput, PlayerOwner, Position, Velocity};

// Re-export world types
pub use world::{GroundPlane, GroundPlaneSize, PlayerColorAssigner};

/// Handle for managing an embedded server running in a separate thread.
///
/// The embedded server runs a complete Bevy App with RepliconPlugins
/// in its own thread, allowing the client to host a local server.
#[derive(Resource)]
pub struct ServerHandle {
    thread_handle: Option<JoinHandle<()>>,
    ready_flag: Arc<AtomicBool>,
    port: Arc<std::sync::Mutex<u16>>,
}

impl ServerHandle {
    /// Starts an embedded server in a new thread.
    ///
    /// The server will attempt to bind to the specified port, falling back
    /// to nearby ports if the requested one is in use.
    ///
    /// # Example
    /// ```ignore
    /// let server = ServerHandle::start_embedded(Port(5000));
    /// while !server.is_ready() {
    ///     std::thread::sleep(Duration::from_millis(10));
    /// }
    /// println!("Server ready on port {}", server.port());
    /// ```
    pub fn start_embedded(port: Port) -> Self {
        let ready_flag = Arc::new(AtomicBool::new(false));
        let server_ready = ready_flag.clone();

        let actual_port = Arc::new(std::sync::Mutex::new(port.0));
        let thread_port = actual_port.clone();

        let thread_handle = thread::spawn(move || {
            use network::{PortStorage, ServerReadyFlag};

            let thread_ready = ServerReadyFlag(server_ready);
            let port_storage = PortStorage(thread_port);

            App::new()
                .add_plugins((MinimalPlugins, StatesPlugin, RepliconPlugins))
                .insert_resource(thread_ready)
                .insert_resource(port_storage)
                .add_plugins(ServerPlugin { port: port.0 })
                .run();
        });

        Self {
            thread_handle: Some(thread_handle),
            ready_flag,
            port: actual_port,
        }
    }

    /// Shuts down the embedded server and waits for the thread to finish.
    pub fn shutdown(&mut self) {
        if let Some(handle) = self.thread_handle.take() {
            info!(target: LOG_SERVER, "Shutting down embedded server...");
            handle.join().expect("Failed to join server thread");
        }
    }

    /// Checks if the server has finished initializing and is ready to accept connections.
    pub fn is_ready(&self) -> bool {
        self.ready_flag.load(Ordering::Acquire)
    }

    /// Returns the actual port the server is bound to.
    ///
    /// This may differ from the requested port if it was already in use.
    pub fn port(&self) -> u16 {
        *self.port.lock().unwrap()
    }
}
