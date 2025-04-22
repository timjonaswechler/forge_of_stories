// src/lib.rs
pub mod attributes;
pub mod initialization; // <- Geändert
pub mod simulation;
pub mod ui; // <- Geändert

// Konstanten
pub const FIXED_SEED: u64 = 1234567890;
pub const USE_FIXED_SEED: bool = true;

// System Sets
use bevy::prelude::*;
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum SimulationSystemSet {
    GenotypePhenotype,
    AttributeApplication,
    VisualTraitApplication,
    AttributeCalculation,
}

// Re-exportiere den AppState aus dem initialization Modul
pub use initialization::AppState;
