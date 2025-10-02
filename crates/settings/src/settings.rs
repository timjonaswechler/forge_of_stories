pub use crate::store::*;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

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

    /// Perform in-place schema migrations for this section.
    ///
    /// This method is **only called when a migration is actually needed**, i.e., when
    /// `file_version < target_version`. It will not be called for unversioned files
    /// or when the file version already matches the target version.
    ///
    /// `file_version` reflects the schema version recorded in the settings file
    /// (guaranteed to be `Some` when this method is called) and `target_version` is
    /// the version the application expects (usually the crate version).
    ///
    /// Return the transformed data plus a flag indicating whether any changes were applied.
    #[allow(unused_variables)]
    fn migrate(
        file_version: Option<&Version>,
        target_version: &Version,
        data: JsonValue,
    ) -> Result<(JsonValue, bool), SettingsError> {
        let _ = file_version;
        let _ = target_version;
        Ok((data, false))
    }

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
