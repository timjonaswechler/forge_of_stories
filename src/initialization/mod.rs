// src/initialization/mod.rs

// Declare sub-modules
mod assets;
mod core;
mod debug;
mod events;
mod plugin;
mod state; // Main initialization plugin module

// --- Public Exports ---

// The main plugin for this module
pub use plugin::InitializationPlugin;

// Important types and resources needed outside the module
pub use assets::types::{AssetLoadingTracker, EssentialAssets, GameAssets, SpeciesTemplate};
pub use events::types::{AppStartupCompletedEvent, AssetsLoadedEvent};
pub use state::types::AppState;

// Re-export internal plugins if needed elsewhere (less common)
// pub use assets::plugin::AssetManagementPlugins;
// pub use core::plugin::CorePlugin;
// pub use debug::plugin::DebugPlugin;
// pub use events::plugin::EventPlugin;
// pub use state::plugin::StatePlugin;

// Note: AttributeDistribution and FontsFolderHandle are usually internal
// to the assets module and don't need to be exported publicly here.
