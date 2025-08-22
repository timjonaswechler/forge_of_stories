#![allow(dead_code)] // Remove this once you start using the code

use std::{collections::HashMap, env, path::PathBuf};

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use derive_deref::{Deref, DerefMut};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use serde::{Deserialize, de::Deserializer};
use serde_json::Value;
use tracing::error;

use crate::action::Action;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub data_dir: PathBuf,
    #[serde(default)]
    pub config_dir: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default, flatten)]
    pub config: AppConfig,
    // #[serde(default)]
    // pub keybindings: KeyBindings,
}

lazy_static! {
    pub static ref PROJECT_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
    pub static ref DATA_FOLDER: Option<PathBuf> =
        env::var(format!("{}_DATA", PROJECT_NAME.clone()))
            .ok()
            .map(PathBuf::from);
    pub static ref CONFIG_FOLDER: Option<PathBuf> =
        env::var(format!("{}_CONFIG", PROJECT_NAME.clone()))
            .ok()
            .map(PathBuf::from);
}

impl Config {
    pub fn new() -> Result<Self, config::ConfigError> {
        // let default_config: Config = json5::from_str(CONFIG).unwrap();
        let data_dir = get_data_dir();
        let config_dir = get_config_dir();
        let mut builder = config::Config::builder()
            .set_default("data_dir", data_dir.to_str().unwrap())?
            .set_default("config_dir", config_dir.to_str().unwrap())?;

        let config_files = [
            ("config.json5", config::FileFormat::Json5),
            ("config.toml", config::FileFormat::Toml),
        ];
        let mut found_config = false;
        for (file, format) in &config_files {
            let source = config::File::from(config_dir.join(file))
                .format(*format)
                .required(false);
            builder = builder.add_source(source);
            if config_dir.join(file).exists() {
                found_config = true
            }
        }
        if !found_config {
            error!("No configuration file found. Application may not behave as expected");
        }

        let mut cfg: Self = builder.build()?.try_deserialize()?;

        // Merge default keybindings: keys are strings (page ids or scope names)
        // for (page, default_bindings) in default_config.keybindings.0.iter() {
        //     let user_bindings = cfg.keybindings.0.entry(page.clone()).or_default();
        //     for (key, cmd) in default_bindings.iter() {
        //         user_bindings
        //             .entry(key.clone())
        //             .or_insert_with(|| cmd.clone());
        //     }
        // }

        Ok(cfg)
    }
}

pub fn get_data_dir() -> PathBuf {
    let directory = if let Some(s) = DATA_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".data")
    };
    directory
}

pub fn get_config_dir() -> PathBuf {
    let directory = if let Some(s) = CONFIG_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.config_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".config")
    };
    directory
}

fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "chicken105", env!("CARGO_PKG_NAME"))
}

// #[derive(Clone, Debug, Default, Deref, DerefMut)]
// pub struct KeyBindings(pub HashMap<String, HashMap<Vec<KeyEvent>, Action>>);

// impl KeyBindings {
//     /// Return bindings for an optional page name.
//     /// If `page` is Some(page_id) we return bindings for that page scope.
//     /// If `page` is None, there is no fallback to Mode (Mode removed).
//     pub fn get(&self, page: Option<&str>) -> Option<&HashMap<Vec<KeyEvent>, Action>> {
//         if let Some(p) = page {
//             self.0.get(p)
//         } else {
//             None
//         }
//     }

//     /// Return bindings by page id/name.
//     pub fn get_by_name(&self, name: &str) -> Option<&HashMap<Vec<KeyEvent>, Action>> {
//         self.0.get(name)
//     }

