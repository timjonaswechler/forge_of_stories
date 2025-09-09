/*!
Keymap Intent Mapper (Phase 7 – Tasks 7.1 & 7.2)

This module introduces a static label → Intent mapping independent of any
dynamic keymap configuration. It is the foundation for migrating from the
previous ad-hoc `map_label_to_action` function (defined in `ui::keymap::mod`)
to a structured, testable, and Intent‑centric approach.

Status:
- Task 7.1 (DONE here):
  * Static `LABEL_TO_INTENT` map (label → Intent(Action) clone)
  * Public API: `intent_for_label`
  * Tests covering several representative mappings
- Task 7.2 (PARTIAL skeleton):
  * Introduced `resolve_intent_with_fallback` (popup > page > global lookup)
  * Uses (local) chord normalization & dynamic keymap exports
  * Will need integration (mod.rs update + replacing old calls)
  * Additional tests for fallback semantics will be added in Task 7.2

Integration Plan:
1. Replace call sites of `map_label_to_action(label)` with `intent_for_label(label)`.
2. In the event loop, after deriving context(s), invoke
     resolve_intent_with_fallback(&settings, &[popup_ctx, page_ctx, "global"], key_event)
   falling back gracefully.
3. Remove/retire the legacy `map_label_to_action` once all usages migrate.

Why static mapping?
- Declarative, easily testable, zero runtime allocation after first access
- Separation of label parsing from chord/key resolution
- Future extension: allow layering custom (user-defined) intent tables on top

Caveats:
- Some legacy labels (e.g. "OpenPopup") that require constructing complex
  values (like boxed trait objects) are intentionally omitted from the static
  table. These can be handled by a specialized factory layer later.
- The `Intent` alias currently points to `Action`. As semantics split later,
  this table will naturally shift toward pure Intent variants.

*/

use crate::action::{Action, UiOutcome};
use crate::theme::Mode;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use settings::{DeviceFilter, SettingsStore};

/// Return the Intent corresponding to a label (stateless match implementation).
/// This replaces the earlier dynamic/static map approach to avoid requiring
/// `Sync` for the full `Action` enum (which contains non-Sync variants).
pub fn intent_for_label(label: &str) -> Option<Action> {
    use Action::*;
    match label {
        // Core / global
        "Quit" => Some(Quit),
        "Help" => Some(Help),
        "Submit" => Some(Submit),
        "Cancel" => Some(Action::UiOutcome(crate::action::UiOutcome::Cancelled)),

        // Navigation / focus
        "Up" => Some(Up),
        "Down" => Some(Down),
        "NextField" => Some(Down),
        "PreviousField" => Some(Up),
        "FocusNext" => Some(FocusNext),
        "FocusPrev" | "FocusPrevious" => Some(FocusPrev),
        "Switch" => Some(FocusNext),

        // Mode control
        "ModeCycle" | "ModeNext" => Some(CycleMode),
        "ModeNormal" => Some(SetMode(Mode::Normal)),
        "ModeInsert" => Some(SetMode(Mode::Insert)),
        "ModeVisual" => Some(SetMode(Mode::Visual)),
        "SwitchInputMode" => Some(SwitchInputMode),

        // Refresh / update
        "ResetFields" => Some(Refresh),

        // Overlay / keymap
        "ToggleKeymap" | "Keymap" => Some(ToggleKeymapOverlay),

        _ => None,
    }
}

