# Keymap

A Zed-inspired keymap system for context-aware key binding dispatch in Rust.

## Features

- **Context-based binding resolution**: Key bindings can be scoped to specific UI contexts
- **Multi-keystroke sequences**: Support for chord bindings like `cmd-k cmd-t`
- **Hierarchical context matching**: Predicates with `&&`, `||`, `!`, `>` operators
- **Action dispatch system**: Type-safe action handling with custom data
- **User/Default precedence**: User bindings override default bindings
- **Platform-aware**: Automatic OS detection in contexts

## Core Concepts

### Actions

Actions represent commands that can be triggered by key bindings.

```rust
use keymap::{action, actions, action_with_data};

// Simple action
action!(SaveFile);

// Multiple actions at once
actions![
    OpenFile,
    CloseFile,
    SaveAll,
];

// Action with custom data
action_with_data!(GoToLine {
    line: usize
});
```

### Keystrokes

Parse and match keyboard input.

```rust
use keymap::Keystroke;

// Single keystroke
let ks = Keystroke::parse("cmd-s").unwrap();

// Multi-keystroke sequence
let seq = keymap::parse_keystroke_sequence("cmd-k cmd-t").unwrap();
```

Supported modifiers:
- `ctrl` or `control`
- `alt` or `option`
- `shift`
- `cmd`, `command`, or `super`

### Contexts

Contexts represent the current state of the UI and are arranged in a stack.

```rust
use keymap::KeyContext;

let mut context = KeyContext::new_with_defaults(); // Includes OS
context.add("Editor");
context.set("mode", "full");
context.set("language", "rust");

println!("{:?}", context); // Editor mode=full language=rust os=macos
```

### Context Predicates

Define when bindings should be active using a simple DSL.

```rust
use keymap::KeyBindingContextPredicate;

// Simple identifier
let pred = KeyBindingContextPredicate::parse("Editor").unwrap();

// Key-value equality
let pred = KeyBindingContextPredicate::parse("mode == full").unwrap();

// Logical operators
let pred = KeyBindingContextPredicate::parse(
    "Editor && mode == full && !readonly"
).unwrap();

// Hierarchical (child context)
let pred = KeyBindingContextPredicate::parse(
    "Workspace > Editor"
).unwrap();
```

### Default Actions

The keymap system provides a comprehensive set of default actions organized by category:

```rust
use keymap::actions::*;

// File operations
file::Save, file::SaveAs, file::Open, file::Close, file::Quit

// Edit operations
edit::Undo, edit::Redo, edit::Cut, edit::Copy, edit::Paste

// Navigation
navigation::NextTab, navigation::PreviousTab, navigation::FocusEditor

// View
view::ZoomIn, view::ZoomOut, view::ToggleFullscreen

// Search
search::Find, search::Replace, search::FindInFiles

// Debug
debug::ToggleInspector, debug::ReloadKeymap

// Game-specific
game::Pause, game::QuickSave, game::OpenMenu, game::Screenshot
```

### Default Keybindings

Platform-appropriate default keybindings are available:

```rust
use keymap::defaults;

// Automatically select platform defaults (macOS/Windows/Linux)
let builder = KeymapStore::builder();
let builder = defaults::register_platform_defaults(builder);

// Or explicitly choose:
let builder = defaults::register_macos_defaults(builder);
let builder = defaults::register_windows_linux_defaults(builder);
```

### Key Bindings

Connect keystrokes to actions with optional context predicates.

```rust
use keymap::{KeyBinding, KeyBindingMetaIndex, Keystroke};
use std::sync::Arc;

action!(SaveFile);

let keystrokes = vec![Keystroke::parse("cmd-s").unwrap()];
let action = Box::new(SaveFile);
let predicate = Some(Arc::new(
    KeyBindingContextPredicate::parse("Editor").unwrap()
));

let binding = KeyBinding::new(keystrokes, action, predicate)
    .with_meta(KeyBindingMetaIndex::USER);
```

## Precedence Rules

Key bindings are resolved with the following precedence:

1. **Context Depth**: Deeper contexts take precedence (Editor > Workspace)
2. **Source Priority**: USER > VIM > BASE > DEFAULT
3. **Order**: Later bindings override earlier ones at the same depth

## Architecture

```
keymap/
├── action.rs       # Action trait & macros
├── binding.rs      # KeyBinding struct
├── context.rs      # KeyContext & Predicate
├── keystroke.rs    # Keystroke parsing
├── keymap.rs       # Keymap matching logic
├── store.rs        # Persistence & storage
├── bevy.rs         # Bevy integration (optional)
└── lib.rs          # Public API
```

## Example: Complete Workflow

