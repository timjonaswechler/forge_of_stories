# Game Settings Architecture Plan
# Adapted from Zed Editor Settings System

## Überblick
Diese Datei definiert die Architektur für das Settings-System des Forge of Stories Spiels. Es basiert auf Zeds ausgereiftem Settings-System, aber angepasst für Game-spezifische Anforderungen mit TOML-Konfiguration und Bevy-Integration.

## Modulstruktur
```rust
mod graphics_settings;      // Grafikeinstellungen (Resolution, Quality, etc.)
mod audio_settings;         // Audio-Konfiguration (Volume, Device-Settings)
mod gameplay_settings;      // Gameplay-Optionen (Difficulty, Accessibility)
mod network_settings;       // QUIC, Encryption, Connection-Settings
mod server_settings;        // Dedicated Server Konfiguration
mod keymap_settings;        // Input-Mappings für Bevy Input-System
mod settings_store;         // Zentraler Store mit TOML-Unterstützung
mod settings_file;          // File-Watching und TOML-Parsing
mod editable_setting_control; // UI-Controls für TUI Settings-Editor
```

## Core Dependencies
```rust
use bevy::prelude::*;                    // Bevy Engine Integration
use serde::{Deserialize, Serialize};     // TOML Serialization
use toml_edit::{Document, value};        // Format-preserving TOML editing
use tokio::fs;                           // Async file operations
use notify::Watcher;                     // File watching für live-reload
use anyhow::{Context, Result};           // Error handling
use std::{path::PathBuf, sync::Arc};     // Standard library
```

## Settings Trait (Core Interface)
```rust
pub trait Setting: 'static + Send + Sync + Clone {
    /// TOML key für diese Setting-Kategorie
    const TOML_KEY: &'static str;

    /// Fallback key für Rückwärtskompatibilität
    const FALLBACK_KEY: Option<&'static str> = None;

    /// TOML-serializable Struktur für Config-Files
    type TomlContent: Clone + Default + Serialize + for<'de> Deserialize<'de>;

    /// Lädt Setting aus hierarchischen TOML-Quellen
    fn load_from_sources(sources: SettingSources<Self::TomlContent>) -> Result<Self>;

    /// Validiert Setting-Werte (z.B. Resolution-Grenzen)
    fn validate(&self) -> Result<()> { Ok(()) }

    /// Wendet Setting im Bevy-System an (z.B. Window-Resize)
    fn apply_to_bevy(&self, world: &mut World) {}
}
```

## Settings-Hierarchie für Games
```rust
pub struct SettingSources<'a, T> {
    pub default: &'a T,              // Embedded defaults
    pub global: Option<&'a T>,       // ~/.config/forge_of_stories/
    pub user: Option<&'a T>,         // User-specific overrides
    pub server: Option<&'a T>,       // Server-provided settings
    pub world: Option<&'a T>,        // World/Save-specific settings
    pub profile: Option<&'a T>,      // Settings-Profile (Single/Multiplayer)
}
```

### Prioritäts-Hierarchie (niedrig → hoch):
1. **Default**: Eingebrannte Standard-Werte
2. **Global**: System-weite Game-Config
3. **User**: User-spezifische Einstellungen
4. **Server**: Server-übermittelte Settings (Multiplayer)
5. **World**: Welt-spezifische Overrides
6. **Profile**: Temporäre Profile (Competitive vs Casual)

## Game Settings Kategorien

### 1. Graphics Settings
```toml
[graphics]
resolution = "1920x1080"
fullscreen = true
quality_preset = "High"
vsync = true
max_fps = 144
render_scale = 1.0
shadow_quality = "Medium"
texture_quality = "High"
anti_aliasing = "FXAA"
```

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphicsSettings {
    pub resolution: (u32, u32),
    pub fullscreen: bool,
    pub quality_preset: QualityPreset,
    pub vsync: bool,
    pub max_fps: Option<u32>,
    pub render_scale: f32,
    pub shadow_quality: QualityLevel,
    pub texture_quality: QualityLevel,
    pub anti_aliasing: AntiAliasingMode,
}

impl Setting for GraphicsSettings {
    const TOML_KEY: &'static str = "graphics";
    type TomlContent = GraphicsTomlContent;

