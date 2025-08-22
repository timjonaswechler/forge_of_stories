# Plan: crates/settings/src/game_input_profile.rs (Game Version)

## Overview
This file implements the **GameInputProfile system** - an enum-based system for selecting different game control schemes (FPS, RTS, MMORPG, etc.) with platform-specific TOML asset management.

## Imports
```rust
use std::fmt::{Display, Formatter};
use crate::{Settings, SettingsSources};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use bevy::prelude::*;
```
- **Standard Display**: For user-friendly names in game UI
- **Settings Integration**: Implements the game settings system
- **JSON Schema**: For autocomplete and validation in settings files
- **Serialization**: For TOML-based configuration
- **Bevy**: Game engine integration

## GameInputProfile Enum
```rust
#[derive(Copy, Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default, Resource)]
pub enum GameInputProfile {
    #[default]
    Standard,
    FPS,
    RTS,
    MMORPG,
    Racing,
    Platformer,
    Fighting,
    FlightSim,
    Custom,
}
```

### Enum Design Features:
1. **Copy Trait**: Lightweight - only an enum discriminant
2. **Default = Standard**: Generic game controls for most players
3. **Genre Coverage**: Covers major game genres with different control needs
4. **JsonSchema**: Automatic schema generation for settings validation
5. **Custom Option**: Enables completely custom control schemes
6. **Bevy Resource**: Integrates directly with Bevy's ECS system

### Supported Game Genres:
- **Standard**: Default choice, general game controls
- **FPS**: First-person shooter optimized (WASD, mouse look, etc.)
- **RTS**: Real-time strategy (camera controls, selection, etc.)
- **MMORPG**: MMO-style with many keybinds and UI navigation
- **Racing**: Racing game controls (throttle, brake, steering)
- **Platformer**: 2D platformer controls (jump, run, dash)
- **Fighting**: Fighting game inputs (combos, special moves)
- **FlightSim**: Flight simulator controls (pitch, yaw, roll, throttle)
- **Custom**: No base profile - completely user-defined

## Display Implementation
```rust
impl Display for GameInputProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GameInputProfile::Standard => write!(f, "Standard"),
            GameInputProfile::FPS => write!(f, "First Person Shooter"),
            GameInputProfile::RTS => write!(f, "Real-Time Strategy"),
            GameInputProfile::MMORPG => write!(f, "MMORPG"),
            GameInputProfile::Racing => write!(f, "Racing"),
            GameInputProfile::Platformer => write!(f, "Platformer"),
            GameInputProfile::Fighting => write!(f, "Fighting"),
            GameInputProfile::FlightSim => write!(f, "Flight Simulator"),
            GameInputProfile::Custom => write!(f, "Custom"),
        }
    }
}
```

### Display Features:
1. **Human-Readable Names**: "First Person Shooter" instead of "FPS"
2. **Genre-Specific Labels**: Clear indication of intended game type
3. **UI Integration**: Used for settings UI and in-game menus

## Platform-Specific Options
```rust
#[cfg(target_os = "windows")]
pub const OPTIONS: [(&'static str, Self); 9] = [
    ("Standard (Default)", Self::Standard),
    ("First Person Shooter", Self::FPS),
    ("Real-Time Strategy", Self::RTS),
    ("MMORPG", Self::MMORPG),
    ("Racing", Self::Racing),
    ("Platformer", Self::Platformer),
    ("Fighting", Self::Fighting),
    ("Flight Simulator", Self::FlightSim),
    ("Custom", Self::Custom),
];

#[cfg(target_os = "macos")]
pub const OPTIONS: [(&'static str, Self); 9] = [
    ("Standard (Default)", Self::Standard),
    ("First Person Shooter", Self::FPS),
    ("Real-Time Strategy", Self::RTS),
    ("MMORPG", Self::MMORPG),
    ("Racing", Self::Racing),
    ("Platformer", Self::Platformer),
    ("Fighting", Self::Fighting),
    ("Flight Simulator", Self::FlightSim),
    ("Custom", Self::Custom),
];

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub const OPTIONS: [(&'static str, Self); 9] = [
    ("Standard (Default)", Self::Standard),
    ("First Person Shooter", Self::FPS),
    ("Real-Time Strategy", Self::RTS),
    ("MMORPG", Self::MMORPG),
    ("Racing", Self::Racing),
    ("Platformer", Self::Platformer),
    ("Fighting", Self::Fighting),
    ("Flight Simulator", Self::FlightSim),
    ("Custom", Self::Custom),
];
```

