// src/ui/plugin.rs (NEUE DATEI)
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use super::{
    loading_screen::LoadingScreenPlugin, // Dein Ladebildschirm
    main_menu::MainMenuPlugin,           // Dein Hauptmenü
    // theme::ThemePlugin, // Optional: Wenn du ein separates Plugin für Theming hast
    // widgets::WidgetPlugins, // Optional: Wenn Widgets eigene Plugins haben
    systems::{button_interaction_system, update_widget_button_style}, // Unser generisches Button-System
    theme::{apply_theme_on_change, load_theme, ThemeAsset, UiTheme},  // Die Theme-Resource
    widgets::button::ButtonPressedEvent,                              // Das Event für Button-Klicks
};

/// Das Haupt-Plugin für die gesamte Benutzeroberfläche.
/// Fügt spezifische UI-Zustände (Ladebildschirm, Menü) hinzu
/// und initialisiert gemeinsame UI-Systeme und Ressourcen.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<ThemeAsset>::new(&["ron"]))
            // --- Ressourcen & Events ---
            .init_resource::<UiTheme>() // Standard-Theme initialisieren
            .add_event::<ButtonPressedEvent>() // Event für Button-Klicks
            .add_systems(Startup, load_theme) // Lade das Theme, wenn die App startet
            // --- Gemeinsame UI-Systeme ---
            // Fügt das System hinzu, das auf Button-Interaktionen reagiert und das Aussehen ändert
            .add_systems(Update, button_interaction_system)
            .add_systems(Update, apply_theme_on_change)
            // neuer Live‑Reload für Padding & Font‑Size
            .add_systems(Update, update_widget_button_style)
            // --- Spezifische UI-Module / Plugins ---
            .add_plugins((
                LoadingScreenPlugin, // Fügt die Logik für den Ladebildschirm hinzu
                MainMenuPlugin,      // Fügt die Logik für das Hauptmenü hinzu
                                     // Hier könnten weitere UI-Plugins hinzukommen (HUD, Inventar, etc.)
            ));

        info!("UiPlugin loaded.");
    }
}
