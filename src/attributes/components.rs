// src/components/attributes.rs
use bevy::prelude::*;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeCategory {
    Physical,
    Mental,
    Social,
}

#[derive(Component, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeType {
    Strength,
    Agility,
    Toughness,
    Endurance,
    Recuperation,
    DiseaseResistance,
    Focus,
    Creativity,
    Willpower,
    AnalyticalAbility,
    Intuition,
    Memory,
    Patience,
    SpatialSense,
    Empathy,
    Leadership,
    SocialAwareness,
    LinguisticAbility,
    Negotiation,
    Musicality,
}

#[derive(Component, Debug, Clone)]
pub struct Attribute {
    pub id: AttributeType,
    pub name: String,
    pub category: AttributeCategory,
    pub base_value: f32,
    pub current_value: f32,
    pub effective_value: f32,
    pub max_value: f32,
    pub last_used: Option<Duration>,
    pub rust_level: Option<u8>,
}

// KEINE Default Implementierung für Attribute nötig

impl Attribute {
    pub fn new(
        id: AttributeType,
        name: &str,
        category: AttributeCategory,
        base_value: f32,
    ) -> Self {
        const MAX_ATTRIBUTE_VALUE: f32 = 5000.0;
        Self {
            id,
            name: name.to_string(),
            category,
            base_value,
            current_value: base_value,
            effective_value: base_value,
            max_value: MAX_ATTRIBUTE_VALUE,
            last_used: None,
            rust_level: None,
        }
    }
}

// Physische Attribute Komponente (KEIN Default derive mehr)
#[derive(Component, Debug, Clone)]
pub struct PhysicalAttributes {
    pub strength: Attribute,
    pub agility: Attribute,
    pub toughness: Attribute,
    pub endurance: Attribute,
    pub recuperation: Attribute,
    pub disease_resistance: Attribute,
}

impl PhysicalAttributes {
    pub fn new() -> Self {
        let default_base = 2500.0;
        Self {
            strength: Attribute::new(
                AttributeType::Strength,
                "Stärke",
                AttributeCategory::Physical,
                default_base,
            ),
            agility: Attribute::new(
                AttributeType::Agility,
                "Beweglichkeit",
                AttributeCategory::Physical,
                default_base,
            ),
            toughness: Attribute::new(
                AttributeType::Toughness,
                "Widerstandsfähigkeit",
                AttributeCategory::Physical,
                default_base,
            ),
            endurance: Attribute::new(
                AttributeType::Endurance,
                "Ausdauer",
                AttributeCategory::Physical,
                default_base,
            ),
            recuperation: Attribute::new(
                AttributeType::Recuperation,
                "Heilungsfähigkeit",
                AttributeCategory::Physical,
                default_base,
            ),
            disease_resistance: Attribute::new(
                AttributeType::DiseaseResistance,
                "Krankheitsresistenz",
                AttributeCategory::Physical,
                default_base,
            ),
        }
    }
}
// Füge explizite Default Implementierung hinzu, die ::new() aufruft
impl Default for PhysicalAttributes {
    fn default() -> Self {
        Self::new()
    }
}

// Mentale Attribute Komponente (KEIN Default derive mehr)
#[derive(Component, Debug, Clone)]
pub struct MentalAttributes {
    pub analytical_ability: Attribute,
    pub focus: Attribute,
    pub willpower: Attribute,
    pub creativity: Attribute,
    pub intuition: Attribute,
    pub patience: Attribute,
    pub memory: Attribute,
    pub spatial_sense: Attribute,
}

impl MentalAttributes {
    pub fn new() -> Self {
        let default_base = 2500.0;
        Self {
            analytical_ability: Attribute::new(
                AttributeType::AnalyticalAbility,
                "Analytische Fähigkeit",
                AttributeCategory::Mental,
                default_base,
            ),
            focus: Attribute::new(
                AttributeType::Focus,
                "Konzentration",
                AttributeCategory::Mental,
                default_base,
            ),
            willpower: Attribute::new(
                AttributeType::Willpower,
                "Willenskraft",
                AttributeCategory::Mental,
                default_base,
            ),
            creativity: Attribute::new(
                AttributeType::Creativity,
                "Kreativität",
                AttributeCategory::Mental,
                default_base,
            ),
            intuition: Attribute::new(
                AttributeType::Intuition,
                "Intuition",
                AttributeCategory::Mental,
                default_base,
            ),
            patience: Attribute::new(
                AttributeType::Patience,
                "Geduld",
                AttributeCategory::Mental,
                default_base,
            ),
            memory: Attribute::new(
                AttributeType::Memory,
                "Gedächtnis",
                AttributeCategory::Mental,
                default_base,
            ),
            spatial_sense: Attribute::new(
                AttributeType::SpatialSense,
                "Räumliches Vorstellungsvermögen",
                AttributeCategory::Mental,
                default_base,
            ),
        }
    }
}
// Füge explizite Default Implementierung hinzu
impl Default for MentalAttributes {
    fn default() -> Self {
        Self::new()
    }
}

