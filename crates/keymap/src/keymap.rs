//! Keymap matching engine.
//!
//! This module implements the core matching logic for key bindings,
//! including precedence rules, context evaluation, and multi-keystroke sequences.

use crate::action::{Action, is_no_action};
use crate::binding::{KeyBinding, KeyBindingMetaIndex};
use crate::context::KeyContext;
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
    /// 1. Context depth (deeper contexts win)
    /// 2. Source priority (USER > VIM > BASE > DEFAULT)
    /// 3. Order (later bindings win)
    ///
    /// If a user has disabled a binding with `NoAction`, it will not be returned.
    pub fn bindings_for_input(
        &self,
        input: &[Keystroke],
        context_stack: &[KeyContext],
    ) -> (SmallVec<[KeyBinding; 1]>, bool) {
        let mut matched_bindings = SmallVec::<[(usize, usize, &KeyBinding); 1]>::new();
        let mut pending_bindings = SmallVec::<[(usize, &KeyBinding); 1]>::new();
        let mut no_action_indices = Vec::new();

        // First pass: collect all matches and track NoAction bindings
        for (ix, binding) in self.bindings().enumerate().rev() {
            let Some(depth) = self.binding_enabled(binding, context_stack) else {
                continue;
            };

            let Some(pending) = binding.match_keystrokes(input) else {
                continue;
            };

            if is_no_action(binding.action()) {
                no_action_indices.push((ix, depth, binding));
                continue;
            }

            if !pending {
                matched_bindings.push((depth, ix, binding));
            } else {
                pending_bindings.push((ix, binding));
            }
        }

        // Filter out bindings that are disabled by NoAction
        matched_bindings.retain(|(depth, _ix, binding)| {
            for (_no_action_ix, no_action_depth, no_action_binding) in &no_action_indices {
                // NoAction must match the same keystrokes
                if no_action_binding.keystrokes() != binding.keystrokes() {
                    continue;
                }

                // Check if NoAction applies to this binding
                let no_action_applies = match (binding.predicate(), no_action_binding.predicate()) {
                    (_, None) => true,        // Global NoAction disables everything
                    (None, Some(_)) => false, // Specific NoAction doesn't disable global bindings
                    (Some(pred), Some(no_pred)) => {
                        // NoAction's predicate must be more specific (superset check reversed)
                        // The binding's predicate should be a superset of NoAction's
                        // meaning NoAction is more specific and should disable the binding
                        no_action_depth >= depth && pred.is_superset(no_pred)
                    }
                };

                if no_action_applies {
                    return false; // This binding is disabled
                }
            }
            true
        });

        // Sort by precedence: depth DESC, then meta ASC, then index DESC
        matched_bindings.sort_by(|(depth_a, ix_a, binding_a), (depth_b, ix_b, binding_b)| {
            depth_b
                .cmp(depth_a)
                .then_with(|| {
                    let meta_a = binding_a.metadata().unwrap_or(KeyBindingMetaIndex::DEFAULT);
                    let meta_b = binding_b.metadata().unwrap_or(KeyBindingMetaIndex::DEFAULT);
                    meta_a.cmp(&meta_b)
                })
                .then(ix_b.cmp(ix_a))
        });

        let bindings: SmallVec<[_; 1]> = matched_bindings
            .into_iter()
            .map(|(_, _, binding)| binding.clone())
            .collect();

        // Determine if there are pending matches
        let has_pending = !pending_bindings.is_empty();

        (bindings, has_pending)
    }

    /// Check if the given binding is enabled, given a certain key context stack.
    /// Returns the deepest depth at which the binding matches, or None if it doesn't match.
    fn binding_enabled(&self, binding: &KeyBinding, contexts: &[KeyContext]) -> Option<usize> {
        if let Some(predicate) = binding.predicate() {
            predicate.depth_of(contexts)
        } else {
            // Bindings with no context predicate are enabled at the deepest level
            Some(contexts.len())
        }
    }

    /// Get all bindings for a specific action.
    pub fn bindings_for_action(&self, action: &dyn Action) -> Vec<&KeyBinding> {
        self.bindings
            .iter()
            .filter(|binding| {
                binding.action().partial_eq(action) && !is_no_action(binding.action())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::NoAction;
    use crate::actions;
    use crate::context::KeyBindingContextPredicate;
    use std::sync::Arc;

    actions![ActionAlpha, ActionBeta];

    #[test]
    fn test_keymap_creation() {
        let keymap = Keymap::new();
        assert_eq!(keymap.bindings().count(), 0);
        assert_eq!(keymap.version(), KeymapVersion(0));
    }

    #[test]
    fn test_add_bindings() {
        let mut keymap = Keymap::new();
        let binding = KeyBinding::new(
            vec![Keystroke::parse("cmd-s").unwrap()],
            Box::new(ActionAlpha),
            None,
        );

        keymap.add_bindings(vec![binding]);
        assert_eq!(keymap.bindings().count(), 1);
        assert_eq!(keymap.version(), KeymapVersion(1));
    }

    #[test]
    fn test_binding_enabled() {
        let binding_global = KeyBinding::new(
            vec![Keystroke::parse("cmd-s").unwrap()],
            Box::new(ActionAlpha),
            None,
        );

        let binding_editor = KeyBinding::new(
            vec![Keystroke::parse("cmd-s").unwrap()],
            Box::new(ActionBeta),
            Some(Arc::new(
                KeyBindingContextPredicate::parse("Editor").unwrap(),
            )),
        );

        let keymap = Keymap::new();
        let empty_ctx: Vec<KeyContext> = vec![];
        let editor_ctx = vec![KeyContext::parse("Editor").unwrap()];

        // Global binding enabled everywhere
        assert_eq!(keymap.binding_enabled(&binding_global, &empty_ctx), Some(0));
        assert_eq!(
            keymap.binding_enabled(&binding_global, &editor_ctx),
            Some(1)
        );

        // Editor binding only enabled in Editor context
        assert_eq!(keymap.binding_enabled(&binding_editor, &empty_ctx), None);
        assert_eq!(
            keymap.binding_enabled(&binding_editor, &editor_ctx),
            Some(1)
        );
    }

    #[test]
    fn test_depth_precedence() {
        let bindings = vec![
            KeyBinding::new(
                vec![Keystroke::parse("cmd-s").unwrap()],
                Box::new(ActionAlpha),
                Some(Arc::new(
                    KeyBindingContextPredicate::parse("Workspace").unwrap(),
                )),
            ),
            KeyBinding::new(
                vec![Keystroke::parse("cmd-s").unwrap()],
                Box::new(ActionBeta),
                Some(Arc::new(
                    KeyBindingContextPredicate::parse("Editor").unwrap(),
                )),
            ),
        ];

        let keymap = Keymap::with_bindings(bindings);
        let context_stack = vec![
            KeyContext::parse("Workspace").unwrap(),
            KeyContext::parse("Editor").unwrap(),
        ];

        let (result, pending) =
            keymap.bindings_for_input(&[Keystroke::parse("cmd-s").unwrap()], &context_stack);

        assert!(!pending);
        assert_eq!(result.len(), 2);
        // Editor binding should come first (deeper context)
        assert!(result[0].action().partial_eq(&ActionBeta));
        assert!(result[1].action().partial_eq(&ActionAlpha));
    }

    #[test]
    fn test_no_action_disables_binding() {
        let bindings = vec![
            KeyBinding::new(
                vec![Keystroke::parse("cmd-s").unwrap()],
                Box::new(ActionAlpha),
                Some(Arc::new(
                    KeyBindingContextPredicate::parse("Editor").unwrap(),
                )),
            ),
            KeyBinding::new(
                vec![Keystroke::parse("cmd-s").unwrap()],
                Box::new(NoAction),
                Some(Arc::new(
                    KeyBindingContextPredicate::parse("Editor && mode == full").unwrap(),
                )),
            ),
        ];

        let mut keymap = Keymap::new();
        keymap.add_bindings(bindings);

        // In normal editor, binding is active
        let normal_ctx = vec![KeyContext::parse("Editor").unwrap()];
        let (result, _) =
            keymap.bindings_for_input(&[Keystroke::parse("cmd-s").unwrap()], &normal_ctx);
        assert_eq!(result.len(), 1);
        assert!(result[0].action().partial_eq(&ActionAlpha));

        // In full mode editor, binding is disabled
        let mut full_ctx = KeyContext::parse("Editor").unwrap();
        full_ctx.set("mode", "full");
        let (result, _) =
            keymap.bindings_for_input(&[Keystroke::parse("cmd-s").unwrap()], &[full_ctx]);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_multi_keystroke_pending() {
        let bindings = vec![KeyBinding::new(
            vec![
                Keystroke::parse("cmd-k").unwrap(),
                Keystroke::parse("cmd-t").unwrap(),
            ],
            Box::new(ActionAlpha),
            None,
        )];

        let keymap = Keymap::with_bindings(bindings);
        let empty_ctx: Vec<KeyContext> = vec![];

        // First keystroke: pending
        let (result, pending) =
            keymap.bindings_for_input(&[Keystroke::parse("cmd-k").unwrap()], &empty_ctx);
        assert!(pending);
        assert_eq!(result.len(), 0);

        // Complete sequence: match
        let (result, pending) = keymap.bindings_for_input(
            &[
                Keystroke::parse("cmd-k").unwrap(),
                Keystroke::parse("cmd-t").unwrap(),
            ],
            &empty_ctx,
        );
        assert!(!pending);
        assert_eq!(result.len(), 1);
        assert!(result[0].action().partial_eq(&ActionAlpha));
    }

    #[test]
    fn test_source_precedence() {
        let mut binding_default = KeyBinding::new(
            vec![Keystroke::parse("cmd-s").unwrap()],
            Box::new(ActionAlpha),
            None,
        );
        binding_default.set_meta(KeyBindingMetaIndex::DEFAULT);

        let mut binding_user = KeyBinding::new(
            vec![Keystroke::parse("cmd-s").unwrap()],
            Box::new(ActionBeta),
            None,
        );
        binding_user.set_meta(KeyBindingMetaIndex::USER);

        let keymap = Keymap::with_bindings(vec![binding_default, binding_user]);
        let empty_ctx: Vec<KeyContext> = vec![];

        let (result, _) =
            keymap.bindings_for_input(&[Keystroke::parse("cmd-s").unwrap()], &empty_ctx);

        assert_eq!(result.len(), 2);
        // User binding should come first
        assert!(result[0].action().partial_eq(&ActionBeta));
        assert!(result[1].action().partial_eq(&ActionAlpha));
    }

    #[test]
    fn test_bindings_for_action() {
        let bindings = vec![
            KeyBinding::new(
                vec![Keystroke::parse("cmd-s").unwrap()],
                Box::new(ActionAlpha),
                None,
            ),
            KeyBinding::new(
                vec![Keystroke::parse("ctrl-s").unwrap()],
                Box::new(ActionAlpha),
                None,
            ),
            KeyBinding::new(
                vec![Keystroke::parse("cmd-o").unwrap()],
                Box::new(ActionBeta),
                None,
            ),
        ];

        let keymap = Keymap::with_bindings(bindings);
        let alpha_bindings = keymap.bindings_for_action(&ActionAlpha);

        assert_eq!(alpha_bindings.len(), 2);
    }
}
