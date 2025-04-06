// src/components/attributes.rs
use crate::genetics::components::gene_types::AttributeGene;
use bevy::prelude::*;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeCategory {
    Physical,
    Mental,
    Social,
}

#[derive(Component, Debug, Clone)]
pub struct Attribute {
    pub id: AttributeGene,
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
        id: AttributeGene,
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
                AttributeGene::Strength,
                "Stärke",
                AttributeCategory::Physical,
                default_base,
            ),
            agility: Attribute::new(
                AttributeGene::Agility,
                "Beweglichkeit",
                AttributeCategory::Physical,
                default_base,
            ),
            toughness: Attribute::new(
                AttributeGene::Toughness,
                "Widerstandsfähigkeit",
                AttributeCategory::Physical,
                default_base,
            ),
            endurance: Attribute::new(
                AttributeGene::Endurance,
                "Ausdauer",
                AttributeCategory::Physical,
                default_base,
            ),
            recuperation: Attribute::new(
                AttributeGene::Recuperation,
                "Heilungsfähigkeit",
                AttributeCategory::Physical,
                default_base,
            ),
            disease_resistance: Attribute::new(
                AttributeGene::DiseaseResistance,
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
                AttributeGene::AnalyticalAbility,
                "Analytische Fähigkeit",
                AttributeCategory::Mental,
                default_base,
            ),
            focus: Attribute::new(
                AttributeGene::Focus,
                "Konzentration",
                AttributeCategory::Mental,
                default_base,
            ),
            willpower: Attribute::new(
                AttributeGene::Willpower,
                "Willenskraft",
                AttributeCategory::Mental,
                default_base,
            ),
            creativity: Attribute::new(
                AttributeGene::Creativity,
                "Kreativität",
                AttributeCategory::Mental,
                default_base,
            ),
            intuition: Attribute::new(
                AttributeGene::Intuition,
                "Intuition",
                AttributeCategory::Mental,
                default_base,
            ),
            patience: Attribute::new(
                AttributeGene::Patience,
                "Geduld",
                AttributeCategory::Mental,
                default_base,
            ),
            memory: Attribute::new(
                AttributeGene::Memory,
                "Gedächtnis",
                AttributeCategory::Mental,
                default_base,
            ),
            spatial_sense: Attribute::new(
                AttributeGene::SpatialSense,
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
                AttributeGene::Empathy,
                "Empathie",
                AttributeCategory::Social,
                default_base,
            ),
            social_awareness: Attribute::new(
                AttributeGene::SocialAwareness,
                "Soziale Wahrnehmung",
                AttributeCategory::Social,
                default_base,
            ),
            linguistic_ability: Attribute::new(
                AttributeGene::LinguisticAbility,
                "Sprachliche Fähigkeit",
                AttributeCategory::Social,
                default_base,
            ),
            musicality: Attribute::new(
                AttributeGene::Musicality,
                "Musikalität",
                AttributeCategory::Social,
                default_base,
            ),
            leadership: Attribute::new(
                AttributeGene::Leadership,
                "Führungsstärke",
                AttributeCategory::Social,
                default_base,
            ),
            negotiation: Attribute::new(
                AttributeGene::Negotiation,
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

// Generischer Trait für Attributgruppen (Nimmt jetzt AttributeGene)
pub trait AttributeGroup {
    fn get_attribute_mut(&mut self, id: AttributeGene) -> Option<&mut Attribute>;
    // Optional: Eine Methode, um alle Attribute aufzulisten
    // fn get_all_attributes_mut(&mut self) -> Vec<&mut Attribute>;
}

// Implementierung für PhysicalAttributes (Matching auf AttributeGene)
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

// Implementierung für MentalAttributes (Matching auf AttributeGene)
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

// Implementierung für SocialAttributes (Matching auf AttributeGene)
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
