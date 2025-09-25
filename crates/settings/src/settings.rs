pub use crate::store::*;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum SettingsError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid config: {0}")]
    Invalid(&'static str),
    #[error("not registered")]
    NotRegistered,
}

// Der Settings-Trait, um Standardwerte bereitzustellen.
pub trait Settings: Default + Serialize + for<'de> Deserialize<'de> {
    /// Section identifier used in the JSON delta file.
    const SECTION: &'static str;

    /// Backwards-compatible helper; existing code calling `T::name()` still works.
    #[inline]
    fn name() -> &'static str {
        Self::SECTION
    }
}

// Example:
// #[derive(Clone, Serialize, Deserialize, Default)]
// struct Network {
//     port: u16,
// }
// impl Settings for Network {
//     const SECTION: &'static str = "network";
// }
