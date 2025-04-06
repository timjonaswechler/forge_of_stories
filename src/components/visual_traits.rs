use bevy::prelude::*;
use serde::Deserialize; // Behalte Deserialize für EyeColor

#[derive(Component, Debug, Clone)]
pub struct VisualTraits {
    pub skin_color: Color, // <- Änderung: Color
    pub hair_color: Color, // <- Änderung: Color
    pub eye_color: Color,  // <- Änderung: Color (speichert die resultierende RGB Farbe)
}
impl VisualTraits {
    pub fn new() -> Self {
        Self {
            skin_color: Color::srgb_u8(252, 15, 192),
            hair_color: Color::srgb_u8(252, 15, 192),
            eye_color: Color::srgb_u8(252, 15, 192),
        }
    }
}

impl Default for VisualTraits {
    fn default() -> Self {
        Self::new()
    }
}

// EyeColor Enum bleibt unverändert (für Vererbung und Gen-Speicherung)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum EyeColor {
    Brown,
    Green,
    Blue,
    Gray,
    Yellow,
    Red,
    Black,
    White,
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
        // Runden zur Sicherheit, falls der Wert leicht abweicht (z.B. durch Mutation)
        match value.round() as i32 {
            0 => EyeColor::Brown,
            1 => EyeColor::Green,
            2 => EyeColor::Blue,
            3 => EyeColor::Gray,
            4 => EyeColor::Yellow,
            5 => EyeColor::Red,
            6 => EyeColor::Black,
            7 => EyeColor::White,
            _ => {
                warn!(
                    "Ungültiger f32-Wert für EyeColor::from_f32: {}. Fallback auf Brown.",
                    value
                );
                EyeColor::Brown // Fallback
            }
        }
    }
}
