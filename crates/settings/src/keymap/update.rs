use super::KeybindSource;
use super::binding_adapter::KeystrokeSpec;
use crate::settings::toml::parse_toml_value_snippet;
use anyhow::{Context, Result};
use toml_edit::{Array, Value, value as toml_value};

const NO_ACTION_NAME: &str = "NoAction";

#[derive(Clone)]
pub enum KeybindUpdateOperation<'a> {
    Replace {
        /// Describes the keybind to create
        source: KeybindUpdateTarget<'a>,
        /// Describes the keybind to remove
        target: KeybindUpdateTarget<'a>,
        target_keybind_source: KeybindSource,
    },
    Add {
        source: KeybindUpdateTarget<'a>,
        from: Option<KeybindUpdateTarget<'a>>,
    },
    Remove {
        target: KeybindUpdateTarget<'a>,
        target_keybind_source: KeybindSource,
    },
}

impl KeybindUpdateOperation<'_> {
    pub fn generate_telemetry(
        &self,
    ) -> (
        // The keybind that is created
        String,
        // The keybinding that was removed
        String,
        // The source of the keybinding
        String,
    ) {
        let (new_binding, removed_binding, source) = match &self {
            KeybindUpdateOperation::Replace {
                source,
                target,
                target_keybind_source,
            } => (Some(source), Some(target), Some(*target_keybind_source)),
            KeybindUpdateOperation::Add { source, .. } => (Some(source), None, None),
            KeybindUpdateOperation::Remove {
                target,
                target_keybind_source,
            } => (None, Some(target), Some(*target_keybind_source)),
        };

        let new_binding = new_binding
            .map(KeybindUpdateTarget::telemetry_string)
            .unwrap_or("null".to_owned());
        let removed_binding = removed_binding
            .map(KeybindUpdateTarget::telemetry_string)
            .unwrap_or("null".to_owned());

        let source = source
            .as_ref()
            .map(KeybindSource::name)
            .map(ToOwned::to_owned)
            .unwrap_or("null".to_owned());

        (new_binding, removed_binding, source)
    }
}

impl<'a> KeybindUpdateOperation<'a> {
    pub fn add(source: KeybindUpdateTarget<'a>) -> Self {
        Self::Add { source, from: None }
    }
}

#[derive(Debug, Clone)]
pub struct KeybindUpdateTarget<'a> {
    pub context: Option<&'a str>,
    pub keystrokes: &'a [KeystrokeSpec],
    pub action_name: &'a str,
    pub action_arguments: Option<&'a str>,
}

impl<'a> KeybindUpdateTarget<'a> {
    pub(crate) fn action_value(&self) -> Result<Value> {
        // NoAction: als String speichern
        if self.action_name.eq_ignore_ascii_case(NO_ACTION_NAME) {
            return Ok(toml_value(NO_ACTION_NAME));
        }

        // Ohne Argumente: einfache String-Aktion
        if self.action_arguments.is_none() || self.action_arguments.as_deref().unwrap().is_empty() {
            return Ok(toml_value(self.action_name));
        }

        // Mit Argumenten: ["ActionName", <args-as-toml>]
        let args_snippet = self.action_arguments.unwrap();
        let args_val = parse_toml_value_snippet(args_snippet)
            .with_context(|| format!("Failed to parse action arguments as TOML: {args_snippet}"))?;

        let mut arr = Array::default();
        arr.push(toml_value(self.action_name));
        arr.push(args_val);
        Ok(Value::Array(arr))
    }

    pub(crate) fn keystrokes_unparsed(&self) -> String {
        let mut keystrokes = String::with_capacity(self.keystrokes.len() * 8);
        for keystroke in self.keystrokes {
            keystrokes.push_str(&keystroke.unparse());
            keystrokes.push(' ');
        }
        keystrokes.pop();
        keystrokes
    }

    fn telemetry_string(&self) -> String {
        format!(
            "action_name: {}, context: {}, action_arguments: {}, keystrokes: {}",
            self.action_name,
            self.context.unwrap_or("global"),
            self.action_arguments.unwrap_or("none"),
            self.keystrokes_unparsed()
        )
    }
}
