// src/components/attributes.rs
use bevy::prelude::*;
use std::time::Duration;

// Attribut-Kategorien
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeCategory {
    Physical,
    Mental,
    Social,
}

// Attribut-Komponente
#[derive(Component, Debug, Clone)]
#[allow(dead_code)]
pub struct Attribute {
    pub id: String,
    pub name: String,
    pub category: AttributeCategory,
    pub base_value: f32,             // Grundwert (0.0-100.0)
    pub current_value: f32,          // Aktueller Wert mit temporären Modifikatoren
    pub effective_value: f32,        // Berechneter Wert mit allen Modifikatoren
    pub max_value: f32,              // Maximaler Wert (normalerweise 100.0)
    pub last_used: Option<Duration>, // Wann das Attribut zuletzt verwendet wurde
    pub rust_level: Option<u8>,      // 0-6 wie in DF
}

impl Attribute {
    pub fn new(id: &str, name: &str, category: AttributeCategory, base_value: f32) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            category,
            base_value,
            current_value: base_value,
            effective_value: base_value,
            max_value: 100.0,
            last_used: None,
            rust_level: None,
        }
    }
}

// Physische Attribute Komponente
#[derive(Component, Debug, Clone)]
pub struct PhysicalAttributes {
    pub strength: Attribute,           // Stärke, Tragfähigkeit, Nahkampfschaden
    pub agility: Attribute,            // Geschwindigkeit, Geschicklichkeit
    pub toughness: Attribute,          // Widerstand gegen Schaden
    pub endurance: Attribute,          // Ausdauer, Widerstand gegen Erschöpfung
    pub recuperation: Attribute,       // Heilungsrate
    pub disease_resistance: Attribute, // Widerstand gegen Krankheiten
}
impl PhysicalAttributes {
    pub fn new() -> Self {
        Self {
            strength: Attribute::new("strength", "Stärke", AttributeCategory::Physical, 50.0),
            agility: Attribute::new(
                "agility",
                "Beweglichkeit",
                AttributeCategory::Physical,
                50.0,
            ),
            toughness: Attribute::new(
                "toughness",
                "Widerstandsfähigkeit",
                AttributeCategory::Physical,
                50.0,
            ),
            endurance: Attribute::new("endurance", "Ausdauer", AttributeCategory::Physical, 50.0),
            recuperation: Attribute::new(
                "recuperation",
                "Heilungsfähigkeit",
                AttributeCategory::Physical,
                50.0,
            ),
            disease_resistance: Attribute::new(
                "disease_resistance",
                "Krankheitsresistenz",
                AttributeCategory::Physical,
                50.0,
            ),
        }
    }
}

// Mentale Attribute Komponente
#[derive(Component, Debug, Clone)]
pub struct MentalAttributes {
    pub analytical_ability: Attribute, // Problemlösung, Forschung
    pub focus: Attribute,              // Konzentration, Präzisionsarbeit
    pub willpower: Attribute,          // Stressresistenz, mentale Stärke
    pub creativity: Attribute,         // Kunstfertigkeit, Ideenreichtum
    pub intuition: Attribute,          // Entscheidungsfindung
    pub patience: Attribute,           // Geduld bei langwierigen Aufgaben
    pub memory: Attribute,             // Lernfähigkeit
    pub spatial_sense: Attribute,      // Konstruktion, Navigation
}

impl MentalAttributes {
    pub fn new() -> Self {
        Self {
            analytical_ability: Attribute::new(
                "analytical_ability",
                "Analytische Fähigkeit",
                AttributeCategory::Mental,
                50.0,
            ),
            focus: Attribute::new("focus", "Konzentration", AttributeCategory::Mental, 50.0),
            willpower: Attribute::new("willpower", "Willenskraft", AttributeCategory::Mental, 50.0),
            creativity: Attribute::new(
                "creativity",
                "Kreativität",
                AttributeCategory::Mental,
                50.0,
            ),
            intuition: Attribute::new("intuition", "Intuition", AttributeCategory::Mental, 50.0),
            patience: Attribute::new("patience", "Geduld", AttributeCategory::Mental, 50.0),
            memory: Attribute::new("memory", "Gedächtnis", AttributeCategory::Mental, 50.0),
            spatial_sense: Attribute::new(
                "spatial_sense",
                "Räumliches Vorstellungsvermögen",
                AttributeCategory::Mental,
                50.0,
            ),
        }
    }
}

// Soziale Attribute Komponente
#[derive(Component, Debug, Clone)]
pub struct SocialAttributes {
    pub empathy: Attribute,            // Verständnis für andere
    pub social_awareness: Attribute,   // Soziale Intelligenz
    pub linguistic_ability: Attribute, // Kommunikationsfähigkeit
    pub leadership: Attribute,         // Führungsqualitäten
    pub negotiation: Attribute,        // Handel, Diplomatie
}
impl SocialAttributes {
    pub fn new() -> Self {
        Self {
            empathy: Attribute::new("empathy", "Empathie", AttributeCategory::Social, 50.0),
            social_awareness: Attribute::new(
                "social_awareness",
                "Soziale Wahrnehmung",
                AttributeCategory::Social,
                50.0,
            ),
            linguistic_ability: Attribute::new(
                "linguistic_ability",
                "Sprachliche Fähigkeit",
                AttributeCategory::Social,
                50.0,
            ),
            leadership: Attribute::new(
                "leadership",
                "Führungsstärke",
                AttributeCategory::Social,
                50.0,
            ),
            negotiation: Attribute::new(
                "negotiation",
                "Verhandlungsgeschick",
                AttributeCategory::Social,
                50.0,
            ),
        }
    }
}
