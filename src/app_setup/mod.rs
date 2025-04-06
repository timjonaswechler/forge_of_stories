// src/app_setup/mod.rs
pub mod core_plugin;
pub mod event_plugin;
pub mod setup_plugin;

// Re-exportiere den AppState für einfacheren Zugriff aus diesem Modul heraus
// Nützlich, wenn andere Teile von app_setup den State brauchen
pub use setup_plugin::AppState;
