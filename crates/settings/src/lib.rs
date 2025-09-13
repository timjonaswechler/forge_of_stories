#[cfg(feature = "bevy")]
mod bevy_adapter;
pub(crate) mod embedded;
pub mod keymap; // Make keymap public for ActionRegistry trait
pub(crate) mod settings;
pub(crate) mod store;

#[cfg(feature = "bevy")]
pub use bevy_adapter::*;
pub use embedded::*;
pub use keymap::{ActionRegistry, DeviceFilter}; // Also export ActionRegistry
pub use settings::{Settings, SettingsStore};

pub fn parse_bool(s: &str) -> color_eyre::Result<bool> {
    let v = s.trim().to_ascii_lowercase();
    match v.as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(color_eyre::eyre::eyre!("invalid bool value: {s}")),
    }
}
