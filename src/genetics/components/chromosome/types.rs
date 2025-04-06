// src/genetics/components/core_types/chromosome.rs
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum ChromosomeType {
    BodyStructure,
    Attributes,
    VisualTraits,
}
