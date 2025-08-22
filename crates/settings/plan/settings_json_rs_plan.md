# Plan: Settings JSON → TOML Editing Layer

Derived solely from crates/settings/memories/settings_json_rs_analysis.md.

## Extracted Model (from memories)
- JSON text editor with format/indent/comment preservation.
- Tree-sitter driven queries to locate key paths precisely.
- Batch edits with minimal diffs; array/object add/replace/remove.
- Pretty-printer and lenient JSON parsing for comments.

## Target Direction (TOML)
- Replace JSON backend with TOML using `toml_edit` for format- and comment-preserving edits.
- Keep batch-edit semantics and minimal-diff philosophy.
- Provide object-like and array-like operations with key paths.

## Addressing Scheme (Key Paths)
- Use dotted TOML key paths (e.g., input.mouse.sensitivity) and array indices (e.g., bindings[3]).
- Define a small `KeyPath` type that supports:
  - Table navigation (dotted identifiers / quoted keys).
  - Array indices and appends.
  - Creation policy (create_intermediate: bool).

## Core APIs
- update_value_in_toml_text(text: &mut String, key_path: &KeyPath, old: &Value, new: &Value, tab_size: usize, edits: &mut Vec<(Range<usize>, String)>) -> Result<()>
  - Mirrors JSON version but operates on `toml_edit::Document` and produces minimal text edits.
- replace_top_level_array_value_in_toml_text(text, key_path, new_value: Option<&Value>, array_index, tab_size) -> Result<(Range<usize>, String)>
- append_top_level_array_value_in_toml_text(text, new_value, tab_size) -> Result<(Range<usize>, String)>
- to_pretty_toml(value, indent_size, indent_prefix_len) -> String

Notes:
- Use `toml_edit::Document` to parse, mutate via Items (Table, Array, Value), then compute textual replacement ranges by reserializing only the mutated subtree when feasible; otherwise fall back to whole-doc replacement.

## Comment & Formatting Preservation
- Respect existing indentation and spacing; probe indentation from surrounding nodes.
- Preserve comments by retaining `decor` on `toml_edit::Item`s.
- When inserting new keys, mirror sibling decor and insertion order.

## Error Handling & Recovery
- If TOML parse fails, return a structured error but optionally provide a best-effort pretty output for diagnostics.
- Guard array indices; extend arrays on out-of-bounds when policy allows.

## Schema/Validation
- Drop JSON schemars integration here; validation occurs in higher layers (domain-specific checks).
- Optionally provide a lightweight schema check for primitive types and required fields.

## Tests (to mirror JSON suite scope)
- Object/table operations: add/replace/remove, nested paths, comment preservation.
- Array operations: index replace, append, formatting consistency.
- Pretty-printer behavior under different indent sizes.

## Implementation Steps
1) Define `KeyPath` and parsing from dotted strings with array indices.
2) Implement core update/array functions using `toml_edit` with minimal diffs.
3) Implement pretty TOML helper that honors indent size/prefix.
4) Build a representative test matrix covering objects, arrays, comments, and formatting.
