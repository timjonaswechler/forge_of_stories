// src/initialization/events/mod.rs
pub mod plugin;
pub mod systems;
pub mod types;

pub use plugin::EventPlugin;
pub use types::{AppStartupCompletedEvent, AssetsLoadedEvent}; // Export event types if needed elsewhere