//     /// Merge and return bindings for the given context.
//     ///
//     /// Scopes resolved in increasing specificity (earlier inserted, later overrides):
//     /// 1) "*" (wildcard)
//     /// 2) "global"
//     /// 3) page scope by simple name (e.g. "login")
//     /// 4) page-qualified scope "page:<name>"
//     /// 5) component-qualified scope "component:<name>"
//     /// 6) page+component scope "page:<name>/component:<component>"
//     ///
//     /// The resulting map contains the effective bindings for the given page/component.
//     pub fn get_scoped(
//         &self,
//         page: Option<&str>,
//         component: Option<&str>,
//     ) -> HashMap<Vec<KeyEvent>, Action> {
//         let mut merged: HashMap<Vec<KeyEvent>, Action> = HashMap::new();

//         // 1) Wildcard scope "*"
//         if let Some(m) = self.0.get("*") {
//             for (k, v) in m.iter() {
//                 merged.insert(k.clone(), v.clone());
//             }
//         }

//         // 2) Global
//         if let Some(m) = self.0.get("global") {
//             for (k, v) in m.iter() {
//                 merged.insert(k.clone(), v.clone());
//             }
//         }

//         // 3) Page by simple name (back-compat)
//         if let Some(p) = page {
//             if let Some(m) = self.0.get(p) {
//                 for (k, v) in m.iter() {
//                     merged.insert(k.clone(), v.clone());
//                 }
//             }
//         }

//         // 4) Page-qualified "page:<name>"
//         if let Some(p) = page {
//             let key = format!("page:{p}");
//             if let Some(m) = self.0.get(&key) {
//                 for (k, v) in m.iter() {
//                     merged.insert(k.clone(), v.clone());
//                 }
//             }
//         }

//         // 5) Component-qualified "component:<name>"
//         if let Some(c) = component {
//             let key = format!("component:{c}");
//             if let Some(m) = self.0.get(&key) {
//                 for (k, v) in m.iter() {
//                     merged.insert(k.clone(), v.clone());
//                 }
//             }
//         }

//         // 6) Page+Component scope "page:<page>/component:<component>"
//         if let (Some(p), Some(c)) = (page, component) {
//             let key = format!("page:{p}/component:{c}");
//             if let Some(m) = self.0.get(&key) {
//                 for (k, v) in m.iter() {
//                     merged.insert(k.clone(), v.clone());
//                 }
//             }
//         }
//         merged
//     }
// }

// impl<'de> Deserialize<'de> for KeyBindings {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         // Deserialize into a string-keyed map first (page id -> map of key string -> Value)
//         let parsed_map = HashMap::<String, HashMap<String, Value>>::deserialize(deserializer)?;

//         let mut keybindings: HashMap<String, HashMap<Vec<KeyEvent>, Action>> = HashMap::new();

//         for (scope, inner_map) in parsed_map.into_iter() {
//             let mut converted_inner_map: HashMap<Vec<KeyEvent>, Action> = HashMap::new();
//             for (key_str, val) in inner_map.into_iter() {
//                 // Determine Action from the serde_json::Value:
//                 // - if it's a string, parse short string commands like "navigate:home" or "quit"
//                 // - otherwise, attempt to deserialize the value into Action directly
//                 let action: Action = match val {
//                     Value::String(s) => parse_action_from_str(&s).map_err(|e| {
//                         serde::de::Error::custom(format!("invalid action string `{}`: {}", s, e))
//                     })?,
//                     other => serde_json::from_value(other).map_err(|e| {
//                         serde::de::Error::custom(format!(
//                             "failed to deserialize Action from value: {}",
//                             e
//                         ))
//                     })?,
//                 };

//                 let key_seq = parse_key_sequence(&key_str).unwrap_or_default();
//                 converted_inner_map.insert(key_seq, action);
//             }
//             keybindings.insert(scope, converted_inner_map);
//         }

//         Ok(KeyBindings(keybindings))
//     }
// }

