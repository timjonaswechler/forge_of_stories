use std::fmt;

/// Simple KeyPath type: a Vec<String> with convenience helpers.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyPath(pub Vec<String>);

impl KeyPath {
    pub fn new(parts: impl Into<Vec<String>>) -> Self {
        KeyPath(parts.into())
    }

    pub fn from_slice(parts: &[&str]) -> Self {
        KeyPath(parts.iter().map(|s| s.to_string()).collect())
    }

    pub fn push(&mut self, part: impl Into<String>) {
        self.0.push(part.into());
    }

    pub fn as_slice(&self) -> &[String] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for KeyPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join("."))
    }
}
