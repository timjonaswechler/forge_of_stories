// src/initialization/state/mod.rs
// Declare files within this module
pub mod plugin;
pub mod types;

// Export the plugin
pub use plugin::StatePlugin;

// Export types needed outside the state module (e.g., AppState)
pub use types::AppState;
