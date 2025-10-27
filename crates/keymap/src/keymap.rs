//! Keymap matching engine.
//!
//! This module implements the core matching logic for key bindings,
//! including precedence rules, context evaluation, and multi-keystroke sequences.

use crate::binding::{KeyBinding, KeyBindingMetaIndex};
use crate::keystroke::Keystroke;
use smallvec::SmallVec;

/// An opaque identifier of which version of the keymap is currently active.
/// The keymap's version is changed whenever bindings are added or removed.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct KeymapVersion(usize);

/// A collection of key bindings for the application.
#[derive(Default)]
pub struct Keymap {
    bindings: Vec<KeyBinding>,
    version: KeymapVersion,
}

impl Keymap {
    /// Create a new empty keymap.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a keymap with the given bindings.
    pub fn with_bindings(bindings: Vec<KeyBinding>) -> Self {
        let mut keymap = Self::new();
        keymap.add_bindings(bindings);
        keymap
    }

    /// Get the current version of the keymap.
    pub fn version(&self) -> KeymapVersion {
        self.version
    }

    /// Add more bindings to the keymap.
    pub fn add_bindings<T: IntoIterator<Item = KeyBinding>>(&mut self, bindings: T) {
        for binding in bindings {
            self.bindings.push(binding);
        }
        self.version.0 += 1;
    }

    /// Reset this keymap to its initial state.
    pub fn clear(&mut self) {
        self.bindings.clear();
        self.version.0 += 1;
    }

    /// Iterate over all bindings, in the order they were added.
    pub fn bindings(&self) -> impl DoubleEndedIterator<Item = &KeyBinding> + ExactSizeIterator {
        self.bindings.iter()
    }

    /// Returns all bindings that might match the input without checking context.
    /// Bindings are returned in precedence order (reverse of the order they were added).
    pub fn all_bindings_for_input(&self, input: &[Keystroke]) -> Vec<KeyBinding> {
        self.bindings()
            .rev()
            .filter_map(|binding| {
                let pending = binding.match_keystrokes(input)?;
                if pending {
                    return None; // Don't include partial matches
                }
                if binding.is_disabled() {
                    return None;
                }
                Some(binding.clone())
            })
            .collect()
    }

    /// Returns a list of bindings that match the given input, and a boolean indicating
    /// whether or not more bindings might match if the input was longer.
    ///
    /// Bindings are returned in precedence order (higher precedence first).
    ///
    /// Precedence is defined by:
    /// 1. Source priority (`meta` field: USER > VIM > BASE > DEFAULT)
    /// 2. Order (later bindings win)
    ///
    /// If a binding with higher precedence has `action_id: None`, it disables
    /// any lower-precedence bindings for the same keystroke sequence.
    pub fn bindings_for_input(&self, input: &[Keystroke]) -> (SmallVec<[KeyBinding; 1]>, bool) {
        let mut matched_bindings = SmallVec::<[(usize, &KeyBinding); 1]>::new();
        let mut pending_bindings = SmallVec::<[&KeyBinding; 1]>::new();
        let mut disabled_keystrokes = std::collections::HashSet::new();

        // Iterate in reverse to handle precedence (later bindings win).
        for (ix, binding) in self.bindings().enumerate().rev() {
            let Some(pending) = binding.match_keystrokes(input) else {
                continue;
            };

            // If a higher-precedence binding disables this keystroke, record it.
            if binding.is_disabled() {
                if !pending {
                    disabled_keystrokes.insert(binding.keystrokes());
                }
                continue;
            }

            // If these exact keystrokes have been disabled, skip.
            if disabled_keystrokes.contains(binding.keystrokes()) {
                continue;
            }

            if !pending {
                matched_bindings.push((ix, binding));
            } else {
                pending_bindings.push(binding);
            }
        }

        // Sort by precedence: meta ASC, then index DESC.
        matched_bindings.sort_by(|(ix_a, binding_a), (ix_b, binding_b)| {
            let meta_a = binding_a.metadata().unwrap_or(KeyBindingMetaIndex::DEFAULT);
            let meta_b = binding_b.metadata().unwrap_or(KeyBindingMetaIndex::DEFAULT);
            meta_a.cmp(&meta_b).then(ix_b.cmp(ix_a))
        });

        let bindings: SmallVec<[_; 1]> = matched_bindings
            .into_iter()
            .map(|(_, binding)| binding.clone())
            .collect();

        // Determine if there are pending matches that are not disabled.
        let has_pending = pending_bindings
            .iter()
            .any(|b| !disabled_keystrokes.contains(b.keystrokes()));

        (bindings, has_pending)
    }

