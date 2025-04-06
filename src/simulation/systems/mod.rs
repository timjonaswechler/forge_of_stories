// src/simulation/systems/mod.rs
pub mod character_spawner;
pub mod event_handlers;
pub mod reproduction;

// Optional: Re-exportiere Systeme für einfacheren Zugriff vom simulation::plugin
pub use character_spawner::*;
pub use event_handlers::*;
pub use reproduction::*;
