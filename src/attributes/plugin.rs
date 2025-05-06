// src/attributes/plugin.rs
use super::{
    // Behalte Komponenten-Imports
    events::AttributeUsedEvent,       // <--- Importiere das Event
    systems::apply_mental_attributes, // Importiere apply_attributes explizit
    systems::*,                       // Behalte System-Imports
};

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
                apply_physical_attributes,
                apply_mental_attributes,
                apply_social_attributes,
            ),
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
            ),
        );
    }
}
