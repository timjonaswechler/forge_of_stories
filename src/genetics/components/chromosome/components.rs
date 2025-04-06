// src/genetics/components/chromosome/components.rs
use bevy::prelude::Component; // Füge Component hinzu, falls nötig
use serde::Deserialize; // Falls aus RON geladen

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)] // Füge ggf. Component hinzu
pub enum ChromosomeType {
    BodyStructure,
    Attributes,
    VisualTraits,
}
