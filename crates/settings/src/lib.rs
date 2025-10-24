#[cfg(feature = "bevy")]
mod bevy_adapter;

pub(crate) mod json_merge;
pub(crate) mod json_utils;
pub(crate) mod settings;
pub(crate) mod store;

#[cfg(feature = "bevy")]
pub use bevy_adapter::{AppSettingsExt, SettingsArc, settings_reload_system};

pub use settings::{Settings, SettingsError, SettingsStore};
