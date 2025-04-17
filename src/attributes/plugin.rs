// src/attributes/plugin.rs
use super::{
    components::*,              // Behalte Komponenten-Imports
    events::AttributeUsedEvent, // <--- Importiere das Event
    systems::*,                 // Behalte System-Imports
};
use crate::{AppState, SimulationSystemSet};
use bevy::prelude::*;

pub struct AttributesPlugin;

impl Plugin for AttributesPlugin {
    fn build(&self, app: &mut App) {
        info!("AttributesPlugin initialized.");
        // --- REGISTRIERE DAS EVENT ---
        app.add_event::<AttributeUsedEvent>();
        // -------------------------

        // Systeme für AttributeApplication (unverändert)
        app.add_systems(
            Update,
            (
                apply_attributes::<PhysicalAttributes>,
                apply_attributes::<MentalAttributes>,
                apply_attributes::<SocialAttributes>,
            )
                .in_set(SimulationSystemSet::AttributeApplication)
                .run_if(in_state(AppState::Running)),
        );

        // Systeme für AttributeCalculation (unverändert)
        app.add_systems(
            Update,
            (
                calculate_effective_attribute_values,
                update_attribute_rust,
                update_physical_attributes,
                update_mental_attributes,
                update_social_attributes,
                update_attribute_usage, // Dieses System kann jetzt auf das Event zugreifen
            )
                .in_set(SimulationSystemSet::AttributeCalculation)
                .run_if(in_state(AppState::Running)),
        );
    }
}