// Soziale Attribute Komponente (KEIN Default derive mehr)
#[derive(Component, Debug, Clone)]
pub struct SocialAttributes {
    pub empathy: Attribute,
    pub social_awareness: Attribute,
    pub linguistic_ability: Attribute,
    pub musicality: Attribute,
    pub leadership: Attribute,
    pub negotiation: Attribute,
}

impl SocialAttributes {
    pub fn new() -> Self {
        let default_base = 2500.0;
        Self {
            empathy: Attribute::new(
                AttributeType::Empathy,
                "Empathie",
                AttributeCategory::Social,
                default_base,
            ),
            social_awareness: Attribute::new(
                AttributeType::SocialAwareness,
                "Soziale Wahrnehmung",
                AttributeCategory::Social,
                default_base,
            ),
            linguistic_ability: Attribute::new(
                AttributeType::LinguisticAbility,
                "Sprachliche Fähigkeit",
                AttributeCategory::Social,
                default_base,
            ),
            musicality: Attribute::new(
                AttributeType::Musicality,
                "Musikalität",
                AttributeCategory::Social,
                default_base,
            ),
            leadership: Attribute::new(
                AttributeType::Leadership,
                "Führungsstärke",
                AttributeCategory::Social,
                default_base,
            ),
            negotiation: Attribute::new(
                AttributeType::Negotiation,
                "Verhandlungsgeschick",
                AttributeCategory::Social,
                default_base,
            ),
        }
    }
}
// Füge explizite Default Implementierung hinzu
impl Default for SocialAttributes {
    fn default() -> Self {
        Self::new()
    }
}

// -- AttributeGroup Trait Anpassung --

// Generischer Trait für Attributgruppen (Nimmt jetzt AttributeType)
pub trait AttributeGroup {
    fn get_attribute_mut(&mut self, id: AttributeType) -> Option<&mut Attribute>;
    // Optional: Eine Methode, um alle Attribute aufzulisten
    // fn get_all_attributes_mut(&mut self) -> Vec<&mut Attribute>;
}

// Implementierung für PhysicalAttributes (Matching auf AttributeType)
impl AttributeGroup for PhysicalAttributes {
    fn get_attribute_mut(&mut self, id: AttributeType) -> Option<&mut Attribute> {
        match id {
            AttributeType::Strength => Some(&mut self.strength),
            AttributeType::Agility => Some(&mut self.agility),
            AttributeType::Toughness => Some(&mut self.toughness),
            AttributeType::Endurance => Some(&mut self.endurance),
            AttributeType::Recuperation => Some(&mut self.recuperation),
            AttributeType::DiseaseResistance => Some(&mut self.disease_resistance),
            _ => None, // Ignoriere mentale/soziale Gene hier
        }
    }
}

// Implementierung für MentalAttributes (Matching auf AttributeType)
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
            _ => None, // Ignoriere physische/soziale Gene hier
        }
    }
}

// Implementierung für SocialAttributes (Matching auf AttributeType)
impl AttributeGroup for SocialAttributes {
    fn get_attribute_mut(&mut self, id: AttributeType) -> Option<&mut Attribute> {
        match id {
            AttributeType::Empathy => Some(&mut self.empathy),
            AttributeType::SocialAwareness => Some(&mut self.social_awareness),
            AttributeType::LinguisticAbility => Some(&mut self.linguistic_ability),
            AttributeType::Musicality => Some(&mut self.musicality),
            AttributeType::Leadership => Some(&mut self.leadership),
            AttributeType::Negotiation => Some(&mut self.negotiation),
            _ => None, // Ignoriere physische/mentale Gene hier
        }
    }
}
