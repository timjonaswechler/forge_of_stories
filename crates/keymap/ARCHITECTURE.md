# Keymap Architecture

## Overview

The keymap system provides a **decentralized, user-customizable key binding system** for Forge of Stories. It follows these core principles:

1. **Simple API**: Only the keystroke is changeable, not the action itself
2. **Central Store**: A single `KeymapStore` (Bevy Resource) is the source of truth
3. **Decentralized Definition**: Each module defines its own default keybindings
4. **Easy Integration**: Seamless conversion to `bevy_enhanced_input` bindings

## Architecture Components

### 1. Core Data Structures

#### `Keystroke` (`keystroke.rs`)
Represents a single key press with optional modifiers.

```rust
pub struct Keystroke {
    pub modifiers: Modifiers,
    pub key: String,
}
```

**Examples:**
- `"space"` → Space key, no modifiers
- `"cmd-s"` → S key with Command modifier
- `"ctrl-shift-p"` → P key with Control + Shift modifiers

#### `ActionBinding` (`spec.rs`)
Simple struct for decentralized keymap registration.

```rust
pub struct ActionBinding {
    pub action_id: &'static str,
    pub default_keystroke: &'static str,
}
```

**Usage:**
```rust
const PLAYER_BINDINGS: &[ActionBinding] = &[
    ActionBinding {
        action_id: "player.jump",
        default_keystroke: "space",
    },
];
```

### 2. Central Store (`store.rs`)

#### `KeymapStore`
A Bevy `Resource` that manages:
- **Default bindings**: Registered by plugins
- **User overrides**: Loaded from disk (JSON)
- **Config persistence**: Auto-save to `keymap.json`

**Key Methods:**
```rust
// Register default bindings (called by plugins)
pub fn register_defaults(&mut self, bindings: &[ActionBinding])

// Get the effective binding (user override or default)
pub fn get_binding(&self, action_id: &str) -> Option<Keystroke>

// Set a user override
pub fn set_user_override(&mut self, action_id: String, new_key: Keystroke)

// Persistence
pub fn load_user_overrides(&mut self) -> Result<()>
pub fn save_user_overrides(&mut self) -> Result<()>
```

### 3. Bevy Integration (`plugin.rs`)

#### `KeymapPlugin`
Initializes the keymap system in your Bevy app.

```rust
App::new()
    .add_plugins(KeymapPlugin::default())
    .add_plugins(YourGamePlugins)
    .run();
```

**What it does:**
- Inserts `KeymapStore` as a Resource
- Loads user overrides at startup
- Auto-saves changes in `PostUpdate`

### 4. Enhanced Input Integration (`enhanced.rs`)

Converts keymap data to `bevy_enhanced_input` bindings.

```rust
// Convert a keystroke to a binding
let binding = keystroke_to_keyboard(&keystroke)?;

// Use with bevy_enhanced_input
commands.spawn((
    Action::<Jump>::new(),
    Bindings::new([binding]),
));
```

## Workflow

### For Plugin Authors (Decentralized Registration)

1. **Define your action bindings** as a static slice:

```rust
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
```

2. **Register them in your plugin's `build` method**:

```rust
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // Register defaults with the central store
        let mut store = app.world_mut().resource_mut::<KeymapStore>();
        store.register_defaults(PLAYER_BINDINGS);
        
        // Add your systems
        app.add_systems(Startup, setup_player_actions);
    }
}
```

3. **Use the bindings at runtime**:

```rust
fn setup_player_actions(
    mut commands: Commands,
    keymap: Res<KeymapStore>
) {
    // Get the final keystroke (user override or default)
    let jump_key = keymap.get_binding("player.jump").unwrap();
    
    // Convert to bevy_enhanced_input binding
    let binding = keymap::enhanced::keystroke_to_keyboard(&jump_key).unwrap();
    
    // Spawn the action
    commands.spawn((
        Action::<Jump>::new(),
        Bindings::new([binding]),
    ));
}
```

### For Users (Customization)

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

Changes are automatically loaded at startup and saved when modified.

## Directory Structure

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
│   └── keymap.rs        # Advanced matching engine (optional)
├── examples/
│   └── decentralized_registration.rs  # Full example
├── Cargo.toml
└── ARCHITECTURE.md      # This file
```

## Design Decisions

### Why Decentralized Registration?

**Problem**: In a modular game, each system (player, UI, camera, etc.) should own its input definitions, but we need a central source of truth.

**Solution**: Each module registers its defaults with the `KeymapStore` during plugin initialization. The store merges defaults with user overrides at runtime.

**Benefits**:
- Clean separation of concerns
- No central "god file" with all keybindings
- Easy to add/remove game modules
- User overrides work transparently

### Why Separate from `bevy_enhanced_input`?

The keymap system handles **data** (parsing, storage, persistence), while `bevy_enhanced_input` handles **runtime input processing**. This separation allows:

1. **Platform independence**: Keymap data is serializable and can be edited outside the game
2. **Hot-reloading**: Changes to keybindings don't require recompiling actions
3. **UI integration**: Easy to build in-game rebinding interfaces
4. **Testing**: Keymap logic can be tested without input simulation

## Next Steps

### Immediate Tasks
✅ Core architecture implemented
✅ Decentralized registration working
✅ Persistence (load/save JSON)
✅ bevy_enhanced_input integration
✅ Example code

### Future Enhancements
- [ ] In-game rebinding UI
- [ ] Conflict detection (warn if multiple actions use same key)
- [ ] Context-aware bindings (e.g., different keys in menu vs gameplay)
- [ ] Profile support (multiple keybinding sets)
- [ ] Gamepad/mouse binding support in `KeymapStore`
- [ ] Migration system for config version changes

## Example Usage

See `examples/decentralized_registration.rs` for a complete working example showing:
- Multiple plugins registering their own bindings
- Runtime retrieval of effective keybindings
- Integration with `bevy_enhanced_input`

Run with:
```bash
cargo run --example decentralized_registration
```

## API Stability

**Current Status**: 🚧 Experimental

The core API (`ActionBinding`, `KeymapStore`, `KeymapPlugin`) is stable and ready for use. Advanced features (multi-key sequences, contexts) are still being designed.