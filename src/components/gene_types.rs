// src/components/gene_types.rs
use serde::Deserialize;
use std::fmt::{self, Display};
use std::str::FromStr;

/// Repräsentiert alle Gene für Attribute
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum AttributeGene {
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

/// Repräsentiert alle Gene für visuelle Eigenschaften
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum VisualGene {
    SkinColorR,
    SkinColorG,
    SkinColorB,
    HairColorR,
    HairColorG,
    HairColorB,
    EyeColor,
    SkinTone,
}

/// Übergeordneter Enum für alle Gen-Typen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum GeneType {
    Attribute(AttributeGene),
    Visual(VisualGene),
}

/// Fehler für die String-Konvertierung
#[derive(Debug)]
pub struct ParseGeneError {
    message: String,
}

impl Display for ParseGeneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse gene: {}", self.message)
    }
}

impl std::error::Error for ParseGeneError {}

impl Display for GeneType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeneType::Attribute(attr_gene) => write!(f, "gene_{}", attr_gene),
            GeneType::Visual(visual_gene) => write!(f, "gene_{}", visual_gene),
        }
    }
}

impl FromStr for GeneType {
    type Err = ParseGeneError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(stripped) = s.strip_prefix("gene_") {
            // Versuche zuerst, einen Attribut-Gen zu parsen
            if let Ok(attr_gene) = AttributeGene::from_str(stripped) {
                return Ok(GeneType::Attribute(attr_gene));
            }

            // Wenn das fehlschlägt, versuche einen Visuellen Gen zu parsen
            if let Ok(visual_gene) = VisualGene::from_str(stripped) {
                return Ok(GeneType::Visual(visual_gene));
            }
        }

        Err(ParseGeneError {
            message: format!("Unknown gene identifier: {}", s),
        })
    }
}

impl Display for AttributeGene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            // Physical Attributes
            AttributeGene::Strength => "strength",
            AttributeGene::Agility => "agility",
            AttributeGene::Toughness => "toughness",
            AttributeGene::Endurance => "endurance",
            AttributeGene::Recuperation => "recuperation",
            AttributeGene::DiseaseResistance => "disease_resistance",
            // Mental Attributes
            AttributeGene::AnalyticalAbility => "analytical_ability",
            AttributeGene::Focus => "focus",
            AttributeGene::Willpower => "willpower",
            AttributeGene::Creativity => "creativity",
            AttributeGene::Intuition => "intuition",
            AttributeGene::Patience => "patience",
            AttributeGene::Memory => "memory",
            AttributeGene::SpatialSense => "spatial_sense",
            // Social Attributes
            AttributeGene::Empathy => "empathy",
            AttributeGene::SocialAwareness => "social_awareness",
            AttributeGene::LinguisticAbility => "linguistic_ability",
            AttributeGene::Musicality => "musicality",
            AttributeGene::Leadership => "leadership",
            AttributeGene::Negotiation => "negotiation",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for AttributeGene {
    type Err = ParseGeneError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "strength" => Ok(AttributeGene::Strength),
            "agility" => Ok(AttributeGene::Agility),
            "toughness" => Ok(AttributeGene::Toughness),
            "endurance" => Ok(AttributeGene::Endurance),
            "recuperation" => Ok(AttributeGene::Recuperation),
            "disease_resistance" => Ok(AttributeGene::DiseaseResistance),
            "focus" => Ok(AttributeGene::Focus),
            "creativity" => Ok(AttributeGene::Creativity),
            "willpower" => Ok(AttributeGene::Willpower),
            "analytical_ability" => Ok(AttributeGene::AnalyticalAbility),
            "intuition" => Ok(AttributeGene::Intuition),
            "memory" => Ok(AttributeGene::Memory),
            "patience" => Ok(AttributeGene::Patience),
            "spatial_sense" => Ok(AttributeGene::SpatialSense),
            "empathy" => Ok(AttributeGene::Empathy),
            "leadership" => Ok(AttributeGene::Leadership),
            "social_awareness" => Ok(AttributeGene::SocialAwareness),
            "linguistic_ability" => Ok(AttributeGene::LinguisticAbility),
            "negotiation" => Ok(AttributeGene::Negotiation),
            "musicality" => Ok(AttributeGene::Musicality),
            _ => Err(ParseGeneError {
                message: format!("Unknown attribute gene: {}", s),
            }),
        }
    }
}

impl Display for VisualGene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            VisualGene::SkinColorR => "skin_r",
            VisualGene::SkinColorG => "skin_g",
            VisualGene::SkinColorB => "skin_b",
            VisualGene::HairColorR => "hair_r",
            VisualGene::HairColorG => "hair_g",
            VisualGene::HairColorB => "hair_b",
            VisualGene::EyeColor => "eye_color",
            VisualGene::SkinTone => "skin_tone",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for VisualGene {
    type Err = ParseGeneError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "skin_r" => Ok(VisualGene::SkinColorR),
            "skin_g" => Ok(VisualGene::SkinColorG),
            "skin_b" => Ok(VisualGene::SkinColorB),
            "hair_r" => Ok(VisualGene::HairColorR),
            "hair_g" => Ok(VisualGene::HairColorG),
            "hair_b" => Ok(VisualGene::HairColorB),
            "eye_color" => Ok(VisualGene::EyeColor),
            "skin_tone" => Ok(VisualGene::SkinTone),
            _ => Err(ParseGeneError {
                message: format!("Unknown visual gene: {}", s),
            }),
        }
    }
}
