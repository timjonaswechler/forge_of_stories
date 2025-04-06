// src/attributes/plugin.rs
use super::components::*;
use super::systems::*;
use crate::{AppState, SimulationSystemSet};
use bevy::prelude::*;

pub struct AttributesPlugin;

impl Plugin for AttributesPlugin {
    fn build(&self, app: &mut App) {
        // Systeme für AttributeApplication
        app.add_systems(
            Update,
            (
                apply_attributes::<PhysicalAttributes>,
                apply_attributes::<MentalAttributes>,
                apply_attributes::<SocialAttributes>,
            )
                .in_set(SimulationSystemSet::AttributeApplication) // Nur dieses Set
                .run_if(in_state(AppState::Running)),
        );

        // Systeme für AttributeCalculation
        app.add_systems(
            Update,
            (
                calculate_effective_attribute_values,
                update_attribute_rust,
                update_physical_attributes, // Kann hier bleiben (wenn Body hinzugefügt wird) oder weggelassen werden
                update_mental_attributes,
                update_social_attributes,
                update_attribute_usage, // Gehört auch eher hierher
            )
                .in_set(SimulationSystemSet::AttributeCalculation) // Nur dieses Set
                .run_if(in_state(AppState::Running)),
        );
    }
}