// /// Parse user-friendly string representations of actions (used in config files).
// /// Supported forms (case-insensitive):
// /// - "navigate:page_id" or "navigate page_id" -> Action::Navigate(page_id)
// /// - "quit" -> Action::Quit
// /// - "suspend" -> Action::Suspend
// /// - "resume" -> Action::Resume
// /// - "clearscreen" or "clear" -> Action::ClearScreen
// /// - "render" -> Action::Render
// /// - "tick" -> Action::Tick
// /// - "help" -> Action::Help
// /// - "error:<message>" -> Action::Error(message)
// fn parse_action_from_str(s: &str) -> Result<Action, String> {
//     let s_trim = s.trim();
//     let s_lower = s_trim.to_ascii_lowercase();

//     if s_lower.starts_with("navigate:") {
//         let rest = s_trim[s_trim.find(':').unwrap() + 1..].trim().to_string();
//         return Ok(Action::Navigate(rest));
//     }
//     if s_lower.starts_with("navigate ") {
//         let rest = s_trim[s_trim.find(' ').unwrap() + 1..].trim().to_string();
//         return Ok(Action::Navigate(rest));
//     }
//     match s_lower.as_str() {
//         "quit" | "q" => Ok(Action::Quit),
//         "suspend" => Ok(Action::Suspend),
//         "resume" => Ok(Action::Resume),
//         "clearscreen" | "clear" => Ok(Action::ClearScreen),
//         "render" => Ok(Action::Render),
//         "tick" => Ok(Action::Tick),
//         "help" => Ok(Action::Help),
//         "reload" => Ok(Action::Reload),
//         "save" => Ok(Action::Save),
//         _ => {
//             // support error:message
//             if s_lower.starts_with("error:") {
//                 let msg = s_trim[s_trim.find(':').unwrap() + 1..].trim().to_string();
//                 return Ok(Action::Error(msg));
//             }
//             Err(format!("unrecognized action string `{}`", s))
//         }
//     }
// }

// fn parse_key_event(raw: &str) -> Result<KeyEvent, String> {
//     let raw_lower = raw.to_ascii_lowercase();
//     let (remaining, modifiers) = extract_modifiers(&raw_lower);
//     parse_key_code_with_modifiers(remaining, modifiers)
// }

// fn extract_modifiers(raw: &str) -> (&str, KeyModifiers) {
//     let mut modifiers = KeyModifiers::empty();
//     let mut current = raw;

//     // current is already lowercased by parse_key_event; accept both "+" and "-" separators
//     loop {
//         if current.starts_with("ctrl-") || current.starts_with("ctrl+") {
//             modifiers.insert(KeyModifiers::CONTROL);
//             current = &current[5..];
//         } else if current.starts_with("alt-") || current.starts_with("alt+") {
//             modifiers.insert(KeyModifiers::ALT);
//             current = &current[4..];
//         } else if current.starts_with("shift-") || current.starts_with("shift+") {
//             modifiers.insert(KeyModifiers::SHIFT);
//             current = &current[6..];
//         } else {
//             break;
//         }
//     }

//     (current, modifiers)
// }

// fn parse_key_code_with_modifiers(
//     raw: &str,
//     mut modifiers: KeyModifiers,
// ) -> Result<KeyEvent, String> {
//     let c = match raw {
//         "esc" => KeyCode::Esc,
//         "enter" => KeyCode::Enter,
//         "left" => KeyCode::Left,
//         "right" => KeyCode::Right,
//         "up" => KeyCode::Up,
//         "down" => KeyCode::Down,
//         "home" => KeyCode::Home,
//         "end" => KeyCode::End,
//         "pageup" => KeyCode::PageUp,
//         "pagedown" => KeyCode::PageDown,
//         "backtab" => {
//             modifiers.insert(KeyModifiers::SHIFT);
//             KeyCode::BackTab
//         }
//         "backspace" => KeyCode::Backspace,
//         "delete" => KeyCode::Delete,
//         "insert" => KeyCode::Insert,
//         "f1" => KeyCode::F(1),
//         "f2" => KeyCode::F(2),
//         "f3" => KeyCode::F(3),
//         "f4" => KeyCode::F(4),
//         "f5" => KeyCode::F(5),
//         "f6" => KeyCode::F(6),
//         "f7" => KeyCode::F(7),
//         "f8" => KeyCode::F(8),
//         "f9" => KeyCode::F(9),
//         "f10" => KeyCode::F(10),
//         "f11" => KeyCode::F(11),
//         "f12" => KeyCode::F(12),
//         "space" => KeyCode::Char(' '),
//         "hyphen" => KeyCode::Char('-'),
//         "minus" => KeyCode::Char('-'),
//         "tab" => KeyCode::Tab,
//         c if c.len() == 1 => {
//             let mut c = c.chars().next().unwrap();
//             if modifiers.contains(KeyModifiers::SHIFT) {
//                 c = c.to_ascii_uppercase();
//             }
//             KeyCode::Char(c)
//         }
//         _ => return Err(format!("Unable to parse {raw}")),
//     };
//     let ev = KeyEvent::new(c, modifiers);
//     Ok(ev)
// }

