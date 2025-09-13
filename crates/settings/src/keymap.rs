use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use toml;

// ---------- Action Registry System ----------

/// Trait für Anwendungen zum Registrieren ihrer Actions
/// Settings arbeitet nur mit String-Action-Names, die App übersetzt zu konkreten Actions
pub trait ActionRegistry {
    /// Der konkrete Action-Type der Anwendung (z.B. crate::action::Action)
    type Action;

    /// Übersetzt einen Action-Namen + optionale Daten zu einer konkreten Action
    fn resolve_action(
        &self,
        action_name: &str,
        action_data: Option<&toml::Value>,
    ) -> Option<Self::Action>;

    /// Liste aller verfügbaren Action-Namen (für Validierung/Autocomplete)
    fn get_action_names(&self) -> Vec<String>;
}

// ---------- TOML Data Structures ----------

/// Root-Struktur der keymap.toml Datei
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeymapFile {
    /// Liste von Binding-Gruppen, spätere haben höhere Priorität
    pub bindings: Vec<BindingGroup>,
}

/// Eine Gruppe von Tastenbelegungen mit gemeinsamem Kontext
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingGroup {
    /// Kontext-String (z.B. "global", "dashboard && !popup-visible")
    pub context: String,
    /// Actions innerhalb dieser Gruppe: action_name -> key_definitions
    pub actions: HashMap<String, Vec<KeyDefinition>>,
}

/// Definition einer einzelnen Tastenbelegung
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDefinition {
    /// Taste als String (z.B. "ctrl+c", "tab", "esc")
    pub key: String,
    /// Optional: spezifisches Gerät (default: Keyboard)
    pub device: Option<DeviceKind>,
    /// Optional: statische Action-Daten als TOML-Value
    pub action_data: Option<toml::Value>,
}

// ---------- Context Predicate System ----------

/// Logische Ausdrücke für Kontext-Bedingungen
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextPredicate {
    /// Einfacher Kontext ist aktiv (z.B. "dashboard")
    Is(String),
    /// Negation eines Prädikats (z.B. "!popup-visible")
    Not(Box<ContextPredicate>),
    /// Logisches UND mehrerer Prädikate
    And(Vec<ContextPredicate>),
    /// Logisches ODER mehrerer Prädikate
    Or(Vec<ContextPredicate>),
}

impl ContextPredicate {
    /// Evaluiert das Prädikat gegen eine Liste aktiver Kontexte
    pub fn eval(&self, active_contexts: &[&str]) -> bool {
        match self {
            ContextPredicate::Is(ctx) => active_contexts.contains(&ctx.as_str()),
            ContextPredicate::Not(pred) => !pred.eval(active_contexts),
            ContextPredicate::And(preds) => preds.iter().all(|p| p.eval(active_contexts)),
            ContextPredicate::Or(preds) => preds.iter().any(|p| p.eval(active_contexts)),
        }
    }

    /// Parst einen Kontext-String zu einem ContextPredicate
    /// Einfacher Parser für "dashboard && !popup-visible" etc.
    pub fn parse(context_str: &str) -> Result<Self, String> {
        // Für jetzt: einfache Implementierung, später erweitern
        let context_str = context_str.trim();

        // Einfacher Fall: nur ein Kontext-Name
        if !context_str.contains("&&")
            && !context_str.contains("||")
            && !context_str.starts_with('!')
        {
            return Ok(ContextPredicate::Is(context_str.to_string()));
        }

        // Negation
        if let Some(inner) = context_str.strip_prefix('!') {
            return Ok(ContextPredicate::Not(Box::new(Self::parse(inner.trim())?)));
        }

        // UND-Verknüpfung
        if context_str.contains("&&") {
            let parts: Result<Vec<_>, _> = context_str
                .split("&&")
                .map(|part| Self::parse(part.trim()))
                .collect();
            return Ok(ContextPredicate::And(parts?));
        }

        // ODER-Verknüpfung
        if context_str.contains("||") {
            let parts: Result<Vec<_>, _> = context_str
                .split("||")
                .map(|part| Self::parse(part.trim()))
                .collect();
            return Ok(ContextPredicate::Or(parts?));
        }

        Err(format!("Could not parse context: {}", context_str))
    }
}

// ---------- Runtime Optimization ----------

