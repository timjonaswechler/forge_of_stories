//! Core gameplay systems.
//!
//! This module contains server-authoritative game logic including:
//! - Player input processing
//! - Physics simulation
//! - Movement systems

pub mod physics;
pub mod player_input;

pub use physics::*;
pub use player_input::*;
