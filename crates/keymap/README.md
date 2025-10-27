# Keymap

A Zed-inspired keymap store focused on *data*: parsing, merging, and
persisting bindings that can be fed into input runtimes such as
[`bevy_enhanced_input`](https://crates.io/crates/bevy_enhanced_input).

## Features

- **Context-driven resolution** – bindings can be scoped to rich predicates
- **Chord support** – parse multi-step sequences such as `cmd-k cmd-t`
- **Deterministic precedence** – depth → source → order
- **JSON persistence** – defaults in code, user overrides on disk
- **Bridging helpers** – convert keymap keystrokes into enhanced-input bindings

## Core Concepts

### Action identifiers

Bindings reference logical actions via lightweight IDs. Consumers are free to
map them to whatever runtime types they need.

```rust
use keymap::ActionId;

let save = ActionId::from("file::Save");
assert_eq!(save.as_str(), "file::Save");
```

### Keystrokes

Parse user-supplied strings into structured keystrokes.

```rust
use keymap::{Keystroke, parse_keystroke_sequence};

let single = Keystroke::parse("cmd-s").unwrap();
let seq = parse_keystroke_sequence("cmd-k cmd-t").unwrap();
```

Supported modifiers:

- `ctrl` / `control`
- `alt` / `option`
- `shift`
- `cmd` / `command` / `super`

### Contexts & predicates

Contexts capture the active UI state. Predicates decide when bindings apply.

```rust
use keymap::{KeyContext, KeyBindingContextPredicate};

let mut ctx = KeyContext::default();
ctx.add("Editor");
ctx.set("language", "rust");

let pred = KeyBindingContextPredicate::parse("Editor && language == rust").unwrap();
assert!(pred.eval(&[ctx]));
```

### Bindings

Each binding couples physical input with meta-information. Use
`BindingDescriptor` to author defaults or overrides; `None` `action_id`
disables the sequence.

```rust
use keymap::{
    ActionId, BindingDescriptor, BindingInputDescriptor, KeyBindingContextPredicate,
    KeyBindingMetaIndex, parse_keystroke_sequence,
};
use std::sync::Arc;

let predicate = Some(Arc::new(KeyBindingContextPredicate::parse("Editor").unwrap()));

let binding = BindingDescriptor {
    action_id: Some(ActionId::from("file::Save")),
    context_id: None,
    predicate: predicate.as_ref().map(|p| p.to_string()),
    meta: Some(KeyBindingMetaIndex::DEFAULT),
    modifiers: Vec::new(),
    conditions: Vec::new(),
    settings: None,
    input: Some(BindingInputDescriptor::keyboard(
        parse_keystroke_sequence("cmd-s").unwrap(),
    )),
};
```

## Persistence workflow

`KeymapStore` merges built-in defaults with user overrides stored in JSON and
exposes both the merged descriptor set and a legacy keymap for keyboard lookup.

```rust
use keymap::{
    ActionId, BindingDescriptor, BindingInputDescriptor, KeyBindingMetaIndex, KeymapStore,
    parse_keystroke_sequence,
};

let store = KeymapStore::builder()
    .with_user_keymap_path("~/.config/my_app/keymap.json")
    .add_default_binding(BindingDescriptor {
        action_id: Some(ActionId::from("file::Save")),
        context_id: None,
        predicate: None,
        meta: Some(KeyBindingMetaIndex::DEFAULT),
        modifiers: Vec::new(),
        conditions: Vec::new(),
        settings: None,
        input: Some(BindingInputDescriptor::keyboard(
            parse_keystroke_sequence("cmd-s").unwrap(),
        )),
    })
    .build()
    .unwrap();

// Load user overrides (if the file exists)
store.load_user_bindings().unwrap();

// Consume merged view
store.with_keymap(|keymap| {
    let (bindings, pending) =
        keymap.bindings_for_input(&[Keystroke::parse("cmd-s").unwrap()], &[]);
    assert!(!pending);
    assert!(!bindings.is_empty());
});
```

Serialization format mirrors the descriptor layout:

```json
{
  "schema_version": "0.1.0",
  "spec": {
    "actions": [],
    "contexts": [],
    "bindings": [
    {
      "action_id": "file::Save",
      "meta": 3,
      "input": {
        "type": "keyboard",
        "sequence": [
          { "modifiers": { "ctrl": false, "alt": false, "shift": false, "cmd": true }, "key": "s" }
        ]
      }
    }
  ]
}
```

## Enhanced input bridge

The `enhanced` module converts descriptors into
`bevy_enhanced_input` bindings. Keyboard and mouse inputs are supported out of
the box; gamepad helpers fall back to readable errors when a mapping is
unknown, making it easy to extend behaviour on demand.

```rust
use bevy_enhanced_input::binding::Binding;
use keymap::{enhanced, BindingDescriptor, BindingInputDescriptor, ActionId, parse_keystroke_sequence};

let descriptor = BindingDescriptor {
    action_id: Some(ActionId::from("file::Open")),
    context_id: None,
    predicate: None,
    meta: None,
    modifiers: Vec::new(),
    conditions: Vec::new(),
    settings: None,
    input: Some(BindingInputDescriptor::keyboard(
        parse_keystroke_sequence("ctrl-p").unwrap(),
    )),
};

let binding_component: Binding = enhanced::binding_descriptor_to_binding(&descriptor)
    .unwrap()
    .expect("descriptor maps to a binding");
```

### Full integration example

This example shows how to integrate the keymap store with `bevy_enhanced_input` to create
a type-safe, rebindable input system.

```rust
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use keymap::{
    ActionId, BindingDescriptor, BindingInputDescriptor, KeymapStore,
    KeyBindingMetaIndex, enhanced::binding_descriptor_to_binding,
    parse_keystroke_sequence,
};

// 1. Define your actions as types
#[derive(InputAction)]
#[action_output(bool)]
struct Jump;

#[derive(InputAction)]
#[action_output(Vec2)]
struct Move;

// 2. Create a keymap store with default bindings
fn create_store() -> KeymapStore {
    KeymapStore::builder()
        .with_user_keymap_path("~/.config/my_game/keymap.json")
        .add_default_binding(BindingDescriptor {
            action_id: Some(ActionId::from("player::jump")),
            context_id: None,
            predicate: None,
            meta: Some(KeyBindingMetaIndex::DEFAULT),
            modifiers: Vec::new(),
            conditions: Vec::new(),
            settings: None,
            input: Some(BindingInputDescriptor::keyboard(
                parse_keystroke_sequence("space").unwrap(),
            )),
        })
        .add_default_binding(BindingDescriptor {
            action_id: Some(ActionId::from("player::move")),
            context_id: None,
            predicate: None,
            meta: Some(KeyBindingMetaIndex::DEFAULT),
            modifiers: Vec::new(),
            conditions: Vec::new(),
            settings: None,
            input: Some(BindingInputDescriptor::keyboard(
                parse_keystroke_sequence("w").unwrap(),
            )),
        })
        .build()
        .unwrap()
}

// 3. Convert keymap bindings to enhanced input and spawn the player
fn spawn_player(mut commands: Commands, keymap_store: Res<KeymapStore>) {
    let mut jump_bindings = Vec::new();
    let mut move_bindings = Vec::new();

    // Extract bindings from the keymap store
    keymap_store.with_spec(|spec| {
        for descriptor in &spec.bindings {
            if let Some(action_id) = &descriptor.action_id {
                if let Ok(Some(binding)) = binding_descriptor_to_binding(descriptor) {
                    match action_id.as_str() {
                        "player::jump" => jump_bindings.push(binding),
                        "player::move" => move_bindings.push(binding),
                        _ => {}
                    }
                }
            }
        }
    });

    // Spawn player with enhanced input context
    commands.spawn((
        Player,
        actions!(Player[
            (
                Action::<Jump>::new(),
                Bindings::spawn(jump_bindings),
            ),
            (
                Action::<Move>::new(),
                Bindings::spawn(move_bindings),
            ),
        ])
    ));
}

// 4. React to actions in your game systems

// Observer style: Event-driven reactions
fn setup(app: &mut App) {
    app.add_observer(handle_jump);
}

fn handle_jump(
    trigger: On<Fire<Jump>>,
    mut transforms: Query<&mut Transform>,
) {
    if let Ok(mut transform) = transforms.get_mut(trigger.context) {
        transform.translation.y += 2.0;
    }
}

// Pull style: Query in update systems
fn handle_movement(
    query: Query<(&Action<Move>, &mut Transform), With<Player>>,
) {
    for (move_action, mut transform) in &query {
        if move_action.state == ActionState::Fired {
            transform.translation += move_action.value.extend(0.0);
        }
    }
}

#[derive(Component)]
struct Player;
```

This architecture provides:
- **Type safety**: Actions are types, not strings
- **Rebindable inputs**: Users can override bindings via JSON
- **Separation of concerns**: Keymap handles data, enhanced-input handles evaluation, your code handles logic

## Precedence rules

When evaluating input, bindings are ordered by:

1. **Context depth** – predicates matching deeper in the context stack win
2. **Source priority** – `USER` > `VIM` > `BASE` > `DEFAULT`
3. **Insertion order** – last added wins inside the same bucket

## Module overview

```
keymap/
├── binding.rs   # ActionId & KeyBinding definitions
├── context.rs   # Context & predicate parsing
├── keystroke.rs # Keystroke parsing utilities
├── keymap.rs    # Matching engine & precedence logic
├── store.rs     # Persistence (defaults + overrides)
├── enhanced.rs  # bevy_enhanced_input conversion helpers
└── lib.rs       # Public API surface
```

## Testing

Run the crate suite:

```shell
cargo test -p keymap
```

This exercises parsing, persistence, matching, and the enhanced-input bridge.
