use crate::components::attributes::{
    Attribute, MentalAttributes, PhysicalAttributes, SocialAttributes,
};
use bevy::prelude::*;

// System zur Berechnung der effektiven Attributwerte
pub fn calculate_effective_attribute_values(mut query: Query<&mut Attribute>) {
    for mut attribute in query.iter_mut() {
        let mut value = attribute.current_value;

        // Berücksichtige Rust/Decay
        if let Some(rust) = attribute.rust_level {
            value *= 1.0 - (rust as f32 * 0.05); // Jeder Rust-Level reduziert um 5%
        }

        // Begrenze den Wert auf den erlaubten Bereich
        attribute.effective_value = value.max(0.0).min(attribute.max_value);
    }
}

// System für Attributverfall/Rust
pub fn update_attribute_rust(time: Res<Time>, mut query: Query<&mut Attribute>) {
    // Konstanten für den Verfall
    const RUST_THRESHOLD_DAYS: f32 = 30.0; // 30 Tage ohne Nutzung = 1 Rust-Level

    for mut attribute in query.iter_mut() {
        if let Some(last_used) = attribute.last_used {
            // Berechne die Zeit seit der letzten Nutzung
            let time_since_used = time.elapsed() - last_used;
            let days_since_used = time_since_used.as_secs_f32() / (24.0 * 60.0 * 60.0);

            // Berechne neuen Rust-Level
            if days_since_used > RUST_THRESHOLD_DAYS {
                let new_rust_level = (days_since_used / RUST_THRESHOLD_DAYS).floor() as u8;
                attribute.rust_level = Some(new_rust_level.min(6)); // Maximal 6 Rust-Level
            }
        }
    }
}

// System zur Aktualisierung der physischen Attribut-Sammlung
pub fn update_physical_attributes(query: Query<&PhysicalAttributes>) {
    for _physical_attrs in query.iter() {
        // Hier könnten zusätzliche Berechnungen für die gesamte Attributgruppe erfolgen
        // z.B. Abhängigkeiten zwischen Attributen
    }
}

// System zur Aktualisierung der mentalen Attribut-Sammlung
pub fn update_mental_attributes(query: Query<&MentalAttributes>) {
    for _mental_attrs in query.iter() {
        // Hier könnten zusätzliche Berechnungen für die gesamte Attributgruppe erfolgen
    }
}

// System zur Aktualisierung der sozialen Attribut-Sammlung
pub fn update_social_attributes(query: Query<&SocialAttributes>) {
    for _social_attrs in query.iter() {
        // Hier könnten zusätzliche Berechnungen für die gesamte Attributgruppe erfolgen
    }
}
