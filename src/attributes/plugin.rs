// src/attributes/plugin.rs
use bevy::prelude::*;

use crate::{AppState, SimulationSystemSet}; // Importiere aus lib.rs

// Importiere die Komponenten und Systeme aus dem eigenen Modul
use super::components::*; // Importiert Attribute, Physical/Mental/SocialAttributes, AttributeGroup
use super::systems::*; // Importiert calculate_effective..., update_rust, apply_attributes

pub struct AttributesPlugin;

impl Plugin for AttributesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // Systeme für Attribut-Berechnung etc.
                calculate_effective_attribute_values,
                update_attribute_rust,
                // apply_attributes wird generisch aufgerufen, für jede Gruppe
                apply_attributes::<PhysicalAttributes>,
                apply_attributes::<MentalAttributes>,
                apply_attributes::<SocialAttributes>,
                // Die update_*_attributes sind leer, können hier ausgelassen oder hinzugefügt werden
                // update_physical_attributes,
                // update_mental_attributes,
                // update_social_attributes,
            )
                // Gruppiere die Systeme in die korrekten Sets
                // Wende apply_attributes nach der Phänotyp-Berechnung an
                .in_set(SimulationSystemSet::AttributeApplication)
                // Berechne effektive Werte etc. danach
                .in_set(SimulationSystemSet::AttributeCalculation)
                // Alle laufen nur im Running State
                .run_if(in_state(AppState::Running)),
        );

        // Die Attribut-Komponenten selbst müssen nicht explizit registriert werden,
        // Bevy macht das automatisch, wenn sie verwendet werden (z.B. in Queries).
    }
}
