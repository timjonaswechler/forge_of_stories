//! Bevy integration for the keymap system.
//!
//! This module provides:
//! - KeyCode → Keystroke conversion (interpreter)
//! - Input handling system for keyboard events
//! - Context stack resource for hierarchical matching
//! - Bevy plugin for easy integration
//!
//! # Usage
//!
//! ```ignore
//! use bevy::prelude::*;
//! use keymap::bevy::{KeymapPlugin, ContextStack, ActionEvent};
//! use keymap::actions::file;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(KeymapPlugin::default())
//!         .add_systems(Startup, setup_keybindings)
//!         .add_systems(Update, handle_actions)
//!         .run();
//! }
//!
//! fn setup_keybindings(store: Res<KeymapStoreResource>) {
//!     // Bindings are automatically registered with default actions
//! }
//!
//! fn handle_actions(mut events: EventReader<ActionEvent>) {
//!     for event in events.read() {
//!         if event.action.partial_eq(&file::Save) {
//!             println!("Saving...");
//!         }
//!     }
//! }
//! ```

use crate::action::Action;
#[cfg(feature = "bevy_plugin")]
use crate::actions;
use crate::context::KeyContext;
use crate::keystroke::{Keystroke, Modifiers};
use crate::store::KeymapStore;
use bevy::ecs::event::Event;
use bevy::input::ButtonState;
use bevy::input::keyboard::{KeyCode, KeyboardInput};
use bevy::prelude::*;
use std::collections::VecDeque;
use std::sync::Arc;

/// Event that carries an action to be dispatched.
///
/// Systems can observe this event to handle specific actions.
#[derive(Event, Clone)]
pub struct ActionEvent {
    /// The action to be executed.
    pub action: Arc<dyn Action>,
}

/// Bevy plugin for keymap integration.
///
/// Adds the keymap system and resources to your Bevy app.
#[derive(Default)]
pub struct KeymapPlugin {
    /// Optional pre-configured KeymapStore.
    /// If None, a default store will be created.
    pub store: Option<KeymapStore>,
}

impl Plugin for KeymapPlugin {
    fn build(&self, app: &mut App) {
        // Insert KeymapStore resource with default actions registered
        let mut builder = KeymapStore::builder();

        #[cfg(feature = "bevy_plugin")]
        {
            builder = actions::register_default_actions(builder);
        }

        let store = builder
            .build()
            .expect("Failed to create default KeymapStore");
        app.insert_resource(KeymapStoreResource(store));

        // Insert context stack resource
        app.insert_resource(ContextStack::default());

        // Insert pending keystroke buffer
        app.insert_resource(PendingKeystrokes::default());

        // Register ActionEvent
        app.add_event::<ActionEvent>();

        // Add input handling system
        app.add_systems(PreUpdate, handle_keyboard_input);
    }

    fn name(&self) -> &str {
        "KeymapPlugin"
    }
}

/// Resource wrapper for KeymapStore to satisfy Bevy's Resource requirements.
#[derive(Resource)]
pub struct KeymapStoreResource(pub KeymapStore);

impl std::ops::Deref for KeymapStoreResource {
    type Target = KeymapStore;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Stack of active key contexts for hierarchical matching.
///
/// Contexts are evaluated from bottom to top (most specific last).
/// This allows child contexts to override parent contexts.
#[derive(Resource, Default, Debug, Clone)]
pub struct ContextStack {
    stack: Vec<KeyContext>,
}

impl ContextStack {
    /// Create a new empty context stack.
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Push a context onto the stack.
    pub fn push(&mut self, context: KeyContext) {
        self.stack.push(context);
    }

    /// Pop the top context from the stack.
    pub fn pop(&mut self) -> Option<KeyContext> {
        self.stack.pop()
    }

    /// Clear all contexts from the stack.
    pub fn clear(&mut self) {
        self.stack.clear();
    }

    /// Get a reference to the context stack.
    pub fn as_slice(&self) -> &[KeyContext] {
        &self.stack
    }

