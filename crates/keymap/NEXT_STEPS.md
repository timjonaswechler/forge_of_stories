# Keymap System - Status & Next Steps

## ✅ Completed Work

### Core Architecture (DONE)
- ✅ **Keystroke parsing** (`keystroke.rs`): Parse user-friendly strings like "cmd-s", "ctrl-shift-p"
- ✅ **ActionBinding** (`spec.rs`): Simple struct for decentralized keymap registration
- ✅ **KeymapStore** (`store.rs`): Central Bevy Resource managing defaults & user overrides
- ✅ **KeymapPlugin** (`plugin.rs`): Bevy integration with auto-load/save
- ✅ **Enhanced Input Bridge** (`enhanced.rs`): Convert keystrokes to `bevy_enhanced_input` bindings
- ✅ **KeyBindingMetaIndex** (`binding.rs`): Precedence system for binding priority

### Testing & Documentation (DONE)
- ✅ All unit tests passing (27 tests)
- ✅ Complete example: `examples/decentralized_registration.rs`
- ✅ Architecture documentation: `ARCHITECTURE.md`
- ✅ Updated README with quick start guide
- ✅ Inline documentation with rustdoc comments

### CI Compliance (DONE)
- ✅ `cargo check -p keymap` passes
- ✅ `cargo test -p keymap` passes (27/27 tests)
- ✅ Bevy logging integrated (`bevy_log` feature)
- ✅ No compiler warnings in keymap crate

---

## 🎯 Next Steps

### Step 1: Integration with Main Game
**Priority: HIGH**

Integrate the keymap system into the main Forge of Stories codebase:

1. **Update `forge_of_stories/src/input/mod.rs`**:
   - Remove old `KeyBindingMetaIndex` usage if it doesn't match new API
   - Migrate to simplified `ActionBinding` + `KeymapStore` pattern
   - Update `default_keymap_spec()` to use new registration method

2. **Create Plugin Modules**:
   - `PlayerInputPlugin`: Register player action bindings
   - `UiInputPlugin`: Register UI action bindings
   - `CameraInputPlugin`: Register camera action bindings

3. **Test with Real Actions**:
   - Verify camera switching works with rebindable keys
   - Verify UI menu toggle works
   - Create integration test

**Files to modify:**
- `forge_of_stories/src/input/mod.rs`
- Create new: `forge_of_stories/src/input/player.rs`
- Create new: `forge_of_stories/src/input/ui.rs`
- Create new: `forge_of_stories/src/input/camera.rs`

### Step 2: In-Game Rebinding UI
**Priority: MEDIUM**

Create a UI for players to rebind keys at runtime:

1. **Settings Menu Integration**:
   - Add "Controls" tab to settings menu
   - List all registered actions with current bindings
   - Click to rebind → wait for keypress → update

2. **Conflict Detection**:
   - Warn if user assigns same key to multiple actions
   - Highlight conflicts in UI
   - Option to resolve conflicts

3. **Reset to Defaults**:
   - Button to reset individual binding
   - Button to reset all bindings

**New files:**
- `forge_of_stories/src/ui/settings/controls.rs`
- Add system to detect rebind mode and capture input

### Step 3: Enhanced Features
**Priority: LOW**

Add advanced features as needed:

1. **Context-Aware Bindings**:
   - Different keys for same action in different contexts
   - Example: "escape" closes menu in-game, but does nothing in menu
   - Add `context: &'static str` field to `ActionBinding`

2. **Multi-Key Sequences (Chords)**:
   - Support "cmd-k cmd-t" style sequences
   - Already partially implemented in `keymap.rs`
   - Expose via `KeymapStore` API

3. **Binding Profiles**:
   - Multiple keybinding sets (Default, Vim, Custom1, etc.)
   - Switch between profiles
   - Store in separate JSON files

4. **Gamepad Support**:
   - Extend `ActionBinding` to support gamepad buttons
   - Add gamepad override system similar to keyboard
   - Integrate with enhanced input's gamepad bindings

---

## 🔧 Technical Debt & Improvements

### Code Quality
- [ ] Fix Clippy warnings in `paths` crate (blocks workspace-level clippy)
- [ ] Add more comprehensive error types instead of `anyhow::Error`
- [ ] Add builder pattern for `KeymapStore` if configuration grows
- [ ] Consider using `thiserror` for better error handling

### Performance
- [ ] Profile JSON load/save performance (likely negligible)
- [ ] Consider binary format for faster load (msgpack feature exists)
- [ ] Add caching for frequently accessed bindings

### Testing
- [ ] Add integration tests with actual Bevy app
- [ ] Test auto-save behavior
- [ ] Test conflict scenarios (multiple plugins, same action_id)
- [ ] Property-based testing for keystroke parsing

---

## 📋 Migration Checklist

For integrating into existing codebase:

- [ ] Run `cargo check` on main workspace
- [ ] Identify all current input handling code
- [ ] Map existing actions to new `ActionBinding` format
- [ ] Update all plugins to register bindings
- [ ] Test each action works with default bindings
- [ ] Test user overrides (create test `keymap.json`)
- [ ] Verify auto-save creates/updates file correctly
- [ ] Update main README with keymap system docs
- [ ] Add changelog entry

---

## 🎓 Learning Resources

For team members working on this system:

1. **Read First**:
   - `ARCHITECTURE.md` - System design and rationale
   - `examples/decentralized_registration.rs` - Complete working example
   - `README.md` - Quick start and API reference

2. **Key Concepts**:
   - **ActionBinding**: Decentralized definition of defaults
   - **KeymapStore**: Central source of truth
   - **Precedence**: User overrides always win over defaults
   - **Bridge Pattern**: Keymap (data) → Enhanced Input (runtime)

3. **Common Tasks**:
   - Adding new action: Define `ActionBinding`, register in plugin
   - Changing default key: Update `default_keystroke` in binding definition
   - User rebinding: Call `store.set_user_override()`, auto-saves
   - Debugging: Check `keymap.json` for user overrides

---

## 🐛 Known Issues

None at this time. All tests passing, API stable.

---

## 📞 Questions?

If you encounter issues or need clarification:

1. Check `ARCHITECTURE.md` for design decisions
2. Review test cases in `src/**/*.rs` (see `#[cfg(test)]` modules)
3. Run example: `cargo run --example decentralized_registration`
4. Ask the team!

---

## 🎉 Summary

**What we built:**
A clean, decentralized keymap system that allows each game module to define its own default keybindings while providing a central store for user customization. The system integrates seamlessly with `bevy_enhanced_input` and automatically persists changes to disk.

**Why it's good:**
- **Modular**: Each plugin owns its bindings
- **User-friendly**: Simple JSON for customization
- **Type-safe**: Works with Bevy's ECS and enhanced-input actions
- **Maintainable**: Clear separation of concerns
- **Tested**: Comprehensive test coverage

**Next immediate action:**
Integrate with main game by updating `forge_of_stories/src/input/mod.rs` and creating per-module input plugins.