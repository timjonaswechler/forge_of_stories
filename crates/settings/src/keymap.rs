use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------- Keymaps: Datenmodell & Merge pro Gerät ----------

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum DeviceKind {
    Keyboard,
    Mouse,
    Gamepad,
}

bitflags::bitflags! {
    #[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Hash, Debug)]
    pub struct Mods: u8 { const CTRL=1; const SHIFT=2; const ALT=4; const META=8; }
}

/// Merkt sich den ursprünglichen Präfix (z. B. "xbox", "dualshock", "mouse")
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct KeyChord {
    pub device: DeviceKind,
    pub mods: Mods,
    pub key: String,
    pub origin_prefix: Option<String>, // <- neu
}

/// Auswahl, welche Eingabeart exportiert werden soll
#[derive(Clone, Debug)]
pub enum DeviceFilter {
    Keyboard,
    Mouse,
    GamepadAny,
    GamepadKind(String), // z. B. "xbox", "dualshock"
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KeymapMeta {
    pub devices: Vec<DeviceKind>,        // optional: bevorzugte Geräte
    pub version: String,                 //
    pub gamepad_profile: Option<String>, // "xbox" | "dualshock" | ...
    pub mouse_enabled: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct MergedKeymaps {
    pub meta: KeymapMeta,
    // context -> action -> device -> chords
    pub contexts: HashMap<String, HashMap<String, HashMap<DeviceKind, Vec<KeyChord>>>>,
}

#[derive(Clone, Debug, Default)]
pub enum InputScheme {
    #[default]
    KeyboardMouse,
    Gamepad {
        kind: String,
    },
}

#[derive(Clone, Debug, Default)]
pub struct KeymapState {
    pub scheme: InputScheme,
}
