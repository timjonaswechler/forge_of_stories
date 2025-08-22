# Plan: Keymap File (Game Controls + TOML)

Derived solely from crates/settings/memories/keymap_file_rs_analysis.md.

## Extracted Model (from memories)
- KeymapFile = list of KeymapSection with context predicates and bindings.
- Permissive parsing with error accumulation and partial load.
- Pluggable validation for actions; keystroke parsing and context-aware enablement.
- JSON-centric schema, comment-preserving updates, and layered sources (KeybindSource).

## Game-Oriented Design (TOML)
- Replace JSON with TOML; use toml_edit for comment/format preservation.
- Keep sections and optional context predicates; keep deterministic order.
- Simplify sources: User > BaseProfile > Default (remove Vim). Name as KeybindSource.
- Actions are game actions (strings) validated against a registry.

## TOML Schema
Example file:
```toml
# keymaps/user/input.toml
[[section]]
context = "InGame && !ChatOpen"
use_key_equivalents = false

[section.bindings]
"W"        = "move_forward"
"Shift+W"  = ["sprint", { when = "pressed" }]
"Escape"   = "pause"

[[section]]
context = "Global"
[section.bindings]
"F1" = "toggle_help"
```
- section.context: optional; boolean expression (AND/OR/! with identifiers).
- section.use_key_equivalents: optional; layout-position mapping if true.
- section.bindings: table mapping keystroke strings to actions or [action, params].

## Parsing & Validation
- Parse TOML into in-memory KeymapFile; collect errors (line/col, message).
- Keystroke parser: robust errors with expected tokens; supports chords and modifiers.
- Action resolution: strings map to registered actions; array form carries params (TOML table).
- Validators: trait KeyBindingValidator { fn action_id(&self) -> &'static str; fn validate(..) -> Result<()> }
  - Registration via a simple runtime registry (no gpui, inventory optional/feature-gated).

## Update API (comment-preserving)
- update_keybinding(operation, contents: String, tab_size) -> Result<String>
  - Add/Replace/Remove with smart fallbacks when target is not in User layer → add/NoAction in User.
- Use toml_edit to:
  - Locate a matching section by context (string equality) or create one.
  - Insert/update key in [section.bindings] while preserving order/comments.
  - Remove key; if section becomes empty, optionally prune it.

## Layering & Resolution
- Sources: Default (engine), BaseProfile (selected preset), User (highest).
- Resolution order: Default < BaseProfile < User; operations write to User.
- For None profile, BaseProfile layer is skipped.

## Keyboard Layout & Equivalents
- Optional layout-aware mapping when use_key_equivalents = true (QWERTY-like behavior on macOS).
- Provide layout id from settings; mapping is a simple char→char table.

## Open Questions
- Keep use_key_equivalents flag or derive from OS/layout automatically?
- Exact keystroke grammar (do we allow multi-stroke chords like "Ctrl+K Ctrl+C")?
- Do we need per-context priority or rely on first-match order?

## Implementation Steps
1) Define data types for KeymapFile/Section/Action (serde for TOML).
2) Implement keystroke parser and action registry + validators.
3) Implement loader with error accumulation and partial success.
4) Implement update_keybinding with toml_edit minimal diffs.
5) Wire layering: merge Default/BaseProfile/User into effective bindings.
