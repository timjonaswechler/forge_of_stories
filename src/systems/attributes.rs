// src/systems/attributes.rs
use crate::components::attributes::{
    Attribute, MentalAttributes, PhysicalAttributes, SocialAttributes,
};
use crate::components::genetics::{ChromosomeType, Phenotype};
use crate::components::visual_traits::EyeColor;
use bevy::prelude::*;
use bevy::time::Time;

// Generischer Trait für Attributgruppen
pub trait AttributeGroup {
    fn get_attribute_mut(&mut self, name: &str) -> Option<&mut Attribute>;
}

// Implementierung für PhysicalAttributes
impl AttributeGroup for PhysicalAttributes {
    fn get_attribute_mut(&mut self, name: &str) -> Option<&mut Attribute> {
        match name {
            "strength" => Some(&mut self.strength),
            "agility" => Some(&mut self.agility),
            "toughness" => Some(&mut self.toughness),
            "endurance" => Some(&mut self.endurance),
            "recuperation" => Some(&mut self.recuperation),
            "disease_resistance" => Some(&mut self.disease_resistance),
            _ => None,
        }
    }
}

// Implementierung für MentalAttributes
impl AttributeGroup for MentalAttributes {
    fn get_attribute_mut(&mut self, name: &str) -> Option<&mut Attribute> {
        match name {
            "analytical_ability" => Some(&mut self.analytical_ability),
            "focus" => Some(&mut self.focus),
            "willpower" => Some(&mut self.willpower),
            "creativity" => Some(&mut self.creativity),
            "intuition" => Some(&mut self.intuition),
            "patience" => Some(&mut self.patience),
            "memory" => Some(&mut self.memory),
            "spatial_sense" => Some(&mut self.spatial_sense),
            _ => None,
        }
    }
}

// Implementierung für SocialAttributes
impl AttributeGroup for SocialAttributes {
    fn get_attribute_mut(&mut self, name: &str) -> Option<&mut Attribute> {
        match name {
            "empathy" => Some(&mut self.empathy),
            "social_awareness" => Some(&mut self.social_awareness),
            "linguistic_ability" => Some(&mut self.linguistic_ability),
            "musicality" => Some(&mut self.musicality),
            "leadership" => Some(&mut self.leadership),
            "negotiation" => Some(&mut self.negotiation),
            _ => None,
        }
    }
}

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

// Generisches System zur Anwendung von Phänotypwerten auf Attribute
pub fn apply_attributes<T: AttributeGroup + Component>(
    mut query: Query<(&Phenotype, &mut T)>,
    attribute_prefix: &str,
) {
    for (phenotype, mut attribute_group) in query.iter_mut() {
        // Attributwerte aus der Attribut-Chromosomen-Gruppe holen
        if let Some(attribute_values) = phenotype.attribute_groups.get(&ChromosomeType::Attributes)
        {
            // Für jedes Attribut im Phänotyp prüfen, ob es angewendet werden soll
            for (gene_id, phenotype_gene) in attribute_values.iter() {
                // Nur Gene mit dem richtigen Präfix verwenden
                if gene_id.starts_with(attribute_prefix) {
                    // Attribute-Namen vom Präfix trennen, z.B. "gene_strength" -> "strength"
                    let attribute_name = gene_id.strip_prefix(attribute_prefix).unwrap_or(gene_id);

                    // Attribute aktualisieren, falls es existiert
                    if let Some(attribute) = attribute_group.get_attribute_mut(attribute_name) {
                        attribute.base_value = phenotype_gene.value * 100.0;
                        attribute.current_value = attribute.base_value;
                    }
                }
            }
        }
    }
}

// Die ursprünglichen Systeme können jetzt erheblich vereinfacht werden:

pub fn apply_physical_attributes_system(query: Query<(&Phenotype, &mut PhysicalAttributes)>) {
    apply_attributes::<PhysicalAttributes>(query, "gene_");
}

pub fn apply_mental_attributes_system(query: Query<(&Phenotype, &mut MentalAttributes)>) {
    apply_attributes::<MentalAttributes>(query, "gene_");
}