// pub fn key_event_to_string(key_event: &KeyEvent) -> String {
//     let char;
//     let key_code = match key_event.code {
//         KeyCode::Backspace => "backspace",
//         KeyCode::Enter => "enter",
//         KeyCode::Left => "left",
//         KeyCode::Right => "right",
//         KeyCode::Up => "up",
//         KeyCode::Down => "down",
//         KeyCode::Home => "home",
//         KeyCode::End => "end",
//         KeyCode::PageUp => "pageup",
//         KeyCode::PageDown => "pagedown",
//         KeyCode::Tab => "tab",
//         KeyCode::BackTab => "backtab",
//         KeyCode::Delete => "delete",
//         KeyCode::Insert => "insert",
//         KeyCode::F(c) => {
//             char = format!("F{c}");
//             &char
//         }
//         KeyCode::Char(' ') => "space",
//         KeyCode::Char(c) => {
//             char = c.to_string();
//             &char
//         }
//         KeyCode::Esc => "esc",
//         KeyCode::Null => "",
//         KeyCode::CapsLock => "",
//         KeyCode::Menu => "",
//         KeyCode::ScrollLock => "",
//         KeyCode::Media(_) => "",
//         KeyCode::NumLock => "",
//         KeyCode::PrintScreen => "",
//         KeyCode::Pause => "",
//         KeyCode::KeypadBegin => "",
//         KeyCode::Modifier(_) => "",
//     };

//     let mut modifiers = Vec::with_capacity(3);

//     if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
//         modifiers.push("Ctrl");
//     }

//     if key_event.modifiers.intersects(KeyModifiers::SHIFT) {
//         modifiers.push("Shift");
//     }

//     if key_event.modifiers.intersects(KeyModifiers::ALT) {
//         modifiers.push("Alt");
//     }

//     let mut key = modifiers.join("+");

//     if !key.is_empty() {
//         key.push('+');
//     }
//     key.push_str(key_code);

//     key
// }

// pub fn parse_key_sequence(raw: &str) -> Result<Vec<KeyEvent>, String> {
//     if raw.chars().filter(|c| *c == '>').count() != raw.chars().filter(|c| *c == '<').count() {
//         return Err(format!("Unable to parse `{}`", raw));
//     }
//     let raw = if !raw.contains("><") {
//         let raw = raw.strip_prefix('<').unwrap_or(raw);
//         let raw = raw.strip_prefix('>').unwrap_or(raw);
//         raw
//     } else {
//         raw
//     };
//     let sequences = raw
//         .split("><")
//         .map(|seq| {
//             if let Some(s) = seq.strip_prefix('<') {
//                 s
//             } else if let Some(s) = seq.strip_suffix('>') {
//                 s
//             } else {
//                 seq
//             }
//         })
//         .collect::<Vec<_>>();

//     sequences.into_iter().map(parse_key_event).collect()
// }
