// src/lib.rs

// Top-Level Module für Features
pub mod app_setup;
pub mod attributes;
pub mod config;
pub mod debug;
pub mod dev_ui;
pub mod genetics;
pub mod simulation;
pub mod ui_components;
pub mod visuals;

// Konstanten (wenn nicht in config verschoben)
pub const FIXED_SEED: u64 = 1234567890;
pub const USE_FIXED_SEED: bool = true;

// System Sets (falls global benötigt)
use bevy::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum SimulationSystemSet {
    GenotypePhenotype,      // genetics::systems::genotype_to_phenotype_system
    AttributeApplication,   // attributes::systems::apply_attributes
    VisualTraitApplication, // visuals::systems::apply_visual_traits_system
    AttributeCalculation,   // attributes::systems::calculate_effective_attribute_values etc.
}

// AppState (wird von vielen Plugins benötigt)
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    Running,
}
