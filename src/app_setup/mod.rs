// src/app_setup/mod.rs
pub mod assets;
pub mod core;
pub mod events;

// Re-exportiere wichtige Komponenten für einfacheren Zugriff
pub use crate::AppState;
pub use assets::{AssetLoadingTracker, GameAssets, SetupPlugin, SpeciesTemplate};
pub use core::CorePlugin;
pub use events::EventPlugin;
