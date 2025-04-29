// crates/forge_ui/src/theme/mod.rs
pub mod data;
pub mod runtime;
pub mod systems;

// Re-export necessary types publicly
pub use data::UiThemeData; // For loading/saving
pub use runtime::UiTheme; // The runtime resource
                          // No need to publicly re-export systems usually
                          // pub use systems::*; // Avoid wildcard re-exports if possible