    /// Get all bindings for a specific action identifier.
    pub fn bindings_for_action(&self, action_id: impl AsRef<str>) -> Vec<&KeyBinding> {
        let action_id = action_id.as_ref();
        self.bindings
            .iter()
            .filter(|binding| {
                binding
                    .action_id()
                    .map(|id| id.as_str() == action_id)
                    .unwrap_or(false)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::ActionId;
    use crate::keystroke::{Keystroke, parse_keystroke_sequence};

    fn binding(sequence: &str, action: Option<&str>) -> KeyBinding {
        let keystrokes = parse_keystroke_sequence(sequence).unwrap();
        let action_id = action.map(ActionId::from);
        KeyBinding::new(keystrokes, action_id)
    }

    #[test]
    fn test_keymap_creation() {
        let keymap = Keymap::new();
        assert_eq!(keymap.bindings().count(), 0);
        assert_eq!(keymap.version(), KeymapVersion(0));
    }

    #[test]
    fn test_add_bindings() {
        let mut keymap = Keymap::new();
        let binding = binding("cmd-s", Some("ActionAlpha"));

        keymap.add_bindings(vec![binding]);
        assert_eq!(keymap.bindings().count(), 1);
        assert_eq!(keymap.version(), KeymapVersion(1));
    }

    #[test]
    fn test_order_precedence() {
        let bindings = vec![
            binding("cmd-s", Some("ActionAlpha")),
            binding("cmd-s", Some("ActionBeta")),
        ];

        let keymap = Keymap::with_bindings(bindings);

        let (result, pending) = keymap.bindings_for_input(&[Keystroke::parse("cmd-s").unwrap()]);

        assert!(!pending);
        assert_eq!(result.len(), 2);
        // Later binding ("Beta") should have higher precedence.
        assert_eq!(result[0].action_id().unwrap().as_str(), "ActionBeta");
        assert_eq!(result[1].action_id().unwrap().as_str(), "ActionAlpha");
    }

    #[test]
    fn test_no_action_disables_binding() {
        // The `None` binding has higher precedence (added later)
        let bindings = vec![
            binding("cmd-s", Some("ActionAlpha")),
            binding("cmd-s", None),
        ];

        let keymap = Keymap::with_bindings(bindings);

        // The "None" binding should prevent the "ActionAlpha" binding from matching.
        let (result, _) = keymap.bindings_for_input(&[Keystroke::parse("cmd-s").unwrap()]);
        assert_eq!(result.len(), 0);

        // Test the other way around: if the disabling binding has lower precedence
        let bindings_reversed = vec![
            binding("cmd-s", None),
            binding("cmd-s", Some("ActionAlpha")),
        ];

        let keymap_reversed = Keymap::with_bindings(bindings_reversed);
        let (result, _) = keymap_reversed.bindings_for_input(&[Keystroke::parse("cmd-s").unwrap()]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].action_id().unwrap().as_str(), "ActionAlpha");
    }

    #[test]
    fn test_multi_keystroke_pending() {
        let bindings = vec![binding("cmd-k cmd-t", Some("ActionAlpha"))];

        let keymap = Keymap::with_bindings(bindings);

        // First keystroke: pending
        let (result, pending) = keymap.bindings_for_input(&[Keystroke::parse("cmd-k").unwrap()]);
        assert!(pending);
        assert_eq!(result.len(), 0);

        // Complete sequence: match
        let (result, pending) = keymap.bindings_for_input(&[
            Keystroke::parse("cmd-k").unwrap(),
            Keystroke::parse("cmd-t").unwrap(),
        ]);
        assert!(!pending);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].action_id().unwrap().as_str(), "ActionAlpha");
    }

    #[test]
    fn test_source_precedence() {
        let mut binding_default = binding("cmd-s", Some("ActionAlpha"));
        binding_default.set_meta(KeyBindingMetaIndex::DEFAULT);

        let mut binding_user = binding("cmd-s", Some("ActionBeta"));
        binding_user.set_meta(KeyBindingMetaIndex::USER);

        let keymap = Keymap::with_bindings(vec![binding_default, binding_user]);

        let (result, _) = keymap.bindings_for_input(&[Keystroke::parse("cmd-s").unwrap()]);

        assert_eq!(result.len(), 2);
        // User binding should come first
        assert_eq!(result[0].action_id().unwrap().as_str(), "ActionBeta");
        assert_eq!(result[1].action_id().unwrap().as_str(), "ActionAlpha");
    }

    #[test]
    fn test_bindings_for_action() {
        let bindings = vec![
            binding("cmd-s", Some("ActionAlpha")),
            binding("ctrl-s", Some("ActionAlpha")),
            binding("cmd-o", Some("ActionBeta")),
        ];

        let keymap = Keymap::with_bindings(bindings);
        let alpha_bindings = keymap.bindings_for_action("ActionAlpha");

        assert_eq!(alpha_bindings.len(), 2);
    }
}
