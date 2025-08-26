use crate::settings::{Settings, source::SettingsSources};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Base key bindings scheme. Base keymaps can be overridden with user keymaps.
///
/// Default: Keyboard
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum BaseKeymap {
    #[default]
    Keyboard,
    Gamepad,
    None,
}

impl Display for BaseKeymap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BaseKeymap::Keyboard => write!(f, "Keyboard"),
            BaseKeymap::Gamepad => write!(f, "Gamepad"),
            BaseKeymap::None => write!(f, "None"),
        }
    }
}

impl BaseKeymap {
    pub const OPTIONS: [(&'static str, Self); 2] = [
        ("Keyboard (Default)", Self::Keyboard),
        ("Gamepad", Self::Gamepad),
    ];

    pub fn asset_path(&self) -> Option<&'static str> {
        match self {
            BaseKeymap::Keyboard => {
                #[cfg(target_os = "macos")]
                {
                    return Some("keymaps/macos/keyboard.toml");
                }
                #[cfg(target_os = "windows")]
                {
                    return Some("keymaps/windows/keyboard.toml");
                }
                #[cfg(not(any(target_os = "windows", target_os = "macos")))]
                {
                    return Some("keymaps/linux/keyboard.toml");
                }
                #[allow(unreachable_code)]
                None
            }
            BaseKeymap::Gamepad => Some("keymaps/gamepad.toml"),
            BaseKeymap::None => None,
        }
    }

    pub fn names() -> impl Iterator<Item = &'static str> {
        Self::OPTIONS.iter().map(|(name, _)| *name)
    }

    pub fn from_names(option: &str) -> BaseKeymap {
        Self::OPTIONS
            .iter()
            .copied()
            .find_map(|(name, value)| (name == option).then_some(value))
            .unwrap_or_default()
    }
}

impl Settings for BaseKeymap {
    const KEY: Option<&'static str> = Some("base_keymap");
    type FileContent = Option<Self>;
    fn load(s: SettingsSources<Self::FileContent>) -> anyhow::Result<Self> {
        if let Some(Some(v)) = s.user.copied() {
            return Ok(v);
        }
        if let Some(Some(v)) = s.server.copied() {
            return Ok(v);
        }
        s.default.ok_or_else(Self::missing_default)
    }
}
