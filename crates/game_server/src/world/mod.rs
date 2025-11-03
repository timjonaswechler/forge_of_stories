//! World management and setup.
//!
//! This module handles world state, including:
//! - World entity components (GroundPlane, etc.)
//! - World initialization and spawning
//! - Player color assignment

pub mod components;
pub mod setup;

pub use components::*;
pub use setup::*;