    fn apply_to_bevy(&self, world: &mut World) {
        // Resize window, update render pipeline settings, etc.
        if let Some(mut window) = world.get_resource_mut::<Window>() {
            window.resolution.set(self.resolution.0 as f32, self.resolution.1 as f32);
            window.mode = if self.fullscreen { WindowMode::Fullscreen } else { WindowMode::Windowed };
        }
    }
}
```

### 2. Audio Settings
```toml
[audio]
master_volume = 0.8
sfx_volume = 0.9
music_volume = 0.7
voice_volume = 1.0
audio_device = "default"
spatial_audio = true
```

### 3. Keymap Settings (Bevy Input Integration)
```toml
# Gameplay Context - Main game controls
[context.gameplay]
move_forward = ["KeyW", "ArrowUp"]
move_backward = ["KeyS", "ArrowDown"]
move_left = ["KeyA", "ArrowLeft"]
move_right = ["KeyD", "ArrowRight"]
jump = ["Space"]
crouch = ["KeyC", "ControlLeft"]
sprint = ["ShiftLeft"]

primary_action = ["MouseLeft"]
secondary_action = ["MouseRight"]
interact = ["KeyE"]
open_inventory = ["Tab", "KeyI"]
open_menu = ["Escape"]

# Chat (Multiplayer)
chat_all = ["KeyT"]
chat_team = ["KeyY"]
voice_chat = ["KeyV"]

# Menu Context - UI navigation
[context.menu]
menu_up = ["ArrowUp", "KeyW"]
menu_down = ["ArrowDown", "KeyS"]
menu_left = ["ArrowLeft", "KeyA"]
menu_right = ["ArrowRight", "KeyD"]
menu_select = ["Enter", "Space"]
menu_back = ["Escape", "Backspace"]

# Inventory Context - Item management
[context.inventory]
sort_items = ["KeyR"]
drop_item = ["KeyQ"]
quick_move = ["ShiftLeft"]
split_stack = ["ControlLeft"]
search_items = ["KeyF"]
close_inventory = ["Tab", "Escape"]

# Chat Context - Text input mode
[context.chat]
send_message = ["Enter"]
cancel_chat = ["Escape"]
chat_history_up = ["ArrowUp"]
chat_history_down = ["ArrowDown"]

# Building Context - Construction mode
[context.building]
rotate_object = ["KeyR"]
confirm_placement = ["MouseLeft"]
cancel_building = ["MouseRight", "Escape"]
toggle_snap = ["KeyG"]
copy_object = ["ControlLeft"]

# Input behavior settings
[context.settings]
mouse_sensitivity = 1.0
invert_mouse_y = false
gamepad_deadzone = 0.1
```

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeymapSettings {
    pub bindings: HashMap<GameAction, Vec<InputBinding>>,
    pub mouse_sensitivity: f32,
    pub invert_mouse_y: bool,
    pub context_bindings: HashMap<InputContext, HashMap<GameAction, Vec<InputBinding>>>,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum GameAction {
    // Movement
    MoveForward, MoveBackward, MoveLeft, MoveRight,
    Jump, Crouch, Sprint,

    // Combat
    PrimaryAction, SecondaryAction, Block, Dodge,

    // Interaction
    Interact, Inventory, Menu, Map,

    // Communication
    ChatAll, ChatTeam, VoiceChat,

    // Custom action support
    Custom(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InputBinding {
    Keyboard(KeyCode),
    Mouse(MouseButton),
    MouseWheel(MouseWheelDirection),
    Gamepad(GamepadButtonType),
    // Combos möglich: Ctrl+C, Alt+F4, etc.
    Combo(Vec<InputBinding>),
}

impl Setting for KeymapSettings {
    const TOML_KEY: &'static str = "context";
    type TomlContent = KeymapTomlContent;

    fn apply_to_bevy(&self, world: &mut World) {
        // Update Bevy's Input systems with new bindings
        // Register action mappings in ActionMapResource
    }
}
```

### 4. Network Settings
```toml
[network]
# QUIC Configuration
server_port = 7777
client_timeout = 30
max_retries = 3
enable_encryption = true

# Multiplayer
max_players = 16
server_browser_timeout = 5
auto_reconnect = true

# Steam Integration
steam_relay = true
friends_only = false
```

### 5. Server Settings (Dedicated Server)
```toml
[server]
world_name = "My Forge World"
description = "Welcome to our server!"
password = ""
admin_password = "secret123"

max_players = 32
pvp_enabled = true
difficulty = "Normal"

# World Settings
world_seed = 12345
world_size = "Large"
regenerate_chunks = true

# Backup & Persistence
auto_save_interval = 300  # seconds
backup_count = 5
backup_interval = 3600    # hourly backups
```