/// Aufgelöste Tastenbelegung für optimierte Laufzeit-Abfragen
#[derive(Debug, Clone)]
pub struct ResolvedBinding {
    /// Die normalisierte Tastenkombination
    pub key_chord: String,
    /// Geräteart für diese Binding
    pub device: DeviceKind,
    /// Ausgewertetes Kontext-Prädikat
    pub context: ContextPredicate,
    /// Action-Name (wird von ActionRegistry übersetzt)
    pub action_name: String,
    /// Optional: statische Action-Daten
    pub action_data: Option<toml::Value>,
    /// Priorität (höher = später definiert = gewinnt bei Konflikten)
    pub priority: i32,
}

// ---------- Keymaps: Datenmodell & Merge pro Gerät ----------

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum DeviceKind {
    Keyboard,
    Mouse,
    Gamepad,
}

bitflags::bitflags! {
    #[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Hash, Debug)]
    pub struct Mods: u8 { const CTRL=1; const SHIFT=2; const ALT=4; const META=8; }
}

/// Merkt sich den ursprünglichen Präfix (z. B. "xbox", "dualshock", "mouse")
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct KeyChord {
    pub device: DeviceKind,
    pub mods: Mods,
    pub key: String,
    pub origin_prefix: Option<String>, // <- neu
}

/// Auswahl, welche Eingabeart exportiert werden soll
#[derive(Clone, Debug)]
pub enum DeviceFilter {
    Keyboard,
    Mouse,
    GamepadAny,
    GamepadKind(String), // z. B. "xbox", "dualshock"
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct KeymapMeta {
    pub devices: Vec<DeviceKind>,        // optional: bevorzugte Geräte
    pub version: String,                 //
    pub gamepad_profile: Option<String>, // "xbox" | "dualshock" | ...
    pub mouse_enabled: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct MergedKeymaps {
    pub meta: KeymapMeta,
    // context -> action -> device -> chords
    pub contexts: HashMap<String, HashMap<String, HashMap<DeviceKind, Vec<KeyChord>>>>,
}

#[derive(Clone, Debug, Default)]
pub enum InputScheme {
    #[default]
    KeyboardMouse,
    Gamepad {
        kind: String,
    },
}

#[derive(Clone, Debug, Default)]
pub struct KeymapState {
    pub scheme: InputScheme,
    /// Kompilierte Tastenbelegungen für optimierte Laufzeit-Abfragen
    pub resolved_bindings: Vec<ResolvedBinding>,
}

impl KeymapState {
    /// Kompiliert eine KeymapFile zu optimierten ResolvedBindings
    pub fn compile_keymap(keymap: &KeymapFile) -> Result<Vec<ResolvedBinding>, String> {
        let mut resolved_bindings = Vec::new();
        let mut priority = 0;

        for binding_group in &keymap.bindings {
            let context_predicate = ContextPredicate::parse(&binding_group.context)?;

            for (action_name, key_definitions) in &binding_group.actions {
                for key_def in key_definitions {
                    let resolved_binding = ResolvedBinding {
                        key_chord: key_def.key.clone(),
                        device: key_def.device.unwrap_or(DeviceKind::Keyboard),
                        context: context_predicate.clone(),
                        action_name: action_name.clone(),
                        action_data: key_def.action_data.clone(),
                        priority,
                    };
                    resolved_bindings.push(resolved_binding);
                    priority += 1;
                }
            }
        }

        // Umgekehrte Reihenfolge: höhere Priorität = später definiert = gewinnt bei Konflikten
        resolved_bindings.reverse();
        Ok(resolved_bindings)
    }

    /// Sucht eine passende Action für eine Tastenkombination und aktive Kontexte
    pub fn resolve_action_for_key<R: ActionRegistry>(
        &self,
        key_chord: &str,
        active_contexts: &[String],
        registry: &R,
    ) -> Option<R::Action> {
        let context_refs: Vec<&str> = active_contexts.iter().map(|s| s.as_str()).collect();

        // Suche erste passende Binding (höchste Priorität durch umgekehrte Reihenfolge)
        for binding in &self.resolved_bindings {
            if binding.key_chord.eq_ignore_ascii_case(key_chord)
                && binding.context.eval(&context_refs)
            {
                return registry.resolve_action(&binding.action_name, binding.action_data.as_ref());
            }
        }

        None
    }

    /// Lädt und kompiliert eine keymap.toml Datei
    pub fn load_keymap_from_toml(toml_content: &str) -> Result<Vec<ResolvedBinding>, String> {
        let keymap: KeymapFile = toml::from_str(toml_content)
            .map_err(|e| format!("Failed to parse keymap TOML: {}", e))?;
        Self::compile_keymap(&keymap)
    }
}
