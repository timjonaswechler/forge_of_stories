// src/ui/mod.rs (AKTUALISIERT)
pub mod loading_screen;
pub mod main_menu;
pub mod plugin; // <- Neu
pub mod systems; // <- Neu (oder integriert in plugin.rs)
pub mod theme; // <- Neu
pub mod widgets; // <- Neu

pub use loading_screen::LoadingScreenPlugin;
pub use main_menu::MainMenuPlugin;
pub use plugin::UiPlugin; // <- Exportiere das Haupt-Plugin
pub use theme::apply_theme_on_change;
pub use theme::load_theme; // <- Korrigiert den Import
