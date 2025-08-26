mod action;
mod base_key;
mod binding_adapter;
mod load_result;
mod source;
mod update;
mod validation;

use super::{
    assets::SettingsAssets,
    keymap::{
        action::KeymapAction, load_result::KeymapFileLoadResult, source::KeybindSource, 
    },
};
use crate::keymap::binding_adapter::BindingSpec;
use serde::Deserialize;
use std::{collections::BTreeMap, fmt::Write, rc::Rc, sync::Arc};
use toml_edit::Value;
use update::KeybindUpdateOperation;
use util::asset_str;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct KeymapFile {
    #[serde(default)]
    pub sections: Vec<KeymapSection>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct KeymapSection {
    #[serde(default)]
    pub context: String,
    #[serde(default)]
    pub bindings: Option<BTreeMap<String, KeymapAction>>,
}

impl KeymapSection {
    pub fn bindings(&self) -> impl DoubleEndedIterator<Item = (&String, &KeymapAction)> {
        self.bindings.iter().flatten()
    }
}

impl KeymapFile {
    pub fn parse(content: &str) -> anyhow::Result<Self> {
        Ok(toml::from_str::<KeymapFile>(content)?)
    }

    pub fn load_asset(
        asset_path: &str,
        source: Option<KeybindSource>,
    ) -> anyhow::Result<Vec<BindingSpec>> {
        match Self::load(asset_str::<SettingsAssets>(asset_path).as_ref()) {
            KeymapFileLoadResult::Success { mut key_bindings } => match source {
                Some(source) => Ok({
                    for key_binding in &mut key_bindings {
                        key_binding.set_meta(source.meta());
                    }
                    key_bindings
                }),
                None => Ok(key_bindings),
            },
            KeymapFileLoadResult::SomeFailedToLoad { error_message, .. } => {
                anyhow::bail!("Error loading built-in keymap \"{asset_path}\": {error_message}",)
            }
            KeymapFileLoadResult::TomlParseFailure { error } => {
                anyhow::bail!("Toml parse error in built-in keymap \"{asset_path}\": {error}")
            }
        }
    }

    pub fn load(content: &str) -> KeymapFileLoadResult {
        // let key_equivalents =
        //     crate::key_equivalents::get_key_equivalents(cx.keyboard_layout().id());

        if content.is_empty() {
            return KeymapFileLoadResult::Success {
                key_bindings: Vec::new(),
            };
        }
        let keymap_file = match Self::parse(content) {
            Ok(keymap_file) => keymap_file,
            Err(error) => {
                return KeymapFileLoadResult::TomlParseFailure { error };
            }
        };

        // Accumulate errors in order to support partial load of user keymap in the presence of
        // errors in context and binding parsing.
        let mut errors = Vec::new();
        let mut key_bindings = Vec::new();

        for KeymapSection {
            context,
            // use_key_equivalents,
            bindings,
        } in keymap_file.0.iter()
        {
            let context_predicate: Option<Rc<KeyBindingContextPredicate>> = if context.is_empty() {
                None
            } else {
                match KeyBindingContextPredicate::parse(context) {
                    Ok(context_predicate) => Some(context_predicate.into()),
                    Err(err) => {
                        // Leading space is to separate from the message indicating which section
                        // the error occurred in.
                        errors.push((
                            context,
                            format!(" Parse error in section `context` field: {}", err),
                        ));
                        continue;
                    }
                }
            };

            let mut section_errors = String::new();

            if let Some(bindings) = bindings {
                for (keystrokes, action) in bindings {
                    let result = Self::load_keybinding(
                        keystrokes,
                        action,
                        context_predicate.clone(),
                        // key_equivalents,
                    );
                    match result {
                        Ok(key_binding) => {
                            key_bindings.push(key_binding);
                        }
                        Err(err) => {
                            let mut lines = err.lines();
                            let mut indented_err = lines.next().unwrap().to_string();
                            for line in lines {
                                indented_err.push_str("  ");
                                indented_err.push_str(line);
                                indented_err.push_str("\n");
                            }
                            write!(
                                section_errors,
                                "\n\n- In binding {}, {indented_err}",
                                MarkdownInlineCode(&format!("\"{}\"", keystrokes))
                            )
                            .unwrap();
                        }
                    }
                }
            }

            if !section_errors.is_empty() {
                errors.push((context, section_errors))
            }
        }

        if errors.is_empty() {
            KeymapFileLoadResult::Success { key_bindings }
        } else {
            let mut error_message = "Errors in user keymap file.\n".to_owned();
            for (context, section_errors) in errors {
                if context.is_empty() {
                    let _ = write!(error_message, "\n\nIn section without context predicate:");
                } else {
                    let _ = write!(
                        error_message,
                        "\n\nIn section with {}:",
                        MarkdownInlineCode(&format!("context = \"{}\"", context))
                    );
                }
                let _ = write!(error_message, "{section_errors}");
            }
            KeymapFileLoadResult::SomeFailedToLoad {
                key_bindings,
                error_message: error_message,
            }
        }
    }

    fn load_keybinding(
        keystrokes: &str,
        action: &KeymapAction,
        context: Option<Rc<KeyBindingContextPredicate>>,
        // cx:App
    ) -> std::result::Result<BindingSpec, String> {
        let (build_result, action_input_string) = match &action.0 {
            Value::Array(items) => {
                if items.len() != 2 {
                    return Err(format!(
                        "expected two-element array of `[name, input]`. \
                        Instead found {}.",
                        MarkdownInlineCode(&action.0.to_string())
                    ));
                }
                None => {
                    return Err(format!(
                        "can't build {} action - it requires input data via [name, input]: {}",
                        MarkdownInlineCode(&format!("\"{}\"", &name)),
                        MarkdownEscaped(&error.to_string())
                    ));
                }
            }
            Value::Null => (Ok("NoAction".boxed_clone()), None),
            _ => {
                return Err(format!(
                    "expected two-element array of `[name, input]`. \
                    Instead found {}.",
                    MarkdownInlineCode(&action.0.to_string())
                ));
            }
        };

        let action = match build_result {
            Ok(action) => action,
            Err(ActionBuildError::NotFound { name }) => {
                return Err(format!(
                    "didn't find an action named {}.",
                    MarkdownInlineCode(&format!("\"{}\"", &name))
                ));
            }
            Err(ActionBuildError::BuildError { name, error }) => match action_input_string {
                Some(action_input_string) => {
                    return Err(format!(
                        "can't build {} action from input value {}: {}",
                        MarkdownInlineCode(&format!("\"{}\"", &name)),
                        MarkdownInlineCode(&action_input_string),
                        MarkdownEscaped(&error.to_string())
                    ));
                }
                None => {
                    return Err(format!(
                        "can't build {} action - it requires input data via [name, input]: {}",
                        MarkdownInlineCode(&format!("\"{}\"", &name)),
                        MarkdownEscaped(&error.to_string())
                    ));
                }
            },
        };

        let key_binding = match BindingSpec::load(
            keystrokes,
            action,
            context,
            // key_equivalents,
            action_input_string.map(SharedString::from),
        ) {
            Ok(key_binding) => key_binding,
            Err(InvalidKeystrokeError { keystroke }) => {
                return Err(format!(
                    "invalid keystroke {}. {}",
                    MarkdownInlineCode(&format!("\"{}\"", &keystroke)),
                    KEYSTROKE_PARSE_EXPECTED_MESSAGE
                ));
            }
        };

        if let Some(validator) = KEY_BINDING_VALIDATORS.get(&key_binding.action().type_id()) {
            match validator.validate(&key_binding) {
                Ok(()) => Ok(key_binding),
                Err(error) => Err(error.0),
            }
        } else {
            Ok(key_binding)
        }
    }

    pub fn sections(&self) -> impl DoubleEndedIterator<Item = &KeymapSection> {
        self.0.iter()
    }

    pub async fn load_keymap_file(fs: &Arc<dyn Fs>) -> Result<String> {
        match fs.load(paths::keymap_file()).await {
            result @ Ok(_) => result,
            Err(err) => {
                if let Some(e) = err.downcast_ref::<std::io::Error>()
                    && e.kind() == std::io::ErrorKind::NotFound
                {
                    return Ok(crate::initial_keymap_content().to_string());
                }
                Err(err)
            }
        }
    }

    pub fn update_keybinding<'a>(
        mut operation: KeybindUpdateOperation<'a>,
        mut keymap_contents: String,
        tab_size: usize,
    ) -> Result<String> {
        match operation {
            // if trying to replace a keybinding that is not user-defined, treat it as an add operation
            KeybindUpdateOperation::Replace {
                target_keybind_source: target_source,
                source,
                target,
            } if target_source != KeybindSource::User => {
                operation = KeybindUpdateOperation::Add {
                    source,
                    from: Some(target),
                };
            }
            // if trying to remove a keybinding that is not user-defined, treat it as creating a binding
            // that binds it to `zed::NoAction`
            KeybindUpdateOperation::Remove {
                target,
                target_keybind_source,
            } if target_keybind_source != KeybindSource::User => {
                let mut source = target.clone();
                source.action_name = "NoAction";
                source.action_arguments.take();
                operation = KeybindUpdateOperation::Add {
                    source,
                    from: Some(target),
                };
            }
            _ => {}
        }

        // Sanity check that keymap contents are valid, even though we only use it for Replace.
        // We don't want to modify the file if it's invalid.
        let keymap = Self::parse(&keymap_contents).context("Failed to parse keymap")?;

        if let KeybindUpdateOperation::Remove { target, .. } = operation {
            let target_action_value = target
                .action_value()
                .context("Failed to generate target action JSON value")?;
            let Some((index, keystrokes_str)) =
                find_binding(&keymap, &target, &target_action_value)
            else {
                anyhow::bail!("Failed to find keybinding to remove");
            };
            let is_only_binding = keymap.0[index]
                .bindings
                .as_ref()
                .is_none_or(|bindings| bindings.len() == 1);
            let key_path: &[&str] = if is_only_binding {
                &[]
            } else {
                &["bindings", keystrokes_str]
            };
            let (replace_range, replace_value) = replace_top_level_array_value_in_json_text(
                &keymap_contents,
                key_path,
                None,
                None,
                index,
                tab_size,
            )
            .context("Failed to remove keybinding")?;
            keymap_contents.replace_range(replace_range, &replace_value);
            return Ok(keymap_contents);
        }

        if let KeybindUpdateOperation::Replace { source, target, .. } = operation {
            let target_action_value = target
                .action_value()
                .context("Failed to generate target action JSON value")?;
            let source_action_value = source
                .action_value()
                .context("Failed to generate source action JSON value")?;

            if let Some((index, keystrokes_str)) =
                find_binding(&keymap, &target, &target_action_value)
            {
                if target.context == source.context {
                    // if we are only changing the keybinding (common case)
                    // not the context, etc. Then just update the binding in place

                    let (replace_range, replace_value) =
                        replace_top_level_array_value_in_json_text(
                            &keymap_contents,
                            &["bindings", keystrokes_str],
                            Some(&source_action_value),
                            Some(&source.keystrokes_unparsed()),
                            index,
                            tab_size,
                        )
                        .context("Failed to replace keybinding")?;
                    keymap_contents.replace_range(replace_range, &replace_value);

                    return Ok(keymap_contents);
                } else if keymap.0[index]
                    .bindings
                    .as_ref()
                    .is_none_or(|bindings| bindings.len() == 1)
                {
                    // if we are replacing the only binding in the section,
                    // just update the section in place, updating the context
                    // and the binding

                    let (replace_range, replace_value) =
                        replace_top_level_array_value_in_json_text(
                            &keymap_contents,
                            &["bindings", keystrokes_str],
                            Some(&source_action_value),
                            Some(&source.keystrokes_unparsed()),
                            index,
                            tab_size,
                        )
                        .context("Failed to replace keybinding")?;
                    keymap_contents.replace_range(replace_range, &replace_value);

                    let (replace_range, replace_value) =
                        replace_top_level_array_value_in_json_text(
                            &keymap_contents,
                            &["context"],
                            source.context.map(Into::into).as_ref(),
                            None,
                            index,
                            tab_size,
                        )
                        .context("Failed to replace keybinding")?;
                    keymap_contents.replace_range(replace_range, &replace_value);
                    return Ok(keymap_contents);
                } else {
                    // if we are replacing one of multiple bindings in a section
                    // with a context change, remove the existing binding from the
                    // section, then treat this operation as an add operation of the
                    // new binding with the updated context.

                    let (replace_range, replace_value) =
                        replace_top_level_array_value_in_json_text(
                            &keymap_contents,
                            &["bindings", keystrokes_str],
                            None,
                            None,
                            index,
                            tab_size,
                        )
                        .context("Failed to replace keybinding")?;
                    keymap_contents.replace_range(replace_range, &replace_value);
                    operation = KeybindUpdateOperation::Add {
                        source,
                        from: Some(target),
                    };
                }
            } else {
                log::warn!(
                    "Failed to find keybinding to update `{:?} -> {}` creating new binding for `{:?} -> {}` instead",
                    target.keystrokes,
                    target_action_value,
                    source.keystrokes,
                    source_action_value,
                );
                operation = KeybindUpdateOperation::Add {
                    source,
                    from: Some(target),
                };
            }
        }

        if let KeybindUpdateOperation::Add {
            source: keybinding,
            from,
        } = operation
        {
            let mut value = serde_json::Map::with_capacity(4);
            if let Some(context) = keybinding.context {
                value.insert("context".to_string(), context.into());
            }
            // let use_key_equivalents = from.and_then(|from| {
            //     let action_value = from.action_value().context("Failed to serialize action value. `use_key_equivalents` on new keybinding may be incorrect.").log_err()?;
            //     let (index, _) = find_binding(&keymap, &from, &action_value)?;
            //     Some(keymap.0[index].use_key_equivalents)
            // }).unwrap_or(false);
            // if use_key_equivalents {
            //     value.insert("use_key_equivalents".to_string(), true.into());
            // }

            value.insert("bindings".to_string(), {
                let mut bindings = serde_json::Map::new();
                let action = keybinding.action_value()?;
                bindings.insert(keybinding.keystrokes_unparsed(), action);
                bindings.into()
            });

            let (replace_range, replace_value) = append_top_level_array_value_in_json_text(
                &keymap_contents,
                &value.into(),
                tab_size,
            )?;
            keymap_contents.replace_range(replace_range, &replace_value);
        }
        return Ok(keymap_contents);

        fn find_binding<'a, 'b>(
            keymap: &'b KeymapFile,
            target: &KeybindUpdateTarget<'a>,
            target_action_value: &Value,
        ) -> Option<(usize, &'b str)> {
            let target_context_parsed =
                KeyBindingContextPredicate::parse(target.context.unwrap_or("")).ok();
            for (index, section) in keymap.sections().enumerate() {
                let section_context_parsed =
                    KeyBindingContextPredicate::parse(&section.context).ok();
                if section_context_parsed != target_context_parsed {
                    continue;
                }
                let Some(bindings) = &section.bindings else {
                    continue;
                };
                for (keystrokes_str, action) in bindings {
                    let Ok(keystrokes) = keystrokes_str
                        .split_whitespace()
                        .map(Keystroke::parse)
                        .collect::<Result<Vec<_>, _>>()
                    else {
                        continue;
                    };
                    if keystrokes.len() != target.keystrokes.len()
                        || !keystrokes
                            .iter()
                            .zip(target.keystrokes)
                            .all(|(a, b)| a.should_match(b))
                    {
                        continue;
                    }
                    if &action.0 != target_action_value {
                        continue;
                    }
                    return Some((index, keystrokes_str));
                }
            }
            None
        }
    }
}
