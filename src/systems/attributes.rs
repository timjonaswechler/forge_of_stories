// src/systems/attributes.rs
use crate::components::attributes::{
    Attribute, MentalAttributes, PhysicalAttributes, SocialAttributes,
};
use crate::components::gene_types::{AttributeGene, GeneType, VisualGene};
use crate::components::genetics::{ChromosomeType, Phenotype};
use crate::components::visual_traits::EyeColor;
use bevy::prelude::*;
use bevy::time::Time;
use std::str::FromStr;

// Generischer Trait für Attributgruppen
pub trait AttributeGroup {
    fn get_attribute_mut(&mut self, id: AttributeGene) -> Option<&mut Attribute>;
}

// Implementierungen für PhysicalAttributes, MentalAttributes, SocialAttributes (unverändert)...
impl AttributeGroup for PhysicalAttributes {
    fn get_attribute_mut(&mut self, id: AttributeGene) -> Option<&mut Attribute> {
        match id {
            AttributeGene::Strength => Some(&mut self.strength),
            AttributeGene::Agility => Some(&mut self.agility),
            AttributeGene::Toughness => Some(&mut self.toughness),
            AttributeGene::Endurance => Some(&mut self.endurance),
            AttributeGene::Recuperation => Some(&mut self.recuperation),
            AttributeGene::DiseaseResistance => Some(&mut self.disease_resistance),
            _ => None, // Ignoriere mentale/soziale Gene hier
        }
    }
}
impl AttributeGroup for MentalAttributes {
    fn get_attribute_mut(&mut self, id: AttributeGene) -> Option<&mut Attribute> {
        match id {
            AttributeGene::AnalyticalAbility => Some(&mut self.analytical_ability),
            AttributeGene::Focus => Some(&mut self.focus),
            AttributeGene::Willpower => Some(&mut self.willpower),
            AttributeGene::Creativity => Some(&mut self.creativity),
            AttributeGene::Intuition => Some(&mut self.intuition),
            AttributeGene::Patience => Some(&mut self.patience),
            AttributeGene::Memory => Some(&mut self.memory),
            AttributeGene::SpatialSense => Some(&mut self.spatial_sense),
            _ => None, // Ignoriere physische/soziale Gene hier
        }
    }
}
impl AttributeGroup for SocialAttributes {
    fn get_attribute_mut(&mut self, id: AttributeGene) -> Option<&mut Attribute> {
        match id {
            AttributeGene::Empathy => Some(&mut self.empathy),
            AttributeGene::SocialAwareness => Some(&mut self.social_awareness),
            AttributeGene::LinguisticAbility => Some(&mut self.linguistic_ability),
            AttributeGene::Musicality => Some(&mut self.musicality),
            AttributeGene::Leadership => Some(&mut self.leadership),
            AttributeGene::Negotiation => Some(&mut self.negotiation),
            _ => None, // Ignoriere physische/mentale Gene hier
        }
    }
}

// System zur Berechnung der effektiven Attributwerte
pub fn calculate_effective_attribute_values(mut query: Query<&mut Attribute>) {
    for mut attribute in query.iter_mut() {
        let mut value = attribute.current_value; // Startet mit dem aktuellen Wert (der temporäre Effekte beinhalten könnte)

        // Berücksichtige Rust/Decay
        if let Some(rust) = attribute.rust_level {
            value *= 1.0 - (rust as f32 * 0.05);
        }

        // Begrenze den Wert auf den erlaubten Bereich [0.0, max_value]
        attribute.effective_value = value.clamp(0.0, attribute.max_value);
    }
}

// System für Attributverfall/Rust (unverändert)
pub fn update_attribute_rust(time: Res<Time>, mut query: Query<&mut Attribute>) {
    const RUST_THRESHOLD_DAYS: f32 = 30.0; // 30 Tage ohne Nutzung = 1 Rust-Level

    for mut attribute in query.iter_mut() {
        if let Some(last_used) = attribute.last_used {
            let time_since_used = time.elapsed() - last_used;
            let days_since_used = time_since_used.as_secs_f32() / (24.0 * 60.0 * 60.0);

            if days_since_used > RUST_THRESHOLD_DAYS {
                let new_rust_level = (days_since_used / RUST_THRESHOLD_DAYS).floor() as u8;
                attribute.rust_level = Some(new_rust_level.min(6)); // Maximal 6 Rust-Level
            }
            // Optional: Rust zurücksetzen, wenn Tage < Threshold?
            // else { attribute.rust_level = Some(0) oder None; }
        }
        // Optional: Was passiert, wenn last_used None ist? Soll Rust starten?
        // else { attribute.rust_level = Some(initial_rust); }
    }
}