```rust
use keymap::{
    KeyBinding, KeyContext, Keystroke, 
    KeyBindingContextPredicate, action
};
use std::sync::Arc;

action!(SaveFile);
action!(OpenFile);

// Create bindings
let save_binding = KeyBinding::new(
    vec![Keystroke::parse("cmd-s").unwrap()],
    Box::new(SaveFile),
    Some(Arc::new(KeyBindingContextPredicate::parse("Editor").unwrap()))
);

let open_binding = KeyBinding::new(
    vec![Keystroke::parse("cmd-o").unwrap()],
    Box::new(OpenFile),
    None // Global binding
);

// Create context stack
let workspace_ctx = KeyContext::parse("Workspace").unwrap();
let editor_ctx = KeyContext::parse("Editor mode=full").unwrap();
let context_stack = vec![workspace_ctx, editor_ctx];

// Match keystrokes
let typed = vec![Keystroke::parse("cmd-s").unwrap()];
if save_binding.match_keystrokes(&typed) == Some(false) {
    // Check context
    if let Some(pred) = save_binding.predicate() {
        if pred.eval(&context_stack) {
            println!("Execute SaveFile action!");
        }
    }
}
```

## Bevy Integration

The keymap system can be integrated with Bevy using the optional `bevy_plugin` feature.

### Setup

Add to your `Cargo.toml`:

```toml
keymap = { path = "crates/keymap", features = ["bevy_plugin"] }
```

### Basic Usage

```rust
use bevy::prelude::*;
use keymap::bevy::{KeymapPlugin, ContextStack, ActionEvent};
use keymap::actions::file;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(KeymapPlugin::default())
        .add_systems(Update, handle_actions)
        .run();
}

fn handle_actions(mut events: EventReader<ActionEvent>) {
    for event in events.read() {
        // Check action type and handle it
        if event.action.partial_eq(&file::Save) {
            println!("Saving file...");
            // Perform save operation
        } else if event.action.partial_eq(&file::Open) {
            println!("Opening file...");
        }
    }
}
```

### Action Dispatcher

The plugin automatically dispatches actions as Bevy events when key bindings are triggered:

```rust
// The ActionEvent contains the triggered action
#[derive(Event, Clone)]
pub struct ActionEvent {
    pub action: Arc<dyn Action>,
}

// Listen for actions in any system
fn my_system(mut events: EventReader<ActionEvent>) {
    for event in events.read() {
        // Handle the action
        println!("Action: {}", event.action.debug_name());
    }
}
```

### Default Actions & Keybindings

The plugin automatically registers all default actions and can load platform defaults:

```rust
use keymap::bevy::KeymapPlugin;
use keymap::defaults;

fn main() {
    App::new()
        .add_plugins(KeymapPlugin::default()) // Includes all default actions
        .run();
}

// Actions are automatically available:
// - file::Save, file::Open, etc.
// - edit::Cut, edit::Copy, edit::Paste, etc.
// - All categories from keymap::actions
```

### Managing Context

```rust
fn update_context(mut context_stack: ResMut<ContextStack>) {
    // Update context based on UI state
    context_stack.clear();
    let mut ctx = keymap::KeyContext::new_with_defaults();
    ctx.add("Editor");
    context_stack.push(ctx);
}
```

### KeyCode → Keystroke Interpreter

The Bevy plugin automatically converts Bevy's `KeyCode` events to our `Keystroke` format:

- Letter keys: `KeyA` → `"a"`
- Modifiers: Automatically detected from `ButtonInput<KeyCode>`
- Function keys: `F1` → `"f1"`
- Special keys: `Escape` → `"escape"`, `Space` → `"space"`, etc.

### Context Stack

The `ContextStack` resource manages hierarchical contexts:

```rust
fn my_system(mut stack: ResMut<ContextStack>) {
    // Push context when entering UI element
    let mut ctx = KeyContext::default();
    ctx.add("MyPanel");
    stack.push(ctx);
    
    // Pop when leaving
    stack.pop();
    
    // Clear all
    stack.clear();
}
```

## Testing

```bash
# Test without Bevy
cargo test -p keymap

# Test with Bevy integration
cargo test -p keymap --features bevy_plugin
```

## Completed Features

- [x] Core Keymap infrastructure (Action, Binding, Context, Predicate)
- [x] Keystroke parsing system
- [x] Keymap matching logic with precedence rules
- [x] KeymapStore with persistence
- [x] JSON serialization/deserialization
- [x] Default + User merge logic
- [x] NoAction support for disabling bindings
- [x] Bevy Plugin integration (optional)
- [x] KeyCode → Keystroke interpreter
- [x] Context stack for hierarchical matching
- [x] Action Dispatcher (Bevy Event System)
- [x] Default Actions (file, edit, navigation, view, search, debug, game)
- [x] Default Keybindings (macOS/Windows/Linux)

## License

See workspace license.