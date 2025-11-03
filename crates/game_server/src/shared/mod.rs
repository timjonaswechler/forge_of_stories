//! Shared types between server and client.
//!
//! This module contains all replicated components and network events that both
//! the server and client need to know about.

pub mod components;
pub mod events;

pub use components::*;
pub use events::*;
