// src/components/visual_traits.rs
use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct VisualTraits {
    pub skin_color: (f32, f32, f32),
    pub hair_color: (f32, f32, f32),
    pub eye_color: (f32, f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EyeColor {
    Brown,  // Braun
    Green,  // Grün
    Blue,   // Blau
    Gray,   // Grau
    Yellow, // Gelb
    Red,    // Rot
    Black,  // Schwarz
    White,  // Weiß
}
