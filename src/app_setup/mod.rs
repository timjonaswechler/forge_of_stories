// src/app_setup/mod.rs
pub mod assets;
pub mod core;
pub mod events;

// Re-exportiere den AppState für einfacheren Zugriff aus diesem Modul heraus
// Nützlich, wenn andere Teile von app_setup den State brauchen
pub use assets::AppState;
pub use assets::SetupPlugin;
pub use core::CorePlugin;
pub use events::EventPlugin;
