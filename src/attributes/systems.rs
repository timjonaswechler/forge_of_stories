// src/attributes/systems.rs

// Importiere die notwendigen Komponenten und den Trait aus dem components-Modul
use crate::attributes::{
    components::{
        Attribute, AttributeGroup, MentalAttributes, PhysicalAttributes, SocialAttributes,
    },
    events::AttributeUsedEvent, // <--- Importiere das Event von seinem neuen Ort
};
use bevy::prelude::*;
use bevy::time::Time; // Importiere Time für update_attribute_rust und update_attribute_usage

// System zur Berechnung der effektiven Attributwerte (unverändert)
pub fn calculate_effective_attribute_values(mut query: Query<&mut Attribute>) {
    for mut attribute in query.iter_mut() {
        let mut value = attribute.current_value;

        if let Some(rust) = attribute.rust_level {
            value *= 1.0 - (rust as f32 * 0.05);
        }

        attribute.effective_value = value.clamp(0.0, attribute.max_value);
    }
}

// System für Attributverfall/Rust (unverändert)
pub fn update_attribute_rust(time: Res<Time>, mut query: Query<&mut Attribute>) {
    const RUST_THRESHOLD_DAYS: f32 = 30.0; // Beispielwert

    // Beispielhafte Konvertierung von Sekunden in In-Game-Tage (Annahme: 1 Sekunde = 1 Stunde)
    let seconds_per_ingame_day = 24.0; // * 60.0 * 60.0; // Wenn 1 Sekunde = 1 Sekunde wäre

    for mut attribute in query.iter_mut() {
        if let Some(last_used) = attribute.last_used {
            // Verwende time.elapsed() um die Gesamtzeit seit Spielstart zu bekommen
            let time_since_used = time.elapsed() - last_used;
            let days_since_used = time_since_used.as_secs_f32() / seconds_per_ingame_day;

            if days_since_used > RUST_THRESHOLD_DAYS {
                // Berechne Rust-Level basierend darauf, wie viele RUST_THRESHOLD_DAYS-Perioden vergangen sind
                let rust_periods = (days_since_used / RUST_THRESHOLD_DAYS).floor();
                // Begrenze das Rust-Level (z.B. auf maximal 6)
                let new_rust_level = (rust_periods as u8).min(6);
                attribute.rust_level = Some(new_rust_level);
                // Optional: Logge Änderung
                // info!("Attribute {:?} rust level set to {:?} ({} days since last use)", attribute.id, attribute.rust_level, days_since_used);
            } else {
                // Wenn das Attribut wieder verwendet wurde (last_used aktualisiert) ODER die Zeit noch nicht reicht,
                // sollte der Rust-Level potenziell entfernt werden (oder zumindest nicht erhöht).
                // Das Entfernen von Rust passiert typischerweise, wenn `last_used` aktualisiert wird.
                // Hier könnten wir sicherstellen, dass es None ist, wenn die Bedingung nicht erfüllt ist.
                // Aber Vorsicht: Das würde Rust entfernen, sobald die Zeit *unter* den Threshold fällt,
                // was seltsam wäre. Besser: Rust nur entfernen, wenn das Attribut aktiv genutzt wird.
                // Daher lassen wir den Rust-Level hier bestehen, wenn er einmal gesetzt wurde und die Zeit noch läuft.
                // Das `update_attribute_usage` System sollte `rust_level` auf `None` setzen bei Benutzung.
            }
        } else {
            // Wenn das Attribut noch nie benutzt wurde, startet es ohne Rust.
            attribute.rust_level = None;
        }
    }
}

// Platzhalter-System, korrigiert für Warnungen
// T muss Component sein, damit Query funktioniert.
// T muss AttributeGroup sein, um get_attribute_mut aufrufen zu können.
// Für Bevy-Queries müssen konkrete Typen verwendet werden, daher drei separate Systeme:
pub fn apply_physical_attributes(mut query: Query<&mut PhysicalAttributes>) {
    for mut _attributes_group in query.iter_mut() {
        // Hier Logik für PhysicalAttributes einfügen
    }
}

pub fn apply_mental_attributes(mut query: Query<&mut MentalAttributes>) {
    for mut _attributes_group in query.iter_mut() {
        // Hier Logik für MentalAttributes einfügen
    }
}

pub fn apply_social_attributes(mut query: Query<&mut SocialAttributes>) {
    for mut _attributes_group in query.iter_mut() {
        // Hier Logik für SocialAttributes einfügen
    }
}

// Update-Systeme (nur mit unterstrichenen Parametern, wenn sie nicht verwendet werden)
pub fn update_physical_attributes(_query: Query<&PhysicalAttributes>) {
    // TODO: Implementiere Logik, z.B. basierend auf Gesundheit, Müdigkeit etc.
}

pub fn update_mental_attributes(_query: Query<&MentalAttributes>) {
    // TODO: Implementiere Logik, z.B. basierend auf Stress, Lernen etc.
}

pub fn update_social_attributes(_query: Query<&SocialAttributes>) {
    // TODO: Implementiere Logik, z.B. basierend auf Interaktionen, Reputation etc.
}

// System zum Aktualisieren von 'last_used' (wenn ein Attribut benutzt wird)
// Dieses System würde typischerweise durch Events getriggert werden, die anzeigen,
// dass ein Attribut verwendet wurde.
// Beispiel: Event `AttributeUsedEvent { entity: Entity, attribute_type: AttributeType }`
pub fn update_attribute_usage(
    mut attribute_query: Query<(
        // Query über alle Attribut-Gruppen-Komponenten
        Option<&mut PhysicalAttributes>,
        Option<&mut MentalAttributes>,
        Option<&mut SocialAttributes>,
    )>,
    time: Res<Time>,
    // Hypothetischer Event Reader
    mut ev_attribute_used: EventReader<AttributeUsedEvent>,
) {
    let current_time = time.elapsed();

    for event in ev_attribute_used.read() {
        // Finde die Entität aus dem Event und hole ihre Attribut-Komponenten
        if let Ok((mut phys_opt, mut ment_opt, mut soc_opt)) = attribute_query.get_mut(event.entity)
        {
            let attribute_type = event.attribute_type;

            // Finde das spezifische Attribut über die Gruppen und aktualisiere es
            let mut attribute_found: Option<&mut Attribute> = None;

            if let Some(ref mut phys) = phys_opt {
                if let Some(attr) = phys.get_attribute_mut(attribute_type) {
                    attribute_found = Some(attr);
                }
            }
            if attribute_found.is_none() {
                if let Some(ref mut ment) = ment_opt {
                    if let Some(attr) = ment.get_attribute_mut(attribute_type) {
                        attribute_found = Some(attr);
                    }
                }
            }
            if attribute_found.is_none() {
                if let Some(ref mut soc) = soc_opt {
                    if let Some(attr) = soc.get_attribute_mut(attribute_type) {
                        attribute_found = Some(attr);
                    }
                }
            }

            // Wenn das Attribut gefunden wurde, aktualisiere last_used und entferne Rust
            if let Some(attribute) = attribute_found {
                attribute.last_used = Some(current_time);
                // Wenn ein Attribut verwendet wird, verschwindet der Rost sofort.
                if attribute.rust_level.is_some() {
                    attribute.rust_level = None;
                    // info!("Attribute {:?} used, rust removed.", attribute.id);
                }
            }
        }
    }
}
