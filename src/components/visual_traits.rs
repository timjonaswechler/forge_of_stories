// src/components/visual_traits.rs
use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct VisualTraits {
    pub skin_color: (f32, f32, f32),
    pub hair_color: (f32, f32, f32),
    pub eye_color: (f32, f32, f32),
}
impl VisualTraits {
    pub fn new() -> Self {
        Self {
            skin_color: (0.5, 0.5, 0.5),
            hair_color: (0.5, 0.5, 0.5),
            eye_color: (0.5, 0.5, 0.5),
        }
    }
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
impl EyeColor {
    // Konvertierung zwischen EyeColor und f32 für die Speicherung in GeneVariant
    pub fn to_f32(&self) -> f32 {
        match self {
            EyeColor::Brown => 0.0,
            EyeColor::Green => 1.0,
            EyeColor::Blue => 2.0,
            EyeColor::Gray => 3.0,
            EyeColor::Yellow => 4.0,
            EyeColor::Red => 5.0,
            EyeColor::Black => 6.0,
            EyeColor::White => 7.0,
        }
    }

    pub fn from_f32(value: f32) -> Self {
        match value as i32 {
            0 => EyeColor::Brown,
            1 => EyeColor::Green,
            2 => EyeColor::Blue,
            3 => EyeColor::Gray,
            4 => EyeColor::Yellow,
            5 => EyeColor::Red,
            6 => EyeColor::Black,
            7 => EyeColor::White,
            _ => EyeColor::Brown, // Fallback
        }
    }
}
