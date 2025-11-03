//! World-specific components.

use bevy::color::palettes::css::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Marker component for the ground plane entity.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct GroundPlane;

/// Size dimensions for the ground plane.
/// This is replicated from server to client so the client knows how to render it.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct GroundPlaneSize {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}

/// Resource for assigning player colors.
///
/// Cycles through a predefined palette to ensure each player has a unique color.
#[derive(Resource, Debug, Default)]
pub struct PlayerColorAssigner {
    available_colors: Vec<Color>,
    next_index: usize,
}

impl PlayerColorAssigner {
    pub fn new() -> Self {
        Self {
            available_colors: vec![
                RED.into(),
                BLUE.into(),
                GREEN.into(),
                YELLOW.into(),
                ORANGE.into(),
                PURPLE.into(),
                MAGENTA.into(),
                PINK.into(),
            ],
            next_index: 0,
        }
    }

    /// Assigns the next color from the palette (wraps around if needed).
    pub fn next_color(&mut self) -> Color {
        if self.available_colors.is_empty() {
            *self = Self::new();
        }
        let color = self.available_colors[self.next_index];
        self.next_index = (self.next_index + 1) % self.available_colors.len();
        color
    }
}
