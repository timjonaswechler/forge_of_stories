// src/initialization/mod.rs
mod assets;
mod core;
mod debug;
mod events;
mod plugin; // <- Neu für das Haupt-Plugin
mod state; // <- Neu für AppState

// --- Öffentliche Exporte ---

// Das Haupt-Plugin für dieses Modul
pub use plugin::InitializationPlugin;

// Wichtige Typen und Ressourcen
pub use assets::{AssetLoadingTracker, EssentialAssets, GameAssets, SpeciesTemplate}; // AttributeDistribution ist intern
pub use state::AppState; // AppState hier exportieren

// Optionale Exporte (falls von außerhalb benötigt, sonst privat lassen)
// pub use core::CorePlugin;
// pub use debug::DebugPlugin;
pub use events::{AppStartupCompletedEvent, AssetsLoadedEvent};
// pub use assets::{EssentialAssetsPlugin, GameAssetsPlugin}; // Normalerweise nicht benötigt
