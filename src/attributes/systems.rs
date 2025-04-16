use crate::attributes::components::{
    Attribute, AttributeType, MentalAttributes, PhysicalAttributes, SocialAttributes,
};

// ÄNDERUNG: Importiere VisualTraits direkt
use bevy::prelude::*;
use bevy::time::Time;

// Generischer Trait für Attributgruppen
pub trait AttributeGroup {
    // <- Du definierst den Trait hier
    fn get_attribute_mut(&mut self, id: AttributeType) -> Option<&mut Attribute>;
}

// Implementierungen für PhysicalAttributes, MentalAttributes, SocialAttributes (unverändert)...
// (Die Implementierungen sind korrekt hier platziert, da der Trait hier definiert wird)
// ... (Implementierungen für PhysicalAttributes, MentalAttributes, SocialAttributes) ...
impl AttributeGroup for PhysicalAttributes {
    fn get_attribute_mut(&mut self, id: AttributeType) -> Option<&mut Attribute> {
        match id {
            AttributeType::Strength => Some(&mut self.strength),
            AttributeType::Agility => Some(&mut self.agility),
            AttributeType::Toughness => Some(&mut self.toughness),
            AttributeType::Endurance => Some(&mut self.endurance),
            AttributeType::Recuperation => Some(&mut self.recuperation),
            AttributeType::DiseaseResistance => Some(&mut self.disease_resistance),
            _ => None,
        }
    }
}
impl AttributeGroup for MentalAttributes {
    fn get_attribute_mut(&mut self, id: AttributeType) -> Option<&mut Attribute> {
        match id {
            AttributeType::AnalyticalAbility => Some(&mut self.analytical_ability),
            AttributeType::Focus => Some(&mut self.focus),
            AttributeType::Willpower => Some(&mut self.willpower),
            AttributeType::Creativity => Some(&mut self.creativity),
            AttributeType::Intuition => Some(&mut self.intuition),
            AttributeType::Patience => Some(&mut self.patience),
            AttributeType::Memory => Some(&mut self.memory),
            AttributeType::SpatialSense => Some(&mut self.spatial_sense),
            _ => None,
        }
    }
}
impl AttributeGroup for SocialAttributes {
    fn get_attribute_mut(&mut self, id: AttributeType) -> Option<&mut Attribute> {
        match id {
            AttributeType::Empathy => Some(&mut self.empathy),
            AttributeType::SocialAwareness => Some(&mut self.social_awareness),
            AttributeType::LinguisticAbility => Some(&mut self.linguistic_ability),
            AttributeType::Musicality => Some(&mut self.musicality),
            AttributeType::Leadership => Some(&mut self.leadership),
            AttributeType::Negotiation => Some(&mut self.negotiation),
            _ => None,
        }
    }
}
// System zur Berechnung der effektiven Attributwerte
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
    const RUST_THRESHOLD_DAYS: f32 = 30.0;

    for mut attribute in query.iter_mut() {
        if let Some(last_used) = attribute.last_used {
            let time_since_used = time.elapsed() - last_used;
            let days_since_used = time_since_used.as_secs_f32() / (24.0 * 60.0 * 60.0);

            if days_since_used > RUST_THRESHOLD_DAYS {
                let new_rust_level = (days_since_used / RUST_THRESHOLD_DAYS).floor() as u8;
                attribute.rust_level = Some(new_rust_level.min(6));
            }
        }
    }
}

// Platzhalter-System, korrigiert für Warnungen
pub fn apply_attributes<T: AttributeGroup + Component>(
    mut query: Query<(&mut T, &AttributeType)>,
    _commands: Commands, // Markiert als unbenutzt
) {
    for (mut attributes, attribute_type) in query.iter_mut() {
        if let Some(_attribute) = attributes.get_attribute_mut(*attribute_type) { // Markiert als unbenutzt
             // Hier wird das Attribut angewendet
             // Beispiel: commands.spawn().insert(attribute.clone());
             // Platzhalter - Logik hier einfügen
             // info!("Applying attribute: {:?}", attribute.id); // Beispiel-Log
        }
    }
}
// ... (restliche Systeme unverändert, aber stelle sicher, dass sie Parameter korrekt verwenden oder markieren) ...
pub fn update_physical_attributes(_query: Query<&PhysicalAttributes>) {}
pub fn update_mental_attributes(_query: Query<&MentalAttributes>) {}
pub fn update_social_attributes(_query: Query<&SocialAttributes>) {}
pub fn update_attribute_usage(mut _attribute_query: Query<&mut Attribute>, _time: Res<Time>) {}