## SettingsStore (Central Management)
```rust
pub struct SettingsStore {
    // Type-erased storage für verschiedene Settings
    setting_values: HashMap<TypeId, Box<dyn AnySetting>>,

    // TOML-Dateien aus verschiedenen Quellen
    default_toml: toml::Value,
    global_toml: Option<toml::Value>,
    user_toml: Option<toml::Value>,
    world_toml: HashMap<String, toml::Value>, // Per-World settings

    // File watchers für live-reload
    _file_watchers: Vec<notify::RecommendedWatcher>,

    // Bevy World-Referenz für Setting-Anwendung
    bevy_world: Option<Arc<Mutex<World>>>,
}

impl SettingsStore {
    pub fn new() -> Self { /* ... */ }

    pub fn register_setting<T: Setting>(&mut self) -> Result<()> { /* ... */ }

    pub fn get<T: Setting>(&self) -> &T { /* ... */ }

    pub fn update_setting<T: Setting>(&mut self, setting: T) -> Result<()> { /* ... */ }

    pub fn reload_from_files(&mut self) -> Result<()> { /* ... */ }

    pub fn apply_all_to_bevy(&self) -> Result<()> { /* ... */ }
}
```

## Bevy Integration
```rust
// Bevy Plugin für Settings-System
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SettingsStore>()
            .add_systems(PreUpdate, (
                reload_settings_system,
                apply_graphics_settings_system,
                apply_audio_settings_system,
                apply_keymap_settings_system,
            ))
            .add_systems(PostUpdate, save_settings_system);
    }
}

// System für automatisches Settings-Anwenden
fn apply_graphics_settings_system(
    settings_store: Res<SettingsStore>,
    mut windows: Query<&mut Window>,
) {
    if settings_store.is_changed() {
        let graphics = settings_store.get::<GraphicsSettings>();
        graphics.apply_to_bevy(&mut world);
    }
}
```

## File-Layout für Game Config
```
~/.config/forge_of_stories/
├── settings.toml              # Main settings file
├── keybindings.toml          # Input mappings
├── profiles/
│   ├── competitive.toml      # Competitive gaming profile
│   ├── casual.toml           # Casual gaming profile
│   └── streaming.toml        # Streaming-optimized settings
├── worlds/
│   ├── world1_settings.toml  # Per-world overrides
│   └── world2_settings.toml
└── servers/
    ├── server_config.toml    # Dedicated server settings
    └── admin_settings.toml   # Admin-only settings
```

## TUI Integration für Server-Management
```rust
// Settings-Editor für TUI (Server-Dashboard)
pub struct TuiSettingsEditor {
    store: Arc<Mutex<SettingsStore>>,
    current_category: SettingsCategory,
    modified: bool,
}

impl TuiSettingsEditor {
    pub fn render_graphics_settings(&self, frame: &mut Frame, area: Rect) { /* ... */ }
    pub fn render_network_settings(&self, frame: &mut Frame, area: Rect) { /* ... */ }
    pub fn render_keymap_editor(&self, frame: &mut Frame, area: Rect) { /* ... */ }

    pub fn handle_input(&mut self, event: KeyEvent) -> Result<()> { /* ... */ }
    pub fn save_changes(&mut self) -> Result<()> { /* ... */ }
}
```

## Error Handling & Validation
```rust
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("Invalid resolution: {0}x{1}")]
    InvalidResolution(u32, u32),

    #[error("Volume must be between 0.0 and 1.0, got {0}")]
    InvalidVolume(f32),

    #[error("Invalid key binding: {0}")]
    InvalidKeyBinding(String),

    #[error("TOML parse error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

## Architektonische Verbesserungen gegenüber Zed

### 1. **Game-Specific Optimizations**
- **Bevy Integration**: Direkte Anwendung von Settings auf Bevy-Systeme
- **Performance Profiles**: Competitive vs Casual Settings-Profile
- **Per-World Settings**: Verschiedene Settings pro Spielwelt

### 2. **TOML statt JSON**
- **Menschenlesbar**: Bessere Editierbarkeit für Spieler
- **Kommentare**: Dokumentation direkt in Config-Files
- **Hierarchisch**: Saubere Struktur für komplexe Game-Settings

### 3. **Multiplayer-Awareness**
- **Server Settings**: Separate Konfiguration für Dedicated Server
- **Client-Server Sync**: Server kann Settings an Clients übermitteln
- **Admin Controls**: Admin-only Settings für Server-Management

### 4. **Input-System Integration**
- **Bevy Input**: Native Integration mit Bevy's Input-Systemen
- **Context-Aware**: Verschiedene Bindings je nach Game-State
- **Gamepad Support**: Controller-Unterstützung zusätzlich zu Keyboard/Mouse

### 5. **TUI Dashboard Integration**
- **Live Editing**: Settings-Editor im Server-TUI
- **Real-time Preview**: Sofortige Anwendung von Änderungen
- **Validation UI**: Visuelle Fehler-Anzeige bei ungültigen Werten

Diese Architektur kombiniert Zeds bewährte Patterns mit game-spezifischen Anforderungen und bietet ein robustes, erweiterbares Settings-System für dein Bevy-basiertes Multiplayer-Game.
