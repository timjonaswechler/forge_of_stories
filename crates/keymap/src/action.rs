//! Action system for key bindings.
//!
//! Actions represent commands that can be triggered by key bindings.
//! They are type-erased trait objects that can be dispatched to handlers.

use std::any::{Any, TypeId};
use std::fmt;

/// A command that can be triggered by a key binding.
///
/// Actions are type-erased and can carry optional data.
/// They must be cloneable, sendable across threads, and have a unique name.
pub trait Action: Send + Sync + 'static {
    /// Get the static name of this action type.
    fn name(&self) -> &'static str;

    /// Get debug representation of this action.
    fn debug_name(&self) -> String {
        self.name().to_string()
    }

    /// Convert to `&dyn Any` for downcasting.
    fn as_any(&self) -> &dyn Any;

    /// Clone this action into a boxed trait object.
    fn boxed_clone(&self) -> Box<dyn Action>;

    /// Check if two actions are equal by type and value.
    fn partial_eq(&self, other: &dyn Action) -> bool;

    /// Get the TypeId of this action.
    fn action_type_id(&self) -> TypeId {
        Any::type_id(self.as_any())
    }
}

impl Clone for Box<dyn Action> {
    fn clone(&self) -> Self {
        self.boxed_clone()
    }
}

impl fmt::Debug for dyn Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Action({})", self.debug_name())
    }
}

/// A no-op action used to disable key bindings.
///
/// When a key binding is set to `NoAction`, it effectively disables
/// that key combination in the current context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoAction;

impl Action for NoAction {
    fn name(&self) -> &'static str {
        "no_action"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn boxed_clone(&self) -> Box<dyn Action> {
        Box::new(self.clone())
    }

    fn partial_eq(&self, other: &dyn Action) -> bool {
        other.as_any().downcast_ref::<Self>().is_some()
    }
}

/// Helper to check if an action is a NoAction.
pub fn is_no_action(action: &dyn Action) -> bool {
    action.as_any().downcast_ref::<NoAction>().is_some()
}

/// Macro to define a simple action without parameters.
///
/// # Example
/// ```ignore
/// action!(SaveFile);
/// action!(OpenSettings);
/// ```
#[macro_export]
macro_rules! action {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name;

        impl $crate::action::Action for $name {
            fn name(&self) -> &'static str {
                stringify!($name)
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn boxed_clone(&self) -> Box<dyn $crate::action::Action> {
                Box::new(self.clone())
            }

            fn partial_eq(&self, other: &dyn $crate::action::Action) -> bool {
                other.as_any().downcast_ref::<Self>().is_some()
            }
        }
    };
}

/// Macro to define multiple actions at once.
///
/// # Example
/// ```ignore
/// actions![
///     SaveFile,
///     OpenSettings,
///     CloseWindow
/// ];
/// ```
#[macro_export]
macro_rules! actions {
    ($($name:ident),* $(,)?) => {
        $(
            $crate::action!($name);
        )*
    };
}

/// Macro to define an action with data.
///
/// # Example
/// ```ignore
/// action_with_data!(
///     MoveToLine {
///         line: usize
///     }
/// );
/// ```
#[macro_export]
macro_rules! action_with_data {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name {
            $(pub $field: $ty),*
        }

        impl $crate::action::Action for $name {
            fn name(&self) -> &'static str {
                stringify!($name)
            }

            fn debug_name(&self) -> String {
                format!("{}({:?})", stringify!($name), self)
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn boxed_clone(&self) -> Box<dyn $crate::action::Action> {
                Box::new(self.clone())
            }

            fn partial_eq(&self, other: &dyn $crate::action::Action) -> bool {
                other
                    .as_any()
                    .downcast_ref::<Self>()
                    .map(|other| self == other)
                    .unwrap_or(false)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    action!(TestAction);
    action!(AnotherAction);

    actions![MultiAction1, MultiAction2, MultiAction3];

    action_with_data!(DataAction {
        value: i32,
        text: String
    });

    #[test]
    fn test_action_names() {
        assert_eq!(TestAction.name(), "TestAction");
        assert_eq!(AnotherAction.name(), "AnotherAction");
        assert_eq!(NoAction.name(), "no_action");
    }

    #[test]
    fn test_action_equality() {
        let action1 = TestAction;
        let action2 = TestAction;
        let action3 = AnotherAction;

        assert!(action1.partial_eq(&action2));
        assert!(!action1.partial_eq(&action3));
    }

    #[test]
    fn test_data_action() {
        let action1 = DataAction {
            value: 42,
            text: "hello".to_string(),
        };
        let action2 = DataAction {
            value: 42,
            text: "hello".to_string(),
        };
        let action3 = DataAction {
            value: 99,
            text: "world".to_string(),
        };

        assert!(action1.partial_eq(&action2));
        assert!(!action1.partial_eq(&action3));
    }

    #[test]
    fn test_boxed_action() {
        let action: Box<dyn Action> = Box::new(TestAction);
        let cloned = action.clone();

        assert_eq!(action.name(), cloned.name());
        assert!(action.partial_eq(&*cloned));
    }

    #[test]
    fn test_no_action() {
        let no_action: Box<dyn Action> = Box::new(NoAction);
        assert!(is_no_action(&*no_action));

        let regular_action: Box<dyn Action> = Box::new(TestAction);
        assert!(!is_no_action(&*regular_action));
    }

    #[test]
    fn test_multi_actions() {
        assert_eq!(MultiAction1.name(), "MultiAction1");
        assert_eq!(MultiAction2.name(), "MultiAction2");
        assert_eq!(MultiAction3.name(), "MultiAction3");
    }

    #[test]
    fn test_action_type_id() {
        let action1 = TestAction;
        let action2 = AnotherAction;

        assert_eq!(action1.action_type_id(), TypeId::of::<TestAction>());
        assert_ne!(action1.action_type_id(), action2.action_type_id());
    }
}
