use crate::key_path::KeyPath;
use crate::errors::SettingError;
use toml_edit::Item;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SourceKind {
    Defaults,
    Global,
    User,
    Profiles,
    World,
    Server,
    Keybinds,
}

/// Trait representing a single source of settings.
pub trait SettingSource: Send + Sync {
    fn kind(&self) -> SourceKind;
    fn precedence(&self) -> i32;
    fn is_writable(&self) -> bool;

    /// Load (or reload) the internal representation from backing storage.
    fn load(&mut self) -> Result<(), SettingError>;

    /// Get a single item by key path from the loaded representation.
    fn get(&self, key_path: &KeyPath) -> Result<Option<Item>, SettingError>;

    /// Set a value in the loaded representation. Allowed only if writable.
    fn set(&mut self, key_path: &KeyPath, value: Item) -> Result<(), SettingError>;

    /// Persist the loaded representation to the backing store (if applicable).
    fn persist(&self) -> Result<(), SettingError>;
}
