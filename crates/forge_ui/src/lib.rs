// forge_ui/src/lib.rs

pub mod button;
pub use button::{
    ButtonBuilder, ButtonClickedEvent, ButtonMarker, ButtonSize, ButtonVariant, OnClick,
};

pub mod card;
pub use card::{
    // -- Builder Pattern --
    CardBuilder,  // << Der Haupt-Builder
    ElementStyle, // << Enum für Text-Stile
    NodeElement,  // << Enum für Kind-Elemente
};

pub mod theme; // Theme-Management für UI-Elemente

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
            .add_event::<ButtonClickedEvent>()
            .add_systems(Update, (update_button_visuals, handle_button_clicks_event));
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
