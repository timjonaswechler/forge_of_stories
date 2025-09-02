pub use crate::store::*;
use serde::{Serialize, de::DeserializeOwned};
use toml::Value;

#[derive(thiserror::Error, Debug)]
pub enum SettingsError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml de: {0}")]
    TomlDe(#[from] toml::de::Error),
    #[error("toml ser: {0}")]
    TomlSer(#[from] toml::ser::Error),
    #[error("invalid config: {0}")]
    Invalid(&'static str),
    #[error("not registered")]
    NotRegistered,
}

pub trait Settings: Send + Sync + 'static {
    const SECTION: &'static str;
    type Model: DeserializeOwned + Serialize + Clone + Default;
    fn migrate(v: Value) -> Result<Value, SettingsError> {
        Ok(v)
    }
    fn validate(_m: &Self::Model) -> Result<(), SettingsError> {
        Ok(())
    }
}
