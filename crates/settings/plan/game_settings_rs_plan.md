# Plan: crates/settings/src/settings.rs (Game Version)

## Overview
This file will be the main module of the `settings` crate for the game, serving as the central entry point for the entire game settings system using TOML configuration.

## Module Structure
```rust
mod graphics_settings;
mod audio_settings;
mod gameplay_settings;
mod network_settings;
mod input_bindings;
mod settings_file;
mod settings_toml;
mod settings_store;
```
- **Purpose**: Defines all submodules of the game settings crate
- **Game-Specific**: Each module handles a specific aspect of game configuration

## Imports
```rust
use bevy::prelude::*;
use rust_embed::RustEmbed;
use std::{borrow::Cow, fmt, str};
```
- **bevy**: Game engine framework for app context and resource management
- **rust_embed**: Enables embedding files at compile-time
- **std types**: For efficient string handling and formatting

## Public Re-Exports
- **Function**: Makes all important types and functions from submodules publicly available
- **Game-Specific Exports**:
  - `GameSettings`, `SettingsStore` - Core of the settings system
  - `InputBindings`, `InputValidator` - Input binding management
  - `NetworkConfig` - Multiplayer configuration
  - `GraphicsConfig`, `AudioConfig` - Rendering and sound settings

## ActiveGameProfile
```rust
#[derive(Resource)]
pub struct ActiveGameProfile(pub String);
```
- **Purpose**: Wrapper for the name of the active game profile
- **Bevy Integration**: Implements `Resource` trait - stored as a Bevy resource
- **Pattern**: Newtype Pattern for type safety

## ServerId
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServerId(usize);
```
- **Purpose**: Unique identification of game servers
- **Conversions**: 
  - `from_usize/to_usize`: Internal conversion
  - `from_network/to_network`: Network protocol serialization (u64)
- **Traits**: `Display`, `From`, plus standard derive traits
- **Feature**: Type-safe wrapper around usize with network compatibility

## GameSettingsAssets
```rust
#[derive(RustEmbed)]
#[folder = "../../assets"]
#[include = "game_settings/*.toml"]
#[include = "input_bindings/*.toml"]
#[exclude = "*.DS_Store"]
#[exclude = "*.tmp"]
pub struct GameSettingsAssets;
```
- **Purpose**: Embedding all game settings and input binding assets at compile-time
- **Path**: `../../assets` - relative path to asset folder
- **Filter**: Only game_settings/ and input_bindings/ folders, TOML files only
- **Advantage**: Assets are directly available in binary, no runtime file access needed

## Initialization Function
```rust
pub fn init_game_settings(app: &mut App) {
    let mut settings = SettingsStore::new();
    settings.set_default_settings(&default_game_settings()).unwrap();
    app.insert_resource(settings);
    app.insert_resource(ActiveGameProfile("default".to_string()));
    register_input_bindings(app);
    setup_settings_observers(app);
}
```
- **Purpose**: Bootstrapping the entire game settings system
- **Steps**:
  1. Create SettingsStore
  2. Load default game settings
  3. Insert as Bevy resource
  4. Set default profile
  5. Register input bindings
  6. Setup observers for settings changes

## Asset Access Functions
- **default_game_settings()**: Loads `game_settings/default.toml`
- **default_input_bindings()**: Loads platform-specific input bindings
  - Windows: `input_bindings/default-windows.toml`
  - macOS: `input_bindings/default-macos.toml`
  - Linux: `input_bindings/default-linux.toml`
- **gamepad_bindings()**: Loads `input_bindings/gamepad.toml`
- **initial_*_content()**: Template files for new configurations

## Platform-Specific Constants
```rust
#[cfg(target_os = "windows")]
pub const DEFAULT_INPUT_PATH: &str = "input_bindings/default-windows.toml";

#[cfg(target_os = "macos")]
pub const DEFAULT_INPUT_PATH: &str = "input_bindings/default-macos.toml";

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub const DEFAULT_INPUT_PATH: &str = "input_bindings/default-linux.toml";
```
- **Purpose**: Compile-time selection of correct input bindings based on operating system
- **Feature**: Conditional compilation for platform-specific defaults

## Game-Specific Configuration Categories

### Graphics Settings
```toml
# game_settings/graphics.toml
[display]
resolution = "1920x1080"
fullscreen = false
vsync = true
fps_limit = 144

[quality]
texture_quality = "high"
shadow_quality = "medium"
anti_aliasing = "MSAA4x"
```

### Audio Settings
```toml
# game_settings/audio.toml
[volume]
master = 1.0
music = 0.8
sfx = 0.9
voice = 1.0

[quality]
sample_rate = 48000
bit_depth = 16
```

### Network Settings
```toml
# game_settings/network.toml
[server]
default_port = 7777
max_players = 32
tick_rate = 60

[client]
connection_timeout = 30
retry_attempts = 3
```

### Input Bindings
```toml
# input_bindings/default.toml
[movement]
forward = "W"
backward = "S"
left = "A" 
right = "D"
jump = "Space"

[combat]
primary_attack = "MouseLeft"
secondary_attack = "MouseRight"
reload = "R"
```

## Architectural Features for Game

### 1. **Bevy Integration**
- Functions as facade for the entire game settings system
- Direct integration with Bevy's resource system
- Automatic asset embedding for game configurations

### 2. **TOML Configuration**
- Human-readable configuration format
- Better for game settings than JSON
- Supports comments for documentation

### 3. **Platform Abstraction**
- Automatic selection of correct defaults per platform
- Platform-specific input handling
- Cross-platform compatibility

### 4. **Game-Specific Types**
- ServerId for multiplayer server identification
- ActiveGameProfile for settings profiles (single-player, competitive, etc.)
- Type safety for game-specific identifiers

### 5. **Modular Settings Categories**
- Separate modules for graphics, audio, gameplay, network
- Clean separation of concerns
- Easy to extend with new setting categories

### 6. **Server/Client Architecture Support**
- Network settings for dedicated servers
- Client-specific configurations
- Profile system supporting both local and multiplayer modes