    /// Get the number of contexts in the stack.
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Check if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Replace the entire stack with new contexts.
    pub fn set(&mut self, contexts: Vec<KeyContext>) {
        self.stack = contexts;
    }
}

/// Buffer for pending keystrokes (multi-keystroke sequences).
#[derive(Resource, Default, Debug)]
struct PendingKeystrokes {
    buffer: VecDeque<Keystroke>,
}

impl PendingKeystrokes {
    fn push(&mut self, keystroke: Keystroke) {
        self.buffer.push_back(keystroke);
    }

    fn clear(&mut self) {
        self.buffer.clear();
    }

    fn as_slice(&self) -> Vec<Keystroke> {
        self.buffer.iter().cloned().collect()
    }
}

/// Convert Bevy's KeyCode to our Keystroke representation.
///
/// This is the core interpreter that maps Bevy's input to our keymap system.
pub fn keycode_to_keystroke(key_code: KeyCode, modifiers: &Modifiers) -> Option<Keystroke> {
    let key = match key_code {
        // Letters
        KeyCode::KeyA => "a",
        KeyCode::KeyB => "b",
        KeyCode::KeyC => "c",
        KeyCode::KeyD => "d",
        KeyCode::KeyE => "e",
        KeyCode::KeyF => "f",
        KeyCode::KeyG => "g",
        KeyCode::KeyH => "h",
        KeyCode::KeyI => "i",
        KeyCode::KeyJ => "j",
        KeyCode::KeyK => "k",
        KeyCode::KeyL => "l",
        KeyCode::KeyM => "m",
        KeyCode::KeyN => "n",
        KeyCode::KeyO => "o",
        KeyCode::KeyP => "p",
        KeyCode::KeyQ => "q",
        KeyCode::KeyR => "r",
        KeyCode::KeyS => "s",
        KeyCode::KeyT => "t",
        KeyCode::KeyU => "u",
        KeyCode::KeyV => "v",
        KeyCode::KeyW => "w",
        KeyCode::KeyX => "x",
        KeyCode::KeyY => "y",
        KeyCode::KeyZ => "z",

        // Digits
        KeyCode::Digit0 => "0",
        KeyCode::Digit1 => "1",
        KeyCode::Digit2 => "2",
        KeyCode::Digit3 => "3",
        KeyCode::Digit4 => "4",
        KeyCode::Digit5 => "5",
        KeyCode::Digit6 => "6",
        KeyCode::Digit7 => "7",
        KeyCode::Digit8 => "8",
        KeyCode::Digit9 => "9",

        // Function keys
        KeyCode::F1 => "f1",
        KeyCode::F2 => "f2",
        KeyCode::F3 => "f3",
        KeyCode::F4 => "f4",
        KeyCode::F5 => "f5",
        KeyCode::F6 => "f6",
        KeyCode::F7 => "f7",
        KeyCode::F8 => "f8",
        KeyCode::F9 => "f9",
        KeyCode::F10 => "f10",
        KeyCode::F11 => "f11",
        KeyCode::F12 => "f12",
        KeyCode::F13 => "f13",
        KeyCode::F14 => "f14",
        KeyCode::F15 => "f15",
        KeyCode::F16 => "f16",
        KeyCode::F17 => "f17",
        KeyCode::F18 => "f18",
        KeyCode::F19 => "f19",
        KeyCode::F20 => "f20",
        KeyCode::F21 => "f21",
        KeyCode::F22 => "f22",
        KeyCode::F23 => "f23",
        KeyCode::F24 => "f24",

        // Special keys
        KeyCode::Escape => "escape",
        KeyCode::Space => "space",
        KeyCode::Enter => "enter",
        KeyCode::Tab => "tab",
        KeyCode::Backspace => "backspace",
        KeyCode::Delete => "delete",
        KeyCode::Insert => "insert",
        KeyCode::Home => "home",
        KeyCode::End => "end",
        KeyCode::PageUp => "pageup",
        KeyCode::PageDown => "pagedown",

        // Arrow keys
        KeyCode::ArrowUp => "up",
        KeyCode::ArrowDown => "down",
        KeyCode::ArrowLeft => "left",
        KeyCode::ArrowRight => "right",

        // Punctuation
        KeyCode::Minus => "-",
        KeyCode::Equal => "=",
        KeyCode::BracketLeft => "[",
        KeyCode::BracketRight => "]",
        KeyCode::Backslash => "\\",
        KeyCode::Semicolon => ";",
        KeyCode::Quote => "'",
        KeyCode::Comma => ",",
        KeyCode::Period => ".",
        KeyCode::Slash => "/",
        KeyCode::Backquote => "`",

        // Modifiers (handled separately, don't generate keystrokes)
        KeyCode::ShiftLeft | KeyCode::ShiftRight => return None,
        KeyCode::ControlLeft | KeyCode::ControlRight => return None,
        KeyCode::AltLeft | KeyCode::AltRight => return None,
        KeyCode::SuperLeft | KeyCode::SuperRight => return None,

        // Numpad
        KeyCode::Numpad0 => "numpad0",
        KeyCode::Numpad1 => "numpad1",
        KeyCode::Numpad2 => "numpad2",
        KeyCode::Numpad3 => "numpad3",
        KeyCode::Numpad4 => "numpad4",
        KeyCode::Numpad5 => "numpad5",
        KeyCode::Numpad6 => "numpad6",
        KeyCode::Numpad7 => "numpad7",
        KeyCode::Numpad8 => "numpad8",
        KeyCode::Numpad9 => "numpad9",
        KeyCode::NumpadAdd => "numpad+",
        KeyCode::NumpadSubtract => "numpad-",
        KeyCode::NumpadMultiply => "numpad*",
        KeyCode::NumpadDivide => "numpad/",
        KeyCode::NumpadDecimal => "numpad.",
        KeyCode::NumpadEnter => "numpadenter",
        KeyCode::NumpadEqual => "numpad=",

        // Other keys
        KeyCode::CapsLock => "capslock",
        KeyCode::NumLock => "numlock",
        KeyCode::ScrollLock => "scrolllock",
        KeyCode::PrintScreen => "printscreen",
        KeyCode::Pause => "pause",

        // Ignore other keys
        _ => return None,
    };

    Some(Keystroke {
        key: key.to_string(),
        modifiers: *modifiers,
    })
}

/// Extract modifier state from Bevy's keyboard input.
fn extract_modifiers(keyboard: &ButtonInput<KeyCode>) -> Modifiers {
    Modifiers {
        ctrl: keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]),
        alt: keyboard.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]),
        shift: keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
        cmd: keyboard.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]),
    }
}

