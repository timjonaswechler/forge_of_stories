# Keymap Integration Follow-up

- [ ] Map `ActionId` values to concrete `bevy_enhanced_input::InputAction` implementations and spawn action/context entities at runtime (e.g. via registries).
- [ ] Populate enhanced-input context entities (and legacy context stacks) so `context_id`/predicates actually gate actions such as the in-game menu.
- [ ] Support serialization/deserialization of modifiers, conditions, and action/context settings (currently ignored in the demo default spec).
- [ ] Provide a UI or CLI flow for editing bindings and writing user overrides back to `keybinding.json`.
- [ ] Expand automated tests that cover descriptor → enhanced-input conversion for gamepad axes/buttons and error scenarios.
- [ ] Replace the logging-only `KeymapDemoPlugin` with gameplay systems that react to loaded actions (e.g. toggling menus) and remove the demonstration code.