// Generisches System zur Anwendung von Phänotypwerten auf Attribute
// Reagiert nur auf Änderungen am Phänotyp und verwendet jetzt GeneType
pub fn apply_attributes<T: AttributeGroup + Component>(
    mut query: Query<(&Phenotype, &mut T), Changed<Phenotype>>,
    // Kein attribute_prefix mehr nötig
) {
    for (phenotype, mut attribute_group) in query.iter_mut() {
        if let Some(attribute_values) = phenotype.attribute_groups.get(&ChromosomeType::Attributes)
        {
            // Iteriere über die Gene im Phenotyp für diese Chromosomen-Gruppe
            for (gene_id_str, phenotype_gene) in attribute_values.iter() {
                // Versuche, die String-ID in unseren GeneType zu parsen
                if let Ok(gene_type) = GeneType::from_str(gene_id_str) {
                    // Prüfe, ob es sich um ein Attribut-Gen handelt
                    if let GeneType::Attribute(attribute_gene_id) = gene_type {
                        // Hole das passende mutable Attribut über den AttributeGene Enum
                        if let Some(attribute) =
                            attribute_group.get_attribute_mut(attribute_gene_id)
                        {
                            // Skalierung und Zuweisung wie vorher
                            attribute.base_value = phenotype_gene.value() * attribute.max_value;
                            attribute.base_value =
                                attribute.base_value.clamp(0.0, attribute.max_value);
                            attribute.current_value = attribute.base_value;
                        } else {
                            // Sollte nicht passieren, wenn die AttributeGroup korrekt implementiert ist
                            // und alle AttributeGene-Varianten abdeckt.
                            trace!(
                                "Attribut {:?} nicht in Komponente {} gefunden für Gen ID '{}'.",
                                attribute_gene_id,
                                std::any::type_name::<T>(),
                                gene_id_str
                            );
                        }
                    }
                    // Falls es kein Attribut-Gen ist (z.B. VisualGene), ignoriere es hier einfach.
                } else {
                    // Gen ID konnte nicht geparsed werden (z.B. alte/fremde Gene im Phenotyp?)
                    warn!(
                        "Konnte Gen ID '{}' nicht zu GeneType parsen in apply_attributes.",
                        gene_id_str
                    );
                }
            }
        }
    }
}

// Visuelle Traits Anwendung
pub fn apply_visual_traits_system(
    mut query: Query<
        (
            &Phenotype,
            &mut crate::components::visual_traits::VisualTraits,
        ),
        Changed<Phenotype>,
    >,
) {
    // Generiere die String Keys einmal außerhalb der Loop
    let key_skin_r = GeneType::Visual(VisualGene::SkinColorR).to_string();
    let key_skin_g = GeneType::Visual(VisualGene::SkinColorG).to_string();
    let key_skin_b = GeneType::Visual(VisualGene::SkinColorB).to_string();
    let key_hair_r = GeneType::Visual(VisualGene::HairColorR).to_string();
    let key_hair_g = GeneType::Visual(VisualGene::HairColorG).to_string();
    let key_hair_b = GeneType::Visual(VisualGene::HairColorB).to_string();
    let key_eye_color = GeneType::Visual(VisualGene::EyeColor).to_string();

    for (phenotype, mut visual_traits) in query.iter_mut() {
        if let Some(visual_values) = phenotype
            .attribute_groups
            .get(&ChromosomeType::VisualTraits)
        {
            // Hautfarbe
            let skin_r = visual_values.get(&key_skin_r).map_or(0.5, |g| g.value());
            let skin_g = visual_values.get(&key_skin_g).map_or(0.5, |g| g.value());
            let skin_b = visual_values.get(&key_skin_b).map_or(0.5, |g| g.value());
            visual_traits.skin_color = (
                skin_r.clamp(0.0, 1.0),
                skin_g.clamp(0.0, 1.0),
                skin_b.clamp(0.0, 1.0),
            );

            // Haarfarbe
            let hair_r = visual_values.get(&key_hair_r).map_or(0.5, |g| g.value());
            let hair_g = visual_values.get(&key_hair_g).map_or(0.5, |g| g.value());
            let hair_b = visual_values.get(&key_hair_b).map_or(0.5, |g| g.value());
            visual_traits.hair_color = (
                hair_r.clamp(0.0, 1.0),
                hair_g.clamp(0.0, 1.0),
                hair_b.clamp(0.0, 1.0),
            );

            // Augenfarbe
            if let Some(eye_color_gene) = visual_values.get(&key_eye_color) {
                let eye_color_val = eye_color_gene.value();
                if eye_color_val >= 0.0 {
                    let eye_color = EyeColor::from_f32(eye_color_val);
                    // ... (match eye_color zu RGB wie vorher) ...
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
                } else {
                    warn!(
                        "Ungültiger Genwert für Augenfarbe gefunden: {}",
                        eye_color_val
                    );
                    visual_traits.eye_color = (0.5, 0.5, 0.5);
                }
            } else {
                visual_traits.eye_color = (0.5, 0.5, 0.5);
            }
        }
    }
}

// --- Update-Systeme (unverändert) ---
pub fn update_physical_attributes(query: Query<&PhysicalAttributes>) { /* ... */
}
pub fn update_mental_attributes(query: Query<&MentalAttributes>) { /* ... */
}
pub fn update_social_attributes(query: Query<&SocialAttributes>) { /* ... */
}
pub fn update_attribute_usage(mut attribute_query: Query<&mut Attribute>, _time: Res<Time>) {
    /* ... */
}
