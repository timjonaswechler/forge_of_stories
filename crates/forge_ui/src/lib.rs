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

pub mod label;
pub use label::{
    LabelBuilder, // << Builder exportieren
    LabelMarker,  // << Marker exportieren
};

pub mod checkbox;
pub use checkbox::{
    CheckboxBuilder,      // Der Builder
    CheckboxChangedEvent, // Das Event bei Änderung
    CheckboxMarker,       // Marker-Komponente
    CheckboxState,        // Die Zustandskomponente
};

pub mod tabs;
pub use tabs::{
    TabChangedEvent, // Event bei Tab-Wechsel
    TabId,           // Re-export TabId for use in this file
    // Marker-Komponenten weniger wichtig, wenn Builder verwendet wird
    TabsBuilder, // Der Haupt-Builder
};

pub mod badge;
pub use badge::{
    BadgeBuilder, // Der Builder
    BadgeMarker,  // Der Marker
    BadgeVariant, // Das Varianten-Enum
};

pub mod dialog;
pub use dialog::{
    CloseDialogEvent,   // Event zum Schließen
    DialogBuilder,      // Der Builder
    DialogCloseTrigger, // Marker für Schließen-Buttons
    // Marker wie DialogRoot, DialogContent weniger oft direkt gebraucht
    DialogId,        // ID-Komponente
    OpenDialogEvent, // Event zum Öffnen
};

pub mod layout;
pub use layout::{
    HorizontalStackBuilder, // Builder für horizontale Stacks
    UiRoot,                 // Marker für die Wurzel (falls benötigt)
    VerticalStackBuilder,   // Builder für vertikale Stacks
};

pub mod theme; // Theme-Management für UI-Elemente

use bevy::prelude::*;
use button::{handle_button_clicks_event, update_button_visuals};
use checkbox::{
    handle_checkbox_clicks, update_checkbox_visuals, update_checkmark_visibility_on_state_change,
};
use dialog::{
    close_dialog_system, handle_overlay_click_system, open_dialog_system,
    register_initially_open_dialogs, ActiveDialogs,
};
use tabs::{handle_tab_triggers, populate_initial_tab_content, update_tabs_visuals};
// Später: use tabs::{handle_tab_activation};

/// Plugin für die Kernfunktionalität der Forge UI Widgets.
/// Fügt Interaktionssysteme und Events hinzu.
pub struct ForgeUiPlugin;

impl Plugin for ForgeUiPlugin {
    fn build(&self, app: &mut App) {
        // Nur Events registrieren
        app.add_event::<button::ButtonClickedEvent>()
            .add_event::<checkbox::CheckboxChangedEvent>()
            // --- NEU: Tabs spezifisch (Generisch über Typ T) ---
            // Man muss das Event und die Systeme für JEDEN verwendeten Wert-Typ T registrieren.
            // Hier ein Beispiel für TabId als Wert-Typ:
            .add_event::<TabChangedEvent<tabs::TabId>>() // <-- Beispiel mit TabId
            .init_resource::<ActiveDialogs>() // WICHTIG: Ressource initialisieren
            .add_event::<OpenDialogEvent>()
            .add_event::<CloseDialogEvent>()
            .add_systems(
                PostUpdate,
                (
                    // PostUpdate, um Inhalt nach dem Bauen zu füllen
                    populate_initial_tab_content::<tabs::TabId>, // <-- Beispiel mit TabId
                ),
            )
            .add_systems(
                Update,
                (
                    open_dialog_system,                 // Öffnet Dialoge per Event
                    close_dialog_system, // Schließt Dialoge (ESC, Event, Close Button)
                    handle_overlay_click_system, // Schließt bei Klick aufs Overlay
                    handle_tab_triggers::<tabs::TabId>, // <-- Beispiel mit TabId
                    update_tabs_visuals::<tabs::TabId>, // <-- Beispiel mit TabId
                ),
            );
        // ----------------------------------------------------
        // TODO: Wenn Sie andere Typen als Tab-Werte verwenden (z.B. Enums),
        //       müssen Sie das Event und die Systeme dafür separat hinzufügen!
        //       Alternative: Komplexeres Plugin mit Typ-Registrierung.
        info!("ForgeUiPlugin loaded. (Systems must be added by app)");
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
