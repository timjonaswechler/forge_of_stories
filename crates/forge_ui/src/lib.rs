// forge_ui/src/lib.rs

pub mod button; // Erweitere hier um card, tabs, etc.
                // Exportiere *nur*, was von außen gebraucht wird
pub use button::{
    ButtonBuilder,      // Zum Erstellen von Buttons
    ButtonClickedEvent, // Das Event, auf das die App reagiert
    ButtonMarker,       // Nützlich für Queries in der App (z.B. für spezifische Buttons)
    ButtonSize,         // Enum für die Größenkonfiguration
    ButtonVariant,      // Enum für die Variantenkonfiguration
    OnClick,            // Optional: Falls die App direkt Callbacks nutzen möchte (weniger flexibel)
};

use bevy::prelude::*;
use button::{handle_button_clicks_event, update_button_visuals};
// Später: use card::{update_card_visuals};
// Später: use tabs::{handle_tab_activation};

/// Plugin für die Kernfunktionalität der Forge UI Widgets.
/// Fügt Interaktionssysteme und Events hinzu.
pub struct ForgeUiPlugin;

impl Plugin for ForgeUiPlugin {
    fn build(&self, app: &mut App) {
        app
            // --- Button spezifisch ---
            // 1. Event registrieren: Macht das ButtonClickedEvent im ECS bekannt.
            .add_event::<ButtonClickedEvent>()
            // 2. Systeme hinzufügen: Fügen die Logik zum Update-Schedule hinzu.
            .add_systems(
                Update,
                (
                    // Dieses System sorgt für das visuelle Feedback (Farben ändern sich).
                    update_button_visuals,
                    // Dieses System prüft auf Klicks und sendet das ButtonClickedEvent.
                    handle_button_clicks_event,
                    // Optional: Wenn du direkte fn()-Callbacks unterstützen willst.
                    // Normalerweise ist der Event-Ansatz vorzuziehen.
                    // handle_button_clicks_fn,
                ),
            );
        // Log-Meldung, um sicherzustellen, dass das Plugin geladen wird
        info!("ForgeUiPlugin loaded.");
    }
}

// Zukünftige Erweiterungen (Card, Tabs etc.) würden hier ebenfalls ihre Events
// registrieren und ihre Update-Systeme hinzufügen.
// z.B.:
// pub mod card;
// pub use card::{CardBuilder, CardMarker, CardClickedEvent};
// use card::{update_card_visuals, handle_card_clicks};
// ... und im build():
// .add_event::<CardClickedEvent>()
// .add_systems(Update, (update_card_visuals, handle_card_clicks))
