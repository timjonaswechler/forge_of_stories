// src/ui_components/node_graph/context.rs
use bevy::math::Vec2;

pub use super::{
    settings::NodesSettings, state::GraphUiStateManager, storage::GraphStorage, ui_pin::PinSpec,
};

pub type LinkValidationCallback = dyn Fn(
    &PinSpec,      // Start Pin Spec
    &PinSpec,      // End Pin Spec (Hovered)
    &GraphStorage, // Zugriff auf alle Nodes/Pins/Links
    &NodesSettings, // Zugriff auf Style/IO-Settings
                   // Optional: &GraphUiStateManager, // Nur wenn UI-Zustand (Selektion etc.) gebraucht wird
) -> bool;

// Event-Enum für Graph-Änderungen
#[derive(Debug, Clone)]
pub enum GraphChange {
    LinkCreated(usize, usize), // Start Pin ID, End Pin ID
    LinkRemoved {
        start_pin_id: usize,
        end_pin_id: usize,
    },
    LinkModified {
        new_start_pin_id: usize, // Neuer Output Pin
        new_end_pin_id: usize,   // Neuer Input Pin

        old_start_pin_id: usize,
        old_end_pin_id: usize,
    },

    NewLinkRequested(usize, usize),
    NodeMoved(usize, Vec2),
    NodeRemoved(usize),
}
