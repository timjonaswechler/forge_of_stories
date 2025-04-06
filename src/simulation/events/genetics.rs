// src/events/genetics_events.rs
use crate::genetics::components::gene_types::AttributeGene; // Verwende den Enum
use bevy::prelude::*;

/// Wird ausgelöst, wenn eine Entität vollständig initialisiert wurde
#[derive(Event)]
pub struct EntityInitializedEvent {
    pub entity: Entity,
    pub species: Vec<String>,
}

/// Wird ausgelöst, wenn ein Attribut temporär modifiziert werden soll
#[derive(Event, Debug)] // Füge Debug hinzu
pub struct TemporaryAttributeModifierEvent {
    pub entity: Entity,

    pub attribute_id: AttributeGene, // <- Enum als ID
    pub value_change: f32,
    pub duration: f32,
}

/// Event, um Reproduktion anzufordern (Beispiel)
#[derive(Event, Debug)]
pub struct ReproduceRequestEvent {
    pub parent1: Entity,
    pub parent2: Entity,
}

/// Event, das nach erfolgreicher Reproduktion gesendet wird (Beispiel)
#[derive(Event, Debug)]
pub struct ChildBornEvent {
    pub child: Entity,
    pub parent1: Entity,
    pub parent2: Entity,
}
