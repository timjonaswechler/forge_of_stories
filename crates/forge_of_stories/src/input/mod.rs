use crate::{ui::InGameMenuState, GameState};
use anyhow::Result;
use bevy::prelude::*;
use keymap::{
    enhanced::{binding_descriptor_to_binding, ConversionError},
    parse_keystroke_sequence, ActionDescriptor, ActionId, BindingDescriptor,
    BindingInputDescriptor, ContextDescriptor, ContextId, Keystroke, KeyBindingMetaIndex,
    KeymapSpec, KeymapStore, Modifiers,
};
use paths::PathContext;
use tracing::warn;

/// Resource wrapper around [`KeymapStore`] so Bevy can manage it.
#[derive(Resource)]
pub struct KeymapStoreResource(KeymapStore);

impl KeymapStoreResource {
    pub fn new(store: KeymapStore) -> Self {
        Self(store)
    }
}

impl std::ops::Deref for KeymapStoreResource {
    type Target = KeymapStore;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Build a keymap store using the default descriptors bundled with the game.
///
/// The store is configured to read user overrides from the project's keybinding file
/// (see [`PathContext::keybinding_file`]). Invalid user files are ignored with a warning.
pub fn create_keymap_store(path_context: &PathContext) -> Result<KeymapStore> {
    let spec = default_keymap_spec();
    let keymap_path = path_context.keybinding_file();

    let store = KeymapStore::builder()
        .with_default_spec(spec)
        .with_user_keymap_path(keymap_path)
        .build()?;

    if let Err(err) = store.load_user_bindings() {
        warn!("Failed to load user keymap overrides: {err:#}");
    }

    Ok(store)
}

pub struct KeymapInputPlugin;

impl Plugin for KeymapInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, log_loaded_keymap)
            .add_systems(
                Update,
                toggle_menu_from_keymap.run_if(in_state(GameState::InGame)),
            );
    }
}

fn log_loaded_keymap(store: Res<KeymapStoreResource>) {
    store.with_spec(|spec| {
        info!(
            "Loaded keymap with {} actions, {} contexts, {} bindings",
            spec.actions.len(),
            spec.contexts.len(),
            spec.bindings.len()
        );

        for descriptor in &spec.bindings {
            let action = descriptor
                .action_id
                .as_ref()
                .map(|id| id.as_str())
                .unwrap_or("<disabled>");
            let context = descriptor
                .context_id
                .as_ref()
                .map(|id| id.as_str())
                .unwrap_or("<global>");

            match binding_descriptor_to_binding(descriptor) {
                Ok(Some(binding)) => {
                    info!("  [{context}] {action} => {binding:?}");
                }
                Ok(None) => {
                    info!("  [{context}] {action} => (no binding)");
                }
                Err(err) => log_conversion_error(context, action, err),
            }
        }
    });
}

fn log_conversion_error(context: &str, action: &str, err: ConversionError) {
    warn!("  [{context}] {action} => conversion failed: {err}");
}

fn toggle_menu_from_keymap(
    keys: Res<ButtonInput<KeyCode>>,
    store: Res<KeymapStoreResource>,
    mut menu: ResMut<InGameMenuState>,
) {
    let mut toggled = false;

    for key in keys.get_just_pressed() {
        let Some(keystroke) = keycode_to_keystroke(*key, &keys) else {
            continue;
        };

        store.with_keymap(|keymap| {
            let (bindings, _) = keymap.bindings_for_input(&[keystroke.clone()], &[]);
            toggled |= bindings.iter().any(|binding| {
                binding
                    .action_id()
                    .map(|id| id.as_str() == "ui.toggle_menu")
                    .unwrap_or(false)
            });
        });
    }

    if toggled {
        menu.toggle();
    }
}

fn keycode_to_keystroke(
    key_code: KeyCode,
    keys: &ButtonInput<KeyCode>,
) -> Option<Keystroke> {
    let key = match key_code {
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
        KeyCode::ArrowUp => "up",
        KeyCode::ArrowDown => "down",
        KeyCode::ArrowLeft => "left",
        KeyCode::ArrowRight => "right",
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
        KeyCode::CapsLock => "capslock",
        KeyCode::NumLock => "numlock",
        KeyCode::ScrollLock => "scrolllock",
        KeyCode::PrintScreen => "printscreen",
        KeyCode::Pause => "pause",
        KeyCode::ControlLeft | KeyCode::ControlRight | KeyCode::ShiftLeft | KeyCode::ShiftRight
        | KeyCode::AltLeft | KeyCode::AltRight | KeyCode::SuperLeft | KeyCode::SuperRight => {
            return None
        }
        _ => return None,
    };

    let modifiers = Modifiers {
        ctrl: keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]),
        alt: keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]),
        shift: keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
        cmd: keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]),
    };

    Some(Keystroke::new(key, modifiers))
}

fn default_keymap_spec() -> KeymapSpec {
    let actions = vec![ActionDescriptor {
        id: ActionId::from("ui.toggle_menu"),
        output: Some("bool".into()),
        modifiers: Vec::new(),
        conditions: Vec::new(),
        settings: None,
    }];

    let contexts = vec![ContextDescriptor {
        id: ContextId::from("global"),
        predicate: None,
        priority: Some(0.0),
        schedule: None,
        settings: None,
    }];

    let bindings = vec![
        BindingDescriptor {
            action_id: Some(ActionId::from("ui.toggle_menu")),
            context_id: Some(ContextId::from("global")),
            predicate: None,
            meta: Some(KeyBindingMetaIndex::DEFAULT),
            modifiers: Vec::new(),
            conditions: Vec::new(),
            settings: None,
            input: Some(BindingInputDescriptor::keyboard(
                parse_keystroke_sequence("escape").expect("static key sequence"),
            )),
        },
        BindingDescriptor {
            action_id: Some(ActionId::from("ui.toggle_menu")),
            context_id: Some(ContextId::from("global")),
            predicate: None,
            meta: Some(KeyBindingMetaIndex::DEFAULT),
            modifiers: Vec::new(),
            conditions: Vec::new(),
            settings: None,
            input: Some(BindingInputDescriptor::MouseButton {
                button: "right".into(),
                modifiers: vec!["shift".into()],
            }),
        },
        BindingDescriptor {
            action_id: Some(ActionId::from("ui.toggle_menu")),
            context_id: Some(ContextId::from("global")),
            predicate: None,
            meta: Some(KeyBindingMetaIndex::DEFAULT),
            modifiers: Vec::new(),
            conditions: Vec::new(),
            settings: None,
            input: Some(BindingInputDescriptor::GamepadButton {
                button: "start".into(),
                threshold: None,
            }),
        },
    ];

    KeymapSpec {
        actions,
        contexts,
        bindings,
    }
}

pub use KeymapInputPlugin as InputPlugin;
pub use KeymapStoreResource as StoreResource;
