//! World and entity components for the game.
//!
//! This module defines the minimal game entities for the demo:
//! - Ground plane
//! - Player entities (colored shapes)

use bevy::color::palettes::css::*;
use bevy::prelude::*;

/// Marker component for the ground plane entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct GroundPlane;

/// Player entity component.
#[derive(Component, Debug, Clone)]
pub struct Player {
    /// Unique player ID (matches ClientId from networking).
    pub id: u64,
    /// Player's assigned color.
    pub color: Color,
}

/// Player shape type (for rendering).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerShape {
    Cube,
    Sphere,
    Capsule,
}

impl Default for PlayerShape {
    fn default() -> Self {
        Self::Capsule
    }
}

/// Movement velocity component.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Velocity {
    pub linear: Vec3,
}

/// Simple position component (server authoritative).
///
/// The server maintains this, clients receive updates.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Position {
    pub translation: Vec3,
}

/// Resource for assigning player colors.
///
/// Cycles through a predefined palette to ensure each player has a unique color.
#[derive(Resource, Debug)]
pub struct PlayerColorAssigner {
    available_colors: Vec<Color>,
    next_index: usize,
}

impl Default for PlayerColorAssigner {
    fn default() -> Self {
        Self {
            available_colors: vec![
                RED.into(),     // Red
                BLUE.into(),    // Blue
                GREEN.into(),   // Green
                YELLOW.into(),  // Yellow
                ORANGE.into(),  // Orange
                PURPLE.into(),  // Purple
                MAGENTA.into(), // Magenta
                PINK.into(),    // Pink
            ],
            next_index: 0,
        }
    }
}

impl PlayerColorAssigner {
    /// Assigns the next color from the palette (wraps around if needed).
    pub fn next_color(&mut self) -> Color {
        let color = self.available_colors[self.next_index];
        self.next_index = (self.next_index + 1) % self.available_colors.len();
        color
    }
}
