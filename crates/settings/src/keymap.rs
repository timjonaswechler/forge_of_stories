mod action;
mod base_key;
mod binding_adapter;
mod load_result;
mod source;
mod update;
mod validation;

use super::{
    assets::SettingsAssets,
    keymap::{action::KeymapAction, load_result::KeymapFileLoadResult, source::KeybindSource},
};
use crate::keymap::binding_adapter::{
    ActionSpec, BindingSpec, KeyCodeSpec, KeystrokeSpec, Modifiers,
};
use anyhow::Result;
use serde::Deserialize;
use std::{collections::BTreeMap, fmt::Write};
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
    const NO_ACTION_NAME: &str = "NoAction";
    pub fn parse(content: &str) -> anyhow::Result<Self> {
        Ok(toml::from_str::<KeymapFile>(content)?)
    }

    pub fn load_asset(
        asset_path: &str,
        source: Option<KeybindSource>,
    ) -> anyhow::Result<Vec<BindingSpec>> {
        match Self::load(asset_str::<SettingsAssets>(asset_path).as_ref()) {
            KeymapFileLoadResult::Success { mut key_bindings } => {
                if let Some(src) = source {
                    for spec in &mut key_bindings {
                        spec.source = Some(src.name().to_string());
                    }
                }
                Ok(key_bindings)
            }
            KeymapFileLoadResult::SomeFailedToLoad { error_message, .. } => {
                anyhow::bail!("Error loading built-in keymap \"{asset_path}\": {error_message}",)
            }
            KeymapFileLoadResult::TomlParseFailure { error } => {
                anyhow::bail!("Toml parse error in built-in keymap \"{asset_path}\": {error}")
            }
        }
    }

    pub fn load(content: &str) -> KeymapFileLoadResult {
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

        let mut errors = Vec::new();
        let mut key_bindings = Vec::new();

        for section in keymap_file.sections.iter() {
            let context_opt = if section.context.trim().is_empty() {
                None
            } else {
                Some(section.context.clone())
            };

            let mut section_errors = String::new();

            if let Some(bindings) = &section.bindings {
                for (keystrokes, action) in bindings {
                    let result = Self::load_keybinding(keystrokes, action, context_opt.as_deref());
                    match result {
                        Ok(binding) => key_bindings.push(binding),
                        Err(err) => {
                            // Einfache, gut lesbare Fehlermeldung
                            writeln!(
                                &mut section_errors,
                                "- In binding \"{}\", {}",
                                keystrokes, err
                            )
                            .ok();
                        }
                    }
                }
            }

            if !section_errors.is_empty() {
                errors.push((section.context.clone(), section_errors));
            }
        }

        if errors.is_empty() {
            KeymapFileLoadResult::Success { key_bindings }
        } else {
            let mut error_message = String::from("Errors in user keymap file.\n");
            for (context, section_errors) in errors {
                if context.trim().is_empty() {
                    error_message.push_str("\nIn section without context predicate:\n");
                } else {
                    error_message
                        .push_str(&format!("\nIn section with context = \"{}\":\n", context));
                }
                error_message.push_str(&section_errors);
            }
            KeymapFileLoadResult::SomeFailedToLoad {
                key_bindings,
                error_message,
            }
        }
    }

    fn load_keybinding(
        keystrokes: &str,
        action: &KeymapAction,
        context: Option<&str>,
    ) -> std::result::Result<BindingSpec, String> {
        // 1) Action interpretieren
        let action_spec = match &action.0 {
            Value::String(name) => {
                let s = name.value();
                if s.eq_ignore_ascii_case("NoAction") {
                    ActionSpec::NoAction
                } else {
                    ActionSpec::Name(s.to_string())
                }
            }
            Value::Array(items) => {
                if items.len() != 2 {
                    return Err(format!(
                        "expected [name, args], found {}",
                        action.0.to_string()
                    ));
                }
                let name_val = &items[0];
                let args_val = &items[1];
                let name_str = name_val.as_str().ok_or_else(|| {
                    format!(
                        "first element of [name, args] must be a string, found {}",
                        name_val
                    )
                })?;
                if name_str.eq_ignore_ascii_case("NoAction") {
                    ActionSpec::NoAction
                } else {
                    let args = toml_value_from_edit(args_val)
                        .map_err(|e| format!("failed to parse args TOML: {e}"))?;
                    ActionSpec::WithArgs {
                        name: name_str.to_string(),
                        args,
                    }
                }
            }
            other => {
                return Err(format!("invalid action value: {}", other));
            }
        };

        // 2) Keystrokes parsen: Sequenz mit Whitespace getrennt
        let mut seq = Vec::new();
        for token in keystrokes.split_whitespace() {
            let ks = parse_keystroke(token)?;
            seq.push(ks);
        }
        if seq.is_empty() {
            return Err("empty keystrokes".into());
        }

        Ok(BindingSpec {
            source: None,
            context: context.map(|s| s.to_string()),
            keystrokes: seq,
            action: action_spec,
        })
    }

    pub fn sections(&self) -> impl DoubleEndedIterator<Item = &KeymapSection> {
        self.sections.iter()
    }

    // paths wird noch erstellt diese Funktion braucht keine Änderung
    pub async fn load_keymap_file() -> Result<String> {
        let path = keymap_file();
        match tokio::fs::read_to_string(&path).await {
            Ok(s) => Ok(s),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                Ok(crate::assets::default_keymap().to_string())
            }
            Err(err) => Err(err.into()),
        }
    }

    pub fn update_keybinding<'a>(
        mut operation: KeybindUpdateOperation<'a>,
        mut keymap_contents: String,
        _tab_size: usize,
    ) -> Result<String> {
        use toml_edit::{ArrayOfTables, DocumentMut, Item, Table};

        // Nicht-User-Replace/Remove in Add/NoAction umbiegen (wie zuvor)
        match operation {
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

        // Validierung: Parse per serde (semantisch), brich bei Fehlern ab
        let _ = Self::parse(&keymap_contents).context("Failed to parse keymap")?;

        // Jetzt format-preservierend mit toml_edit arbeiten
        let mut doc: DocumentMut = keymap_contents
            .parse()
            .unwrap_or_else(|_| DocumentMut::new());

        // Hilfsfunktionen für Sections/Bindings
        fn ensure_sections<'d>(doc: &'d mut DocumentMut) -> &'d mut ArrayOfTables {
            if !doc["sections"].is_array_of_tables() {
                doc["sections"] = Item::ArrayOfTables(ArrayOfTables::new());
            }
            doc["sections"].as_array_of_tables_mut().unwrap()
        }

        fn section_context(tbl: &Table) -> String {
            tbl.get("context")
                .and_then(|i| i.as_str())
                .unwrap_or("")
                .to_string()
        }

        fn find_section_index<'a>(aot: &'a ArrayOfTables, wanted: Option<&str>) -> Option<usize> {
            let want = wanted.unwrap_or("").trim();
            aot.iter().position(|t| section_context(t) == want)
        }

        fn ensure_bindings_table<'t>(tbl: &'t mut Table) -> &'t mut Table {
            if !tbl.contains_key("bindings") || !tbl["bindings"].is_table_like() {
                tbl["bindings"] = Item::Table(Table::new());
            }
            tbl["bindings"].as_table_mut().unwrap()
        }

        // String der Keystrokes aus Target/Source
        let mut doc_changed = false;

        match operation {
            KeybindUpdateOperation::Remove { target, .. } => {
                let keymap = ensure_sections(&mut doc);
                let target_context = target.context;
                let target_key = target.keystrokes_unparsed();

                if let Some(ix) = find_section_index(keymap, target_context) {
                    let tbl = keymap.get_mut(ix).expect("checked index");
                    if let Some(bindings) =
                        tbl.get_mut("bindings").and_then(|i| i.as_table_like_mut())
                    {
                        if bindings.remove(&target_key).is_some() {
                            doc_changed = true;

                            // Optional: leere Section entfernen
                            let empty = bindings.is_empty()
                                && tbl.iter().all(|(k, _)| k == "context" || k == "bindings");
                            if empty {
                                // ArrayOfTables bietet kein remove by index; Workaround: rebuild
                                let mut new_aot = ArrayOfTables::new();
                                for (j, t) in keymap.iter().enumerate() {
                                    if j != ix {
                                        new_aot.push(t.clone());
                                    }
                                }
                                doc["sections"] = Item::ArrayOfTables(new_aot);
                            }
                        }
                    }
                }
            }

            KeybindUpdateOperation::Replace { source, target, .. } => {
                // 1) erst den alten Binding entfernen
                {
                    let keymap = ensure_sections(&mut doc);
                    let target_context = target.context;
                    let target_key = target.keystrokes_unparsed();

                    if let Some(ix) = find_section_index(keymap, target_context) {
                        let tbl = keymap.get_mut(ix).expect("checked index");
                        if let Some(bindings) =
                            tbl.get_mut("bindings").and_then(|i| i.as_table_like_mut())
                        {
                            if bindings.remove(&target_key).is_some() {
                                doc_changed = true;
                            }
                        }
                    }
                }

                // 2) neuen Binding hinzufügen
                {
                    let keymap = ensure_sections(&mut doc);
                    let src_context = source.context;
                    let src_key = source.keystrokes_unparsed();
                    let action_val = source
                        .action_value()
                        .context("Failed to generate source action TOML value")?;

                    let ix = if let Some(ix) = find_section_index(keymap, src_context) {
                        ix
                    } else {
                        keymap.push(Table::new());
                        let ix = keymap.len() - 1;
                        if let Some(ctx) = src_context {
                            keymap[ix]["context"] = Item::Value(toml_edit::value(ctx));
                        }
                        ix
                    };

                    let tbl = keymap.get_mut(ix).expect("checked index");
                    let bindings = ensure_bindings_table(tbl);
                    bindings.insert(&src_key, Item::Value(action_val));
                    doc_changed = true;
                }
            }

            KeybindUpdateOperation::Add { source, .. } => {
                let keymap = ensure_sections(&mut doc);
                let src_context = source.context;
                let src_key = source.keystrokes_unparsed();
                let action_val = source
                    .action_value()
                    .context("Failed to serialize action TOML value")?;

                let ix = if let Some(ix) = find_section_index(keymap, src_context) {
                    ix
                } else {
                    keymap.push(Table::new());
                    let ix = keymap.len() - 1;
                    if let Some(ctx) = src_context {
                        keymap[ix]["context"] = Item::Value(toml_edit::value(ctx));
                    }
                    ix
                };

                let tbl = keymap.get_mut(ix).expect("checked index");
                let bindings = ensure_bindings_table(tbl);
                bindings.insert(&src_key, Item::Value(action_val));
                doc_changed = true;
            }
        }

        if doc_changed {
            keymap_contents = doc.to_string();
        }
        Ok(keymap_contents)
    }
}
fn keymap_file() -> std::path::PathBuf {
    paths::data_dir().join("keymap.toml")
}

