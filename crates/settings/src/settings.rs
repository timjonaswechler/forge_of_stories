//! Minimal Settings-MVP: Trait, Error, und Basis-Hilfen (ohne gpui/Async).

mod location;
mod read_write;
mod source;
mod store;
mod value;

use crate::settings::source::SettingsSources;
use serde::{Serialize, de::DeserializeOwned};
use std::path::PathBuf;

/// Einheitlicher Fehler für das Settings-MVP.
#[derive(thiserror::Error, Debug)]
pub enum SettingsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("Invalid settings structure: {0}")]
    InvalidStructure(&'static str),
    #[error("Unknown setting type")]
    UnknownSettingType,
    #[error("Not implemented")]
    NotImplemented,
}

pub type SettingsResult<T> = Result<T, SettingsError>;

/// Settings-Trait: typisiert, TOML-basiert, ohne gpui.
pub trait Settings: 'static + Send + Sync {
    /// Optionaler Schlüssel, unter dem das FileContent im TOML lebt (z. B. "video", "audio").
    /// None bedeutet: das Root-Objekt entspricht FileContent.
    const KEY: Option<&'static str>;

    /// Optionaler Fallback-Key, wenn KEY im Dokument fehlt (Migration).
    const FALLBACK_KEY: Option<&'static str> = None;

    /// Schlüssel in FileContent, die immer persistiert werden sollen, auch wenn Default.
    const PRESERVED_KEYS: Option<&'static [&'static str]> = None;

    /// Der Typ, der aus einem einzelnen TOML-Dokument geladen/geschrieben wird.
    type FileContent: Clone + Default + Serialize + DeserializeOwned;

    /// Merge-Logik: aus Sources (default + user + später evtl. mehr) das finale Setting bauen.
    fn load(sources: SettingsSources<Self::FileContent>) -> SettingsResult<Self>
    where
        Self: Sized;
}

/// Standardpfad für die Settings-Datei (MVP: eine Datei).
pub fn settings_file() -> PathBuf {
    // Für MVP: config/settings.toml
    paths::config_dir().join("settings.toml")
}
