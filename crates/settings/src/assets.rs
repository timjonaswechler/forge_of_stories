use rust_embed::RustEmbed;
use std::{borrow::Cow, fmt, str};
use util::asset_str;

#[derive(RustEmbed)]
#[folder = "assets/"]
#[include = "settings/*"]
#[include = "keymaps/*"]
#[exclude = "*.DS_Store"]
pub struct SettingsAssets;

pub fn default_settings() -> Cow<'static, str> {
    asset_str::<SettingsAssets>("settings/default.toml")
}

#[cfg(target_os = "macos")]
pub const DEFAULT_KEYMAP_PATH: &str = "keymaps/default-macos.toml";

#[cfg(target_os = "windows")]
pub const DEFAULT_KEYMAP_PATH: &str = "keymaps/default-windows.toml";

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub const DEFAULT_KEYMAP_PATH: &str = "keymaps/default-linux.toml";

pub fn default_keymap() -> Cow<'static, str> {
    asset_str::<SettingsAssets>(DEFAULT_KEYMAP_PATH)
}
