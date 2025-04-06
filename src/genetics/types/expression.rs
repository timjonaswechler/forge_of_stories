// src/genetics/types/expression.rs
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum GeneExpression {
    Dominant,
    Recessive,
    Codominant,
}
