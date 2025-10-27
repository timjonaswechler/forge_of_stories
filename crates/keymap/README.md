# Keymap

A **decentralized, user-customizable key binding system** for Forge of Stories, designed to integrate seamlessly with [`bevy_enhanced_input`](https://crates.io/crates/bevy_enhanced_input).

## Core Principles

1. **Simple API**: Only the keystroke is changeable, not the action itself
2. **Central Store**: A single `KeymapStore` (Bevy Resource) is the source of truth
3. **Decentralized Definition**: Each module defines its own default keybindings
4. **Easy Integration**: Seamless conversion to `bevy_enhanced_input` bindings

## Quick Start

### 1. Add the Plugin

```rust
use bevy::prelude::*;
use keymap::KeymapPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(KeymapPlugin::default())
        .add_plugins(YourGamePlugins)
        .run();
}
```

### 2. Define Your Keybindings

Each module defines its own default keybindings:

```rust
use bevy::prelude::*;
use keymap::{ActionBinding, KeymapStore};

struct PlayerPlugin;

const PLAYER_BINDINGS: &[ActionBinding] = &[
    ActionBinding {
        action_id: "player.jump",
        default_keystroke: "space",
    },
    ActionBinding {
        action_id: "player.sprint",
        default_keystroke: "shift",
    },
];

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // Register defaults with the central store
        let mut store = app.world_mut().resource_mut::<KeymapStore>();
        store.register_defaults(PLAYER_BINDINGS);
        
        app.add_systems(Startup, setup_player_actions);
    }
}
```

### 3. Use at Runtime

```rust
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use keymap::{KeymapStore, enhanced};

fn setup_player_actions(
    mut commands: Commands,
    keymap: Res<KeymapStore>
) {
    // Get the final keystroke (user override or default)
    let jump_key = keymap.get_binding("player.jump").unwrap();
    
    // Convert to bevy_enhanced_input binding
    let binding = enhanced::keystroke_to_keyboard(&jump_key).unwrap();
    
    // Spawn the action
    commands.spawn((
        Action::<Jump>::new(),
        Bindings::new([binding]),
    ));
}

#[derive(InputAction)]
#[action_output(bool)]
struct Jump;
```

## User Customization

Users can customize keybindings by editing `keymap.json`:

```json
{
  "user_overrides": {
    "player.jump": "j",
    "player.sprint": "ctrl-shift",
    "ui.toggle_menu": "escape"
  }
}
```

Changes are automatically:
- Loaded at startup
- Saved when modified via `set_user_override`

## Keystroke Syntax

Keystrokes are parsed from simple strings:

| Example         | Description                    |
|-----------------|--------------------------------|
| `"s"`           | S key, no modifiers            |
| `"space"`       | Space bar                      |
| `"cmd-s"`       | S with Command/Super modifier  |
| `"ctrl-shift-p"`| P with Control + Shift         |
| `"f1"`          | Function key F1                |

### Supported Modifiers

- `ctrl` / `control` - Control key
- `alt` / `option` - Alt/Option key
- `shift` - Shift key
- `cmd` / `command` / `super` - Command (macOS) / Super (Linux) / Windows key

## API Overview

### `ActionBinding`

Simple struct for defining action-to-keystroke mappings:

```rust
pub struct ActionBinding {
    pub action_id: &'static str,
    pub default_keystroke: &'static str,
}
```

### `KeymapStore`

Central Bevy Resource managing all keybindings:

```rust
// Register default bindings (called by plugins)
store.register_defaults(&[
    ActionBinding {
        action_id: "action.name",
        default_keystroke: "key",
    },
]);

// Get effective binding (user override or default)
let keystroke = store.get_binding("action.name");

// Set user override
store.set_user_override("action.name".to_string(), new_keystroke);

// Persistence
store.load_user_overrides()?;
store.save_user_overrides()?;
```

### `enhanced` Module

Convert keymap data to `bevy_enhanced_input` bindings:

```rust
use keymap::enhanced;

// Single keystroke to binding
let binding = enhanced::keystroke_to_keyboard(&keystroke)?;

// Descriptor to binding (supports mouse, gamepad)
let binding = enhanced::binding_descriptor_to_binding(&descriptor)?;
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      KeymapPlugin                           │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              KeymapStore (Resource)                   │  │
│  │  ┌─────────────────┐   ┌──────────────────────────┐  │  │
│  │  │ Default Bindings│   │   User Overrides         │  │  │
│  │  │  (from plugins) │   │ (loaded from keymap.json)│  │  │
│  │  └─────────────────┘   └──────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                           │
                           │ get_binding()
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    Game Modules                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │PlayerPlugin │  │  UiPlugin   │  │   CameraPlugin      │ │
│  │             │  │             │  │                     │ │
│  │ Registers:  │  │ Registers:  │  │ Registers:          │ │
│  │ player.*    │  │ ui.*        │  │ camera.*            │ │
│  └─────────────┘  └─────────────┘  └─────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                           │
                           │ enhanced::keystroke_to_keyboard()
                           ▼
┌─────────────────────────────────────────────────────────────┐
│               bevy_enhanced_input                           │
│         (Runtime Input Processing)                          │
└─────────────────────────────────────────────────────────────┘
```

## Features

- ✅ **Decentralized registration**: Each module owns its bindings
- ✅ **User customization**: JSON-based override system
- ✅ **Auto-persistence**: Changes saved automatically
- ✅ **Type-safe**: Works with `bevy_enhanced_input` actions
- ✅ **Simple API**: Minimal boilerplate
- ✅ **Well-tested**: Comprehensive test coverage

## Examples

See `examples/decentralized_registration.rs` for a complete working example:

```bash
cargo run --example decentralized_registration
```

## Module Structure

```
crates/keymap/
├── src/
│   ├── lib.rs           # Public API and re-exports
│   ├── binding.rs       # KeyBinding, ActionId, precedence
│   ├── keystroke.rs     # Keystroke parsing and matching
│   ├── spec.rs          # ActionBinding definition
│   ├── store.rs         # KeymapStore (central state)
│   ├── plugin.rs        # Bevy plugin integration
│   ├── enhanced.rs      # bevy_enhanced_input conversion
│   └── keymap.rs        # Advanced matching engine
├── examples/
│   └── decentralized_registration.rs
├── ARCHITECTURE.md      # Detailed architecture docs
├── Cargo.toml
└── README.md            # This file
```

## Testing

Run the test suite:

```bash
cargo test -p keymap
```

All tests include:
- Keystroke parsing
- Binding precedence
- Store operations
- Enhanced input conversion

## Future Enhancements

- [ ] In-game rebinding UI
- [ ] Conflict detection
- [ ] Context-aware bindings (menu vs gameplay)
- [ ] Multiple keybinding profiles
- [ ] Multi-keystroke sequences (chords)
- [ ] Migration system for config changes

## License

Part of Forge of Stories project.