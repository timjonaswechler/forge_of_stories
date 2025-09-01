use color_eyre::Result;
use std::env;

/// Environment variable to toggle environment-based layers on/off for the wizard.
/// Any of: "0", "false", "no", "off" (case-insensitive) disable; everything else enables.
/// Default: enabled (true).
pub const ENV_WIZARD_ENV_LAYERS: &str = "FOS_WIZARD_ENV_LAYERS";

/// Environment prefix used to read environment-based settings when env layers are enabled.
/// Example (conceptual): FOS_WIZARD__NETWORK__PORT=12345 -> [network].port = 12345
/// Note: The actual Env->TOML mapping needs to be provided by the settings crate.
pub const ENV_WIZARD_PREFIX: &str = "FOS_WIZARD";

/// Build the SettingsStore for the Wizard:
/// - Load embedded wizard defaults (settings/keymap) from the settings crate's assets
/// - Add optional per-user overrides from the platform config dir via `with_user_config_dir()`
/// - Optionally add/disable the environment layer controlled by `FOS_WIZARD_ENV_LAYERS`
///
/// Layer priority (last-wins):
/// 1) Embedded defaults
/// 2) User config files (if present)
/// 3) Environment layer (if enabled)
pub fn build_wizard_settings_store() -> Result<settings::SettingsStore> {
    let mut builder = settings::SettingsStore::builder()
        // Embedded Wizard defaults (assets bundled by the settings crate)
        .with_embedded_setting_asset("settings/wizard.toml")
        .with_embedded_keymap_asset("keymaps/default-wizard.toml")
        // Optional user config files under <config_dir>/<app_id>/{settings,keymap}.toml
        .with_user_config_dir();

    // Optional: environment layer toggle (default: enabled)
    let env_layers_enabled = read_bool_env(ENV_WIZARD_ENV_LAYERS).unwrap_or(true);
    builder = builder.enable_env_layers(env_layers_enabled);
    if env_layers_enabled {
        // Register env prefix layer for wizard (no-op if env mapping is not implemented)
        builder = builder.with_env_prefix(ENV_WIZARD_PREFIX);
    }

    Ok(builder.build()?)
}

/// Parse a boolean from environment variables with common truthy/falsey values:
/// "1", "true", "yes", "on" => true
/// "0", "false", "no", "off" => false
/// Any other/non-UTF8/empty value => None
fn read_bool_env(var: &str) -> Option<bool> {
    let s = env::var(var).ok()?;
    let v = s.trim().to_ascii_lowercase();
    match v.as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}
