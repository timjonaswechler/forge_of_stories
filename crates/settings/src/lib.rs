#[cfg(feature = "bevy")]
mod bevy_adapter;

pub(crate) mod settings;
pub(crate) mod store;

#[cfg(feature = "bevy")]
pub use bevy_adapter::*;

pub use settings::{Settings, SettingsError, SettingsStore};