/// System that handles keyboard input and dispatches actions.
///
/// This system:
/// 1. Converts Bevy KeyboardInput events to Keystrokes
/// 2. Buffers keystrokes for multi-keystroke sequences
/// 3. Matches keystrokes against the keymap
/// 4. Triggers actions (sends events)
fn handle_keyboard_input(
    mut keyboard_events: EventReader<KeyboardInput>,
    keyboard: Res<ButtonInput<KeyCode>>,
    store: Res<KeymapStoreResource>,
    context_stack: Res<ContextStack>,
    mut pending: ResMut<PendingKeystrokes>,
    mut action_events: EventWriter<ActionEvent>,
) {
    for event in keyboard_events.read() {
        // Only process key presses, not releases
        if event.state != ButtonState::Pressed {
            continue;
        }

        // Extract modifiers
        let modifiers = extract_modifiers(&keyboard);

        // Convert KeyCode to Keystroke
        let Some(keystroke) = keycode_to_keystroke(event.key_code, &modifiers) else {
            continue;
        };

        // Add to pending buffer
        pending.push(keystroke);

        // Try to match against keymap
        let pending_sequence = pending.as_slice();
        let contexts = context_stack.as_slice();

        let (matches, has_pending) =
            store.with_keymap(|keymap| keymap.bindings_for_input(&pending_sequence, contexts));

        if !matches.is_empty() {
            // Found a match! Execute the action
            let binding = &matches[0];
            let action = binding.action();

            // Log action (using bevy's built-in logging)
            println!(
                "[KEYMAP] Action triggered: {} from keystroke sequence: {}",
                action.debug_name(),
                pending_sequence
                    .iter()
                    .map(|k| k.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            );

            // Dispatch action as event
            action_events.write(ActionEvent {
                action: Arc::from(action.boxed_clone()),
            });

            // Clear pending buffer
            pending.clear();
        } else if has_pending {
            // Waiting for more keystrokes (multi-keystroke sequence)
            println!(
                "[KEYMAP] Pending keystroke sequence: {}",
                pending_sequence
                    .iter()
                    .map(|k| k.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        } else {
            // No match found, clear buffer
            println!(
                "[KEYMAP] No binding found for: {}",
                pending_sequence
                    .iter()
                    .map(|k| k.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            pending.clear();
        }
    }
}

/// Example: How to handle actions in your Bevy app
///
/// ```ignore
/// use bevy::prelude::*;
/// use keymap::bevy::{ActionEvent, KeymapPlugin};
/// use keymap::actions::file;
///
/// fn main() {
///     App::new()
///         .add_plugins(KeymapPlugin::default())
///         .add_systems(Update, handle_file_actions)
///         .run();
/// }
///
/// fn handle_file_actions(mut events: EventReader<ActionEvent>) {
///     for event in events.read() {
///         // Check action type and handle it
///         if event.action.partial_eq(&file::Save) {
///             println!("Saving file...");
///             // Perform save operation
///         } else if event.action.partial_eq(&file::Open) {
///             println!("Opening file...");
///             // Perform open operation
///         }
///     }
/// }
/// ```
///
/// Or use the type-safe observer pattern:
///
/// ```ignore
/// fn setup(mut commands: Commands) {
///     commands.spawn(MyEntity)
///         .observe(on_save_action);
/// }
///
/// fn on_save_action(trigger: Trigger<ActionEvent>) {
///     if trigger.event().action.partial_eq(&file::Save) {
///         // Handle save
///     }
/// }
/// ```
pub fn example_action_handler(mut events: EventReader<ActionEvent>) {
    for event in events.read() {
        // Example: Log all triggered actions
        println!("[ACTION] Triggered: {}", event.action.debug_name());

        // In a real app, you would match against specific actions:
        // if event.action.partial_eq(&crate::actions::file::Save) { ... }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycode_to_keystroke() {
        let modifiers = Modifiers::NONE;

        // Test letter
        let keystroke = keycode_to_keystroke(KeyCode::KeyA, &modifiers).unwrap();
        assert_eq!(keystroke.key, "a");
        assert!(!keystroke.modifiers.ctrl);

        // Test with modifiers
        let modifiers = Modifiers {
            ctrl: true,
            shift: true,
            alt: false,
            cmd: false,
        };
        let keystroke = keycode_to_keystroke(KeyCode::KeyS, &modifiers).unwrap();
        assert_eq!(keystroke.key, "s");
        assert!(keystroke.modifiers.ctrl);
        assert!(keystroke.modifiers.shift);

        // Test function key
        let keystroke = keycode_to_keystroke(KeyCode::F1, &Modifiers::NONE).unwrap();
        assert_eq!(keystroke.key, "f1");

        // Test special key
        let keystroke = keycode_to_keystroke(KeyCode::Escape, &Modifiers::NONE).unwrap();
        assert_eq!(keystroke.key, "escape");

        // Modifiers should return None
        assert!(keycode_to_keystroke(KeyCode::ControlLeft, &Modifiers::NONE).is_none());
    }

    #[test]
    fn test_context_stack() {
        let mut stack = ContextStack::new();
        assert!(stack.is_empty());

        let mut ctx1 = KeyContext::default();
        ctx1.add("Editor");
        stack.push(ctx1.clone());

        assert_eq!(stack.len(), 1);
        assert!(!stack.is_empty());

        let mut ctx2 = KeyContext::default();
        ctx2.add("Panel");
        stack.push(ctx2.clone());

        assert_eq!(stack.len(), 2);
        assert_eq!(stack.as_slice().len(), 2);

        let popped = stack.pop().unwrap();
        assert_eq!(popped, ctx2);
        assert_eq!(stack.len(), 1);

        stack.clear();
        assert!(stack.is_empty());
    }
}
