// src/attributes/events.rs
use super::components::AttributeType;
use bevy::prelude::*; // Stelle sicher, dass AttributeType importiert wird

#[derive(Event)]
pub struct AttributeUsedEvent {
    pub entity: Entity,
    pub attribute_type: AttributeType,
}
