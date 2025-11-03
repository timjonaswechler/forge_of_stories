//! Centralized input handling.
//!
//! This module consolidates all input handling:
//! - **player.rs** - Player movement input (WASD, Space) → sent to server
//! - **menu.rs** - Menu toggling (ESC) → local UI state changes
//! - **camera.rs** - Camera controls (handled by camera systems)
//!
//! ## Architecture
//!
//! - Player input is sent as events to the server via bevy_replicon
//! - Menu/UI input is handled locally and updates UI state
//! - Camera input is kept with camera systems (specialized behavior)

pub mod menu;
pub mod player;

use bevy::prelude::*;

/// Main input plugin that coordinates all input systems.
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((player::PlayerInputPlugin, menu::MenuInputPlugin));
    }
}