### Platform Features:
- **Consistent Options**: Same profiles available across all platforms
- **Platform-Specific Bindings**: Different default keys per OS
- **UI Labels**: Includes "(Default)" marker for Standard profile

## Asset Path Mapping
```rust
pub fn asset_path(&self) -> Option<&'static str> {
    #[cfg(target_os = "windows")]
    match self {
        GameInputProfile::FPS => Some("input_bindings/windows/fps.toml"),
        GameInputProfile::RTS => Some("input_bindings/windows/rts.toml"),
        GameInputProfile::MMORPG => Some("input_bindings/windows/mmorpg.toml"),
        GameInputProfile::Racing => Some("input_bindings/windows/racing.toml"),
        GameInputProfile::Platformer => Some("input_bindings/windows/platformer.toml"),
        GameInputProfile::Fighting => Some("input_bindings/windows/fighting.toml"),
        GameInputProfile::FlightSim => Some("input_bindings/windows/flightsim.toml"),
        GameInputProfile::Standard => None,  // Uses default input bindings
        GameInputProfile::Custom => None,    // No base bindings to load
    }
    
    #[cfg(target_os = "macos")]
    match self {
        GameInputProfile::FPS => Some("input_bindings/macos/fps.toml"),
        GameInputProfile::RTS => Some("input_bindings/macos/rts.toml"),
        GameInputProfile::MMORPG => Some("input_bindings/macos/mmorpg.toml"),
        GameInputProfile::Racing => Some("input_bindings/macos/racing.toml"),
        GameInputProfile::Platformer => Some("input_bindings/macos/platformer.toml"),
        GameInputProfile::Fighting => Some("input_bindings/macos/fighting.toml"),
        GameInputProfile::FlightSim => Some("input_bindings/macos/flightsim.toml"),
        GameInputProfile::Standard => None,
        GameInputProfile::Custom => None,
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    match self {
        GameInputProfile::FPS => Some("input_bindings/linux/fps.toml"),
        GameInputProfile::RTS => Some("input_bindings/linux/rts.toml"),
        GameInputProfile::MMORPG => Some("input_bindings/linux/mmorpg.toml"),
        GameInputProfile::Racing => Some("input_bindings/linux/racing.toml"),
        GameInputProfile::Platformer => Some("input_bindings/linux/platformer.toml"),
        GameInputProfile::Fighting => Some("input_bindings/linux/fighting.toml"),
        GameInputProfile::FlightSim => Some("input_bindings/linux/flightsim.toml"),
        GameInputProfile::Standard => None,
        GameInputProfile::Custom => None,
    }
}
```

### Asset Management Features:
1. **Platform-Specific Paths**: Different bindings for Windows/macOS/Linux
2. **Embedded Assets**: Paths reference embedded TOML files
3. **Optional Loading**: None = no additional bindings to load
4. **Standard Special Case**: None means "use default input bindings"
5. **Genre-Specific Files**: Optimized binding sets per game genre

## Example TOML Configurations

### FPS Profile (fps.toml)
```toml
[movement]
forward = "W"
backward = "S"
strafe_left = "A"
strafe_right = "D"
jump = "Space"
crouch = "LeftCtrl"
sprint = "LeftShift"
walk = "LeftAlt"

[combat]
primary_fire = "MouseLeft"
secondary_fire = "MouseRight"
reload = "R"
melee = "F"
grenade = "G"
use_item = "E"

[ui]
inventory = "Tab"
map = "M"
scoreboard = "Tab"
chat = "Enter"
voice_chat = "V"
```

### RTS Profile (rts.toml)
```toml
[camera]
pan_up = "W"
pan_down = "S"
pan_left = "A"
pan_right = "D"
zoom_in = "MouseWheelUp"
zoom_out = "MouseWheelDown"

[selection]
select = "MouseLeft"
multi_select = "LeftShift+MouseLeft"
box_select = "MouseLeftDrag"
select_all = "Ctrl+A"

[commands]
attack_move = "MouseRight"
stop = "S"
hold_position = "H"
patrol = "P"
```

