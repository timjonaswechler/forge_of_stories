//! Keystroke parsing and matching.
//!
//! This module handles parsing of keystroke strings like "cmd-s", "ctrl-shift-p",
//! and multi-key sequences like "cmd-k cmd-t".

use anyhow::{Result, bail};
use std::fmt;

/// Modifier keys that can be combined with regular keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub cmd: bool,
}

impl Modifiers {
    /// No modifiers pressed.
    pub const NONE: Self = Self {
        ctrl: false,
        alt: false,
        shift: false,
        cmd: false,
    };

    /// Check if any modifiers are set.
    pub fn is_empty(&self) -> bool {
        !self.ctrl && !self.alt && !self.shift && !self.cmd
    }

    /// Create modifiers from a list of modifier names.
    pub fn from_names(names: &[&str]) -> Result<Self> {
        let mut modifiers = Self::NONE;
        for name in names {
            match *name {
                "ctrl" | "control" => modifiers.ctrl = true,
                "alt" | "option" => modifiers.alt = true,
                "shift" => modifiers.shift = true,
                "cmd" | "command" | "super" => modifiers.cmd = true,
                _ => bail!("unknown modifier: {}", name),
            }
        }
        Ok(modifiers)
    }
}

impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("ctrl");
        }
        if self.alt {
            parts.push("alt");
        }
        if self.shift {
            parts.push("shift");
        }
        if self.cmd {
            parts.push("cmd");
        }
        write!(f, "{}", parts.join("-"))
    }
}

/// A single keystroke (key + modifiers).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Keystroke {
    pub modifiers: Modifiers,
    pub key: String,
}

impl Keystroke {
    /// Create a new keystroke.
    pub fn new(key: impl Into<String>, modifiers: Modifiers) -> Self {
        Self {
            modifiers,
            key: key.into(),
        }
    }

    /// Parse a keystroke from a string like "cmd-s" or "ctrl-shift-p".
    ///
    /// Format: [modifier-]*key
    /// Modifiers: ctrl, alt/option, shift, cmd/command/super
    ///
    /// Examples:
    /// - "s" -> key 's', no modifiers
    /// - "cmd-s" -> key 's', cmd modifier
    /// - "ctrl-shift-p" -> key 'p', ctrl+shift modifiers
    /// - "escape" -> key 'escape', no modifiers
    pub fn parse(input: &str) -> Result<Self> {
        if input.is_empty() {
            bail!("empty keystroke");
        }

        let parts: Vec<&str> = input.split('-').collect();

        if parts.is_empty() {
            bail!("invalid keystroke format");
        }

        // Last part is always the key
        let key = parts.last().unwrap().to_string();

        // Everything before is modifiers
        let modifier_parts = &parts[..parts.len() - 1];
        let modifiers = Modifiers::from_names(modifier_parts)?;

        // Validate key
        if key.is_empty() {
            bail!("empty key in keystroke");
        }

        Ok(Self { modifiers, key })
    }

    /// Check if this keystroke matches another keystroke.
    pub fn matches(&self, other: &Keystroke) -> bool {
        self == other
    }
}

impl fmt::Display for Keystroke {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.modifiers.is_empty() {
            write!(f, "{}", self.key)
        } else {
            write!(f, "{}-{}", self.modifiers, self.key)
        }
    }
}

/// Parse a keystroke sequence like "cmd-k cmd-t".
pub fn parse_keystroke_sequence(input: &str) -> Result<Vec<Keystroke>> {
    input.split_whitespace().map(Keystroke::parse).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_key() {
        let k = Keystroke::parse("s").unwrap();
        assert_eq!(k.key, "s");
        assert_eq!(k.modifiers, Modifiers::NONE);
    }

    #[test]
    fn test_parse_with_single_modifier() {
        let k = Keystroke::parse("cmd-s").unwrap();
        assert_eq!(k.key, "s");
        assert!(k.modifiers.cmd);
        assert!(!k.modifiers.ctrl);
        assert!(!k.modifiers.alt);
        assert!(!k.modifiers.shift);
    }

    #[test]
    fn test_parse_with_multiple_modifiers() {
        let k = Keystroke::parse("ctrl-shift-p").unwrap();
        assert_eq!(k.key, "p");
        assert!(k.modifiers.ctrl);
        assert!(k.modifiers.shift);
        assert!(!k.modifiers.cmd);
        assert!(!k.modifiers.alt);
    }

    #[test]
    fn test_parse_special_keys() {
        let k = Keystroke::parse("escape").unwrap();
        assert_eq!(k.key, "escape");
        assert_eq!(k.modifiers, Modifiers::NONE);

        let k = Keystroke::parse("cmd-enter").unwrap();
        assert_eq!(k.key, "enter");
        assert!(k.modifiers.cmd);
    }

    #[test]
    fn test_parse_sequence() {
        let seq = parse_keystroke_sequence("cmd-k cmd-t").unwrap();
        assert_eq!(seq.len(), 2);
        assert_eq!(seq[0].key, "k");
        assert!(seq[0].modifiers.cmd);
        assert_eq!(seq[1].key, "t");
        assert!(seq[1].modifiers.cmd);
    }

    #[test]
    fn test_to_string() {
        let k = Keystroke::parse("ctrl-shift-p").unwrap();
        assert_eq!(k.to_string(), "ctrl-shift-p");

        let k = Keystroke::parse("s").unwrap();
        assert_eq!(k.to_string(), "s");

        let k = Keystroke::parse("cmd-escape").unwrap();
        assert_eq!(k.to_string(), "cmd-escape");
    }

    #[test]
    fn test_matches() {
        let k1 = Keystroke::parse("cmd-s").unwrap();
        let k2 = Keystroke::parse("cmd-s").unwrap();
        let k3 = Keystroke::parse("ctrl-s").unwrap();

        assert!(k1.matches(&k2));
        assert!(!k1.matches(&k3));
    }

    #[test]
    fn test_modifier_aliases() {
        let k1 = Keystroke::parse("command-s").unwrap();
        let k2 = Keystroke::parse("cmd-s").unwrap();
        let k3 = Keystroke::parse("super-s").unwrap();

        assert_eq!(k1.modifiers, k2.modifiers);
        assert_eq!(k2.modifiers, k3.modifiers);

        let k1 = Keystroke::parse("option-s").unwrap();
        let k2 = Keystroke::parse("alt-s").unwrap();
        assert_eq!(k1.modifiers, k2.modifiers);
    }

    #[test]
    fn test_empty_keystroke() {
        assert!(Keystroke::parse("").is_err());
    }

    #[test]
    fn test_invalid_modifier() {
        assert!(Keystroke::parse("invalid-s").is_err());
    }
}