/// Normalize a `KeyEvent` into a chord string (mirrors existing logic).
fn chord_from_key(key: KeyEvent) -> Option<String> {
    let (key_str, include_shift) = match key.code {
        KeyCode::Enter => ("enter".to_string(), true),
        KeyCode::Tab => ("tab".to_string(), true),
        KeyCode::Backspace => ("backspace".to_string(), true),
        KeyCode::Esc => ("esc".to_string(), true),
        KeyCode::Up => ("up".to_string(), true),
        KeyCode::Down => ("down".to_string(), true),
        KeyCode::Left => ("left".to_string(), true),
        KeyCode::Right => ("right".to_string(), true),
        KeyCode::Home => ("home".to_string(), true),
        KeyCode::End => ("end".to_string(), true),
        KeyCode::PageUp => ("pageup".to_string(), true),
        KeyCode::PageDown => ("pagedown".to_string(), true),
        KeyCode::F(n) => (format!("f{}", n), true),
        KeyCode::Char(' ') => ("space".to_string(), true),
        KeyCode::Char(ch) => (ch.to_ascii_lowercase().to_string(), true),
        _ => return None,
    };

    let mut mods: Vec<&str> = Vec::new();
    let m = key.modifiers;
    if m.contains(KeyModifiers::CONTROL) {
        mods.push("ctrl");
    }
    if include_shift && m.contains(KeyModifiers::SHIFT) {
        mods.push("shift");
    }
    if m.contains(KeyModifiers::ALT) {
        mods.push("alt");
    }
    if m.contains(KeyModifiers::SUPER) {
        mods.push("meta");
    }

    Some(if mods.is_empty() {
        key_str
    } else {
        format!("{}+{}", mods.join("+"), key_str)
    })
}

/// (Task 7.2 Skeleton) Resolve a key event to an Intent using a context fallback chain:
/// Order: highest specificity first (e.g. popup) → page → global.
///
/// Returns the first matching Intent produced by:
///  1. Converting the key to a chord.
///  2. Exporting the keymap for each context (SettingsStore).
///  3. Finding the label whose chord set contains the chord.
///  4. Mapping that label via `intent_for_label`.
///
/// NOTE:
/// - This does not (yet) support per-context overrides of the static label mapping.
/// - Task 7.2 will add unit tests ensuring correct fallback precedence.
pub fn resolve_intent_with_fallback(
    store: &SettingsStore,
    context_chain: &[&str],
    key: KeyEvent,
) -> Option<Action> {
    let chord = chord_from_key(key)?;
    for ctx in context_chain {
        let exported = store.export_keymap_for(DeviceFilter::Keyboard, ctx);
        if let Some((label, _)) = exported
            .iter()
            .find(|(_label, chords)| chords.iter().any(|c| c == &chord))
        {
            if let Some(intent) = intent_for_label(label) {
                return Some(intent);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::UiOutcome;
    use crate::theme::Mode;

    #[test]
    fn intent_basic_core() {
        assert!(matches!(intent_for_label("Quit"), Some(Action::Quit)));
        assert!(matches!(intent_for_label("Help"), Some(Action::Help)));
    }

    #[test]
    fn intent_navigation_aliases() {
        // "NextField" shares Down mapping
        assert!(matches!(intent_for_label("NextField"), Some(Action::Down)));
        // "Switch" maps to FocusNext
        assert!(matches!(
            intent_for_label("Switch"),
            Some(Action::FocusNext)
        ));
    }

    #[test]
    fn intent_modes() {
        assert!(matches!(
            intent_for_label("ModeInsert"),
            Some(Action::SetMode(Mode::Insert))
        ));
        assert!(matches!(
            intent_for_label("ModeNormal"),
            Some(Action::SetMode(Mode::Normal))
        ));
        assert!(matches!(
            intent_for_label("ModeCycle"),
            Some(Action::CycleMode)
        ));
    }

    #[test]
    fn intent_cancel_maps_to_ui_outcome_cancelled() {
        if let Some(Action::UiOutcome(crate::action::UiOutcome::Cancelled)) =
            intent_for_label("Cancel")
        {
            // success
        } else {
            panic!("Cancel label did not map to UiOutcome::Cancelled");
        }
    }

    #[test]
    fn intent_unknown_none() {
        assert!(intent_for_label("NonExistingLabelXYZ").is_none());
    }
}
