// src/dev_tools/node_graph/ui_data.rs
use super::ui_pin::PinType;
use bevy::prelude::{Color, Entity, Vec2}; // Vec2 hinzugefügt

// VisNode bleibt unverändert in dieser Iteration
#[derive(Clone, Debug)]
pub struct VisNode {
    pub id: usize,      // Eindeutige ID (oft entity.index())
    pub name: String,   // Name des Knotens
    pub position: Vec2, // Position aus der Simulation/Layout (Bevy Vec2)
    pub color: Color,   // Optional: Farbe für den Knoten
    pub entity: Option<Entity>, // Die tatsächliche Bevy Entity
                        // Optional: pub pins: Vec<VisPin>, // Pins für diesen Knoten (SPÄTER)
}

// VisPin bleibt unverändert in dieser Iteration
#[derive(Clone, Debug)]
pub struct VisPin {
    pub id: usize,         // Eindeutige Pin ID
    pub node_id: usize,    // ID des Knotens, zu dem der Pin gehört
    pub name: String,      // Name des Pins (z.B. Attributname)
    pub color: Color,      // Optional: Farbe
    pub pin_type: PinType, // Wichtig: Nutzt PinType aus ui_pin.rs
}

#[derive(Clone, Debug)]
pub struct VisLink {
    pub id: usize, // Eindeutige Link ID
    // *** GEÄNDERT: Feldnamen an LinkSpec angepasst ***
    pub start_pin_id: usize, // ID des Start-Pins
    pub end_pin_id: usize,   // ID des End-Pins
    pub color: Color,        // Optional: Farbe
}

// Eventuell nützlich: Eine Struktur, um alle UI-Daten zu bündeln
#[derive(Clone, Debug, Default)] // Default hinzugefügt
pub struct GraphUiDataBundle {
    pub nodes: Vec<VisNode>,
    pub links: Vec<VisLink>,
    // Eventuell auch Pins hier sammeln, falls nicht direkt an VisNode?
    // pub pins: Vec<VisPin>,
}