pub fn apply_social_attributes_system(query: Query<(&Phenotype, &mut SocialAttributes)>) {
    apply_attributes::<SocialAttributes>(query, "gene_");
}

pub fn apply_visual_traits_system(
    mut query: Query<(
        &Phenotype,
        &mut crate::components::visual_traits::VisualTraits,
    )>,
) {
    for (phenotype, mut visual_traits) in query.iter_mut() {
        // Für visuelle Merkmale verwenden wir primär die VisualTraits-Chromosomengruppe
        if let Some(visual_values) = phenotype
            .attribute_groups
            .get(&ChromosomeType::VisualTraits)
        {
            // RGB-Komponenten für Hautfarbe
            if visual_values.contains_key("gene_skin_r")
                && visual_values.contains_key("gene_skin_g")
                && visual_values.contains_key("gene_skin_b")
            {
                visual_traits.skin_color = (
                    visual_values.get("gene_skin_r").unwrap().value,
                    visual_values.get("gene_skin_g").unwrap().value,
                    visual_values.get("gene_skin_b").unwrap().value,
                );
            }

            // RGB-Komponenten für Haarfarbe
            if visual_values.contains_key("gene_hair_r")
                && visual_values.contains_key("gene_hair_g")
                && visual_values.contains_key("gene_hair_b")
            {
                visual_traits.hair_color = (
                    visual_values.get("gene_hair_r").unwrap().value,
                    visual_values.get("gene_hair_g").unwrap().value,
                    visual_values.get("gene_hair_b").unwrap().value,
                );
            }

            // Augenfarbe - jetzt werden wir den numerischen Wert in eine EyeColor umwandeln
            // und dann in RGB-Werte
            if let Some(eye_color_gene) = visual_values.get("gene_eye_color") {
                let eye_color = EyeColor::from_f32(eye_color_gene.value);

                // Zuordnung von EyeColor zu RGB-Werten
                visual_traits.eye_color = match eye_color {
                    EyeColor::Brown => (0.55, 0.27, 0.07),
                    EyeColor::Green => (0.21, 0.47, 0.21),
                    EyeColor::Blue => (0.21, 0.35, 0.80),
                    EyeColor::Gray => (0.50, 0.50, 0.50),
                    EyeColor::Yellow => (0.80, 0.80, 0.20),
                    EyeColor::Red => (0.80, 0.20, 0.20),
                    EyeColor::Black => (0.10, 0.10, 0.10),
                    EyeColor::White => (0.90, 0.90, 0.90),
                };
            }
        }
    }
}

// System zur Aktualisierung der physischen Attribut-Sammlung
//TODO: Implementierung fehlt
pub fn update_physical_attributes(query: Query<&PhysicalAttributes>) {
    for _physical_attrs in query.iter() {
        // Hier könnten zusätzliche Berechnungen für die gesamte Attributgruppe erfolgen
        // z.B. Abhängigkeiten zwischen Attributen
    }
}

// System zur Aktualisierung der mentalen Attribut-Sammlung
//TODO: Implementierung fehlt
pub fn update_mental_attributes(query: Query<&MentalAttributes>) {
    for _mental_attrs in query.iter() {
        // Hier könnten zusätzliche Berechnungen für die gesamte Attributgruppe erfolgen
    }
}

// System zur Aktualisierung der sozialen Attribut-Sammlung
//TODO: Implementierung fehlt
pub fn update_social_attributes(query: Query<&SocialAttributes>) {
    for _social_attrs in query.iter() {
        // Hier könnten zusätzliche Berechnungen für die gesamte Attributgruppe erfolgen
    }
}

// Hilfssystem zur Aktualisierung der "last_used" Zeit für Attribute
//TODO: Implementierung fehlt
#[allow(dead_code)]
pub fn update_attribute_usage(
    mut attribute_query: Query<&mut Attribute>,
    _time: Res<Time>, // Underscore-Präfix um die Warnung zu vermeiden
) {
    // Beispielhafter Rahmen für die Implementierung
    for _attribute in attribute_query.iter_mut() {
        // Hier würde man prüfen, ob das Attribut verwendet wurde
        // und entsprechend last_used aktualisieren
    }
}
