// src/initialization/core/mod.rs
pub mod plugin;
pub mod systems;

pub use plugin::CorePlugin;
// No need to export systems unless they are used outside this module