## Utility Methods
```rust
pub fn names() -> impl Iterator<Item = &'static str> {
    Self::OPTIONS.iter().map(|(name, _)| *name)
}

pub fn from_names(option: &str) -> GameInputProfile {
    Self::OPTIONS
        .iter()
        .copied()
        .find_map(|(name, value)| (name == option).then_some(value))
        .unwrap_or_default()
}

pub fn gamepad_compatible(&self) -> bool {
    matches!(self, 
        GameInputProfile::Racing | 
        GameInputProfile::Platformer | 
        GameInputProfile::Fighting |
        GameInputProfile::Standard
    )
}
```

### Utility Features:
1. **names()**: Iterator over available option names for UI
2. **from_names()**: String → Enum conversion with fallback
3. **gamepad_compatible()**: Indicates if profile works well with gamepads
4. **Platform-Aware**: Uses the correct OPTIONS constant
5. **Graceful Degradation**: unwrap_or_default() for unknown names

## Settings Implementation
```rust
impl Settings for GameInputProfile {
    const KEY: Option<&'static str> = Some("input_profile");
    
    type FileContent = Option<Self>;
    
    fn load(sources: SettingsSources<Self::FileContent>, _: &mut App) -> anyhow::Result<Self> {
        if let Some(Some(user_value)) = sources.user.copied() {
            return Ok(user_value);
        }
        if let Some(Some(server_value)) = sources.server.copied() {
            return Ok(server_value);
        }
        sources.default.ok_or_else(Self::missing_default)
    }
    
    fn import_from_legacy(_legacy_settings: &Value, current: &mut Self::FileContent) {
        // Could implement migration from JSON-based settings
        *current = Some(GameInputProfile::Standard);
    }
}
```

### Settings Integration:
1. **TOML Key**: "input_profile" in game_settings.toml
2. **Optional Type**: FileContent = Option<Self> for optional configuration
3. **Priority Order**: User > Server > Default
4. **Legacy Import**: Migration support for older settings format
5. **Error Handling**: Graceful with missing_default() fallback

## Bevy System Integration
```rust
pub fn setup_input_profile_system(app: &mut App) {
    app.add_systems(
        Update,
        (
            apply_input_profile_changes,
            handle_profile_switching,
        ).run_if(resource_changed::<GameInputProfile>()),
    );
}

fn apply_input_profile_changes(
    profile: Res<GameInputProfile>,
    mut input_map: ResMut<InputMap>,
) {
    if profile.is_changed() {
        input_map.reload_from_profile(*profile);
    }
}
```

## Game-Specific Features

### 1. **Genre-Optimized Profiles**
- FPS: WASD movement, mouse look, quick weapon switching
- RTS: Camera panning, selection controls, command hotkeys
- MMORPG: Action bar bindings, UI navigation, chat commands
- Racing: Throttle/brake controls, steering, gear shifting

### 2. **Gamepad Support Detection**
- Some profiles work better with gamepads than others
- Racing and Platformer profiles are gamepad-optimized
- FPS and RTS profiles are primarily keyboard+mouse focused

### 3. **Hot-Swapping Support**
- Can change input profiles during gameplay
- Bevy systems react to profile changes automatically
- Smooth transition between different control schemes

### 4. **Server Override Capability**
- Multiplayer servers can enforce specific input profiles
- Useful for competitive gaming with standardized controls
- Player can still customize within the profile constraints

## Architectural Features for Game

### 1. **Genre-Aware Design**
- Profiles tailored to specific game genres
- Optimized binding layouts for different play styles
- Gamepad compatibility considerations

### 2. **TOML Configuration**
- Human-readable binding configuration
- Comments supported for documentation
- Hierarchical organization of input categories

### 3. **Platform Abstraction**
- OS-specific default bindings
- Cross-platform compatibility
- Automatic selection of appropriate defaults

### 4. **Bevy Integration**
- Direct ECS resource integration
- Reactive systems for profile changes
- Hot-reloading support during gameplay

### 5. **Multiplayer Considerations**
- Server-enforced profile support
- Client preference priority system
- Competitive gaming standardization