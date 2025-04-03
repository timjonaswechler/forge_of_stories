// src/events/genetics_events.rs
use bevy::prelude::*;

/// Wird ausgelöst, wenn eine Entität vollständig initialisiert wurde
#[derive(Event)]
pub struct EntityInitializedEvent {
    pub entity: Entity,
    pub species: Vec<String>,
}

/// Wird ausgelöst, wenn ein Attribut temporär modifiziert werden soll
#[derive(Event)]
pub struct TemporaryAttributeModifierEvent {
    pub entity: Entity,
    pub attribute_id: String,
    pub value_change: f32, // Kann positiv oder negativ sein
    pub duration: f32,     // Dauer in Sekunden
}

// Weitere Events für genetische Ereignisse können hier hinzugefügt werden
// z.B. Mutationen, Vererbung, usw.
