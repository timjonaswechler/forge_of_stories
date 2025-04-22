// src/initialization/assets/mod.rs
pub mod plugin;
pub mod systems;
pub mod types;

// Export the main asset management plugin
pub use plugin::AssetManagementPlugins;

// Export types needed outside the assets module (e.g., for accessing loaded assets)
pub use types::{AssetLoadingTracker, EssentialAssets, GameAssets, SpeciesTemplate};

// Optional: Export specific internal plugins if needed (less common)
// pub use plugin::{EssentialAssetsPlugin, GameAssetsPlugin};
