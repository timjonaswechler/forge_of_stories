//! Example showing how game modules can decentrally register their keybindings.
//!
//! This demonstrates the intended workflow:
//! 1. Each module defines its default keybindings as a static slice
//! 2. During plugin initialization, it registers these with the central KeymapStore
//! 3. At runtime, systems retrieve the final (possibly user-overridden) bindings
//! 4. These are converted to bevy_enhanced_input bindings

use bevy::prelude::*;
use keymap::{ActionBinding, KeymapStore};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            keymap::plugin::KeymapPlugin::default(),
            PlayerPlugin,
            UiPlugin,
        ))
        .run();
}

// ============================================================================
// PLAYER MODULE - Decentralized Keymap Definition
// ============================================================================

struct PlayerPlugin;

/// Player module's default keybindings - defined locally in the module
const PLAYER_BINDINGS: &[ActionBinding] = &[
    ActionBinding {
        action_id: "player.jump",
        default_keystroke: "space",
    },
    ActionBinding {
        action_id: "player.move_forward",
        default_keystroke: "w",
    },
    ActionBinding {
        action_id: "player.move_backward",
        default_keystroke: "s",
    },
    ActionBinding {
        action_id: "player.move_left",
        default_keystroke: "a",
    },
    ActionBinding {
        action_id: "player.move_right",
        default_keystroke: "d",
    },
    ActionBinding {
        action_id: "player.sprint",
        default_keystroke: "shift",
    },
];

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // Register our default keybindings with the central store
        let mut store = app.world_mut().resource_mut::<KeymapStore>();
        store.register_defaults(PLAYER_BINDINGS);

        // Add our systems
        app.add_systems(Startup, setup_player_actions);
        app.add_systems(Update, log_player_bindings);
    }
}

fn setup_player_actions(keymap: Res<KeymapStore>) {
    info!("=== Player Actions Setup ===");

    // Retrieve the final keystroke for each action
    // (might be user-overridden, or the default)
    if let Some(jump_key) = keymap.get_binding("player.jump") {
        info!("Jump: {}", jump_key);

        // In a real app, you would convert this to a bevy_enhanced_input binding:
        // let binding = keymap::enhanced::keystroke_to_keyboard(&jump_key).unwrap();
        // commands.spawn((
        //     Action::<Jump>::new(),
        //     Bindings::new([binding]),
        // ));
    }

    if let Some(forward_key) = keymap.get_binding("player.move_forward") {
        info!("Move Forward: {}", forward_key);
    }
}

fn log_player_bindings(keymap: Res<KeymapStore>, mut has_run: Local<bool>) {
    if *has_run {
        return;
    }
    *has_run = true;

    info!("=== Current Player Keybindings ===");
    for binding in PLAYER_BINDINGS {
        if let Some(keystroke) = keymap.get_binding(binding.action_id) {
            info!("  {} -> {}", binding.action_id, keystroke);
        }
    }
}

// ============================================================================
// UI MODULE - Another decentralized definition
// ============================================================================

struct UiPlugin;

/// UI module's default keybindings
const UI_BINDINGS: &[ActionBinding] = &[
    ActionBinding {
        action_id: "ui.toggle_menu",
        default_keystroke: "escape",
    },
    ActionBinding {
        action_id: "ui.toggle_inventory",
        default_keystroke: "i",
    },
    ActionBinding {
        action_id: "ui.quick_save",
        default_keystroke: "f5",
    },
    ActionBinding {
        action_id: "ui.quick_load",
        default_keystroke: "f9",
    },
];

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // Register UI keybindings
        let mut store = app.world_mut().resource_mut::<KeymapStore>();
        store.register_defaults(UI_BINDINGS);

        app.add_systems(Startup, setup_ui_actions);
    }
}

fn setup_ui_actions(keymap: Res<KeymapStore>) {
    info!("=== UI Actions Setup ===");

    if let Some(menu_key) = keymap.get_binding("ui.toggle_menu") {
        info!("Toggle Menu: {}", menu_key);
    }

    if let Some(inventory_key) = keymap.get_binding("ui.toggle_inventory") {
        info!("Toggle Inventory: {}", inventory_key);
    }
}