fn toml_value_from_edit(v: &toml_edit::Value) -> Result<toml::Value, String> {
    let doc_str = format!("__v__ = {}", v.to_string());
    let table: toml::value::Table =
        toml::from_str(&doc_str).map_err(|e| format!("TOML parse error: {e}"))?;
    table
        .get("__v__")
        .cloned()
        .ok_or_else(|| "failed to extract value".into())
}

// "ctrl-shift-p" oder "ctrl+shift+p"
fn parse_keystroke(token: &str) -> Result<KeystrokeSpec, String> {
    let norm = token.replace('-', "+");
    let mut parts = norm.split('+').filter(|s| !s.is_empty()).peekable();
    let mut mods = Modifiers::empty();
    let mut key_part: Option<&str> = None;

    while let Some(p) = parts.next() {
        if parts.peek().is_some() {
            match p.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => mods.insert(Modifiers::CTRL),
                "alt" => mods.insert(Modifiers::ALT),
                "shift" => mods.insert(Modifiers::SHIFT),
                "cmd" | "super" | "win" => mods.insert(Modifiers::SUPER),
                other => {
                    // Unbekannter Modifier: wir ignorieren ihn bewusst
                    log::warn!("unknown modifier in keystroke: {}", other);
                }
            }
        } else {
            key_part = Some(p);
        }
    }

    let key = key_part.ok_or_else(|| "missing key".to_string())?;
    let key_code = if key.chars().count() == 1 {
        KeyCodeSpec::Char(key.chars().next().unwrap())
    } else {
        KeyCodeSpec::Named(key.to_string())
    };

    Ok(KeystrokeSpec {
        mods,
        key: key_code,
    })
}
