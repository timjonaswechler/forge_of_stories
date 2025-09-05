/*!
Central hub for optional server extensions.

Currently provided:
- `uds::axum`: Control-plane over Unix Domain Socket (UDS) backed by axum.

This module wires the submodules and re-exports the primary control-plane
helpers for convenient use by the server runtime.
*/

pub mod uds {
    // Explicitly map the `axum` module to its file without requiring an intermediate mod.rs.
    #[path = "axum.rs"]
    pub mod axum;

    // Re-export within the uds namespace for convenience.
    pub use axum::{UdsAxumHandle, start_uds_axum};
}

// Re-export at the `extensions` level for ergonomic access.
pub use uds::axum::{UdsAxumHandle, start_uds_axum};
