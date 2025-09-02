use paths::asset_str;
use rust_embed::RustEmbed;
use std::{borrow::Cow, str};

#[derive(RustEmbed)]
#[folder = "assets/"]
#[include = "settings/*"]
#[include = "keymaps/*"]
#[exclude = "*.DS_Store"]
pub struct SettingsAssets;

pub fn default_settings() -> Cow<'static, str> {
    asset_str::<SettingsAssets>("settings/global.toml")
}

pub fn default_settings_client() -> Cow<'static, str> {
    asset_str::<SettingsAssets>("settings/client-default.toml")
}

pub fn default_settings_server() -> Cow<'static, str> {
    asset_str::<SettingsAssets>("settings/server-default.toml")
}

pub fn default_settings_cli() -> Cow<'static, str> {
    asset_str::<SettingsAssets>("settings/cli-default.toml")
}

pub fn default_wizard_setting() -> Cow<'static, str> {
    asset_str::<SettingsAssets>("settings/wizard.toml")
}
#[cfg(target_os = "macos")]
pub const DEFAULT_KEYMAP_PATH: &str = "keymaps/default-macos.toml";

#[cfg(target_os = "windows")]
pub const DEFAULT_KEYMAP_PATH: &str = "keymaps/default-windows.toml";

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub const DEFAULT_KEYMAP_PATH: &str = "keymaps/default-linux.toml";

pub const DEFAULT_GAMEPAD_KEYMAP_PATH: &str = "keymaps/default-gamepad.toml";

pub fn default_keymap() -> Cow<'static, str> {
    asset_str::<SettingsAssets>(DEFAULT_KEYMAP_PATH)
}

pub fn default_gamepad_keymap() -> Cow<'static, str> {
    asset_str::<SettingsAssets>(DEFAULT_GAMEPAD_KEYMAP_PATH)
}

pub fn default_wizard_keymap() -> Cow<'static, str> {
    asset_str::<SettingsAssets>("keymaps/default-wizard.toml")
}
