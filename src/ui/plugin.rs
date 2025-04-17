// src/ui/plugin.rs (NEUE DATEI)
use bevy::prelude::*;

use super::{
    loading_screen::LoadingScreenPlugin, // Dein Ladebildschirm
    main_menu::MainMenuPlugin,           // Dein Hauptmenü
    // theme::ThemePlugin, // Optional: Wenn du ein separates Plugin für Theming hast
    // widgets::WidgetPlugins, // Optional: Wenn Widgets eigene Plugins haben
    systems::button_interaction_system, // Unser generisches Button-System
    theme::UiTheme,                     // Die Theme-Resource
    widgets::button::ButtonPressedEvent, // Das Event für Button-Klicks
};

/// Das Haupt-Plugin für die gesamte Benutzeroberfläche.
/// Fügt spezifische UI-Zustände (Ladebildschirm, Menü) hinzu
/// und initialisiert gemeinsame UI-Systeme und Ressourcen.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // --- Ressourcen & Events ---
            .init_resource::<UiTheme>() // Standard-Theme initialisieren
            .add_event::<ButtonPressedEvent>() // Event für Button-Klicks
            // --- Gemeinsame UI-Systeme ---
            // Fügt das System hinzu, das auf Button-Interaktionen reagiert und das Aussehen ändert
            .add_systems(Update, button_interaction_system)
            // --- Spezifische UI-Module / Plugins ---
            .add_plugins((
                LoadingScreenPlugin, // Fügt die Logik für den Ladebildschirm hinzu
                MainMenuPlugin,      // Fügt die Logik für das Hauptmenü hinzu
                                     // Hier könnten weitere UI-Plugins hinzukommen (HUD, Inventar, etc.)
            ));

        info!("UiPlugin loaded.");
    }
}
