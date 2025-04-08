// src/ui_components/node_graph/ui_data.rs
use super::ui_pin::PinType;
use bevy::prelude::{Color, Entity, Vec2};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// === NEUE ENUMS/STRUCTS für logische Pins ===
#[derive(Clone, Debug, PartialEq, Eq, Hash)] // Hash für ID Generierung?
pub enum PinDirection {
    Input,
    Output,
    InOut, // Für Pins, die beides können (z.B. Freundschaft)
}

#[derive(Clone, Debug)]
pub struct LogicalPinInfo {
    /// Eindeutiger Name/Kennzeichner *innerhalb* des Nodes für diesen Pin-Typ.
    /// Z.B. "Parent", "Child", "BestFriend", "Target".
    /// Wird für die Generierung der visuellen Pin-ID verwendet.
    pub identifier: String,

    /// Angezeigter Name im UI (kann gleich identifier sein).
    /// Z.B. "Parent Output", "Child Input", "Friend".
    pub display_name: String,

    /// Welche Art von Verbindung repräsentiert dieser Pin?
    /// Wird zur Validierung verwendet (z.B. nur "Family" mit "Family" verbinden).
    pub relation_type: String,

    /// In welche Richtung(en) darf dieser Pin verbunden werden?
    pub direction: PinDirection,
    // Optional: Weitere Metadaten
    // pub color: Option<Color>, // Spezifische Farbe für diesen Pin-Typ?
    // pub shape: Option<PinShape>, // Spezifische Form für diesen Pin-Typ?
    // pub data_type: String, // z.B. "EntityRef", "Number", "String"
}
// === ENDE NEUE ENUMS/STRUCTS ===

#[derive(Clone, Debug)]
pub struct VisNode {
    pub id: usize, // Node-ID (z.B. entity.index())
    pub name: String,
    pub position: Vec2,
    pub color: Color,
    pub entity: Option<Entity>,
    /// Definiert, welche logischen Anschlusspunkte dieser Node hat.
    /// Wird vom DataProvider gefüllt.
    pub logical_pins: Vec<LogicalPinInfo>, // <-- NEUES FELD
                                           // Optional: pub details: HashMap<String, String>, // Für Detailansicht
}

// VisPin (Definition für tatsächliche UI-Pin-Daten, aktuell nicht direkt genutzt, Infos kommen aus PinSpec)
#[derive(Clone, Debug)]
pub struct VisPin {
    pub id: usize,
    pub node_id: usize,
    pub name: String,
    pub color: Color,
    pub pin_type: PinType, // Echter UI-Pin-Typ (In/Out)
}

// VisLink bleibt gleich
#[derive(Clone, Debug)]
pub struct VisLink {
    pub id: usize,
    pub start_pin_id: usize,
    pub end_pin_id: usize,
    pub color: Color,
}

// Eventuell nützlich: Eine Struktur, um alle UI-Daten zu bündeln
#[derive(Clone, Debug, Default)] // Default hinzugefügt
pub struct GraphUiDataBundle {
    pub nodes: Vec<VisNode>,
    pub links: Vec<VisLink>,
    // Eventuell auch Pins hier sammeln, falls nicht direkt an VisNode?
    // pub pins: Vec<VisPin>,
}

// === NEUE Hilfsfunktion zur Pin-ID-Generierung ===
/// Generiert eine (relativ) eindeutige ID für einen Pin.
/// **Achtung:** Verwendet node_id (oft entity index) und ist daher NICHT STABIL über Programmstarts!
/// Für persistente Graphen wird eine stabilere node_id benötigt.
pub fn generate_pin_id(node_id: usize, pin_identifier: &str) -> usize {
    // Verwende den Standard-Hasher von Rust
    let mut hasher = DefaultHasher::new();
    // Hashe die Node-ID und den Pin-Identifier zusammen
    node_id.hash(&mut hasher);
    pin_identifier.hash(&mut hasher);
    // Gib den resultierenden Hash als usize zurück
    hasher.finish() as usize // Konvertierung von u64 zu usize (auf 64bit meist kein Problem)
}
// === ENDE NEUE Hilfsfunktion ===
