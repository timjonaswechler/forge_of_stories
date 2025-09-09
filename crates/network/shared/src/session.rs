//! Session identity & helpers shared between client and server.
//!
//! Goals:
//! - Provide a compact 16-byte session identifier (`SessionId`).
//! - Human friendly (hex) Display / parsing.
//! - Constructable without external RNG dependency (monotonic uniqueness heuristic).
//! - (Future) Optional cryptographically secure constructor (feature-gated).
//!
//! Design Notes:
//! - We avoid adding a `rand` dependency right now. `SessionId::new()` uses a mix of
//!   (unix_time_nanos ^ incrementing_counter) to produce 128 bits. This is *not*
//!   cryptographically secure, but is sufficient for temporary uniqueness in the
//!   handshake prototype.
//! - For production-grade guarantees / resistance against guessing, introduce
//!   a `secure` feature that uses `rand::rngs::OsRng` or `getrandom` directly.
//!
//! Future Extensions:
//! - Embed shard / node identifier bits (for multi-node clustering).
//! - Distinguish "ephemeral" vs. "persistent" IDs via high nibble tagging.
//! - Provide ULID-style sortable encoding if ordering becomes relevant.

use core::{
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);

/// 16-byte session identifier.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionId([u8; 16]);

impl SessionId {
    /// Creates a new pseudo-random (collision-resistant for practical purposes) SessionId.
    ///
    /// Not cryptographically secure. For cryptographic randomness, introduce a
    /// feature `secure` later and implement an alternate constructor.
    pub fn new() -> Self {
        let counter = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed) as u128;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        // Simple mixing: xor time with rotated counter.
        let mixed = now ^ counter.rotate_left(17);
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&mixed.to_le_bytes());
        SessionId(bytes)
    }

    /// Returns the inner 16-byte array reference.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Returns the inner array by value.
    pub const fn into_bytes(self) -> [u8; 16] {
        self.0
    }

    /// Creates a SessionId from raw bytes (no validation).
    pub const fn from_bytes(b: [u8; 16]) -> Self {
        SessionId(b)
    }

    /// Parse from a hex string (strict length = 32 chars).
    pub fn from_hex(s: &str) -> Result<Self, SessionIdParseError> {
        if s.len() != 32 {
            return Err(SessionIdParseError::Length);
        }
        let mut out = [0u8; 16];
        for i in 0..16 {
            let hi = decode_hex_byte(s.as_bytes()[2 * i]).ok_or(SessionIdParseError::Hex)?;
            let lo = decode_hex_byte(s.as_bytes()[2 * i + 1]).ok_or(SessionIdParseError::Hex)?;
            out[i] = (hi << 4) | lo;
        }
        Ok(SessionId(out))
    }

    /// Encode to lowercase hex (32 chars).
    pub fn to_hex(&self) -> String {
        let mut s = String::with_capacity(32);
        for b in &self.0 {
            use core::fmt::Write;
            let _ = write!(s, "{:02x}", b);
        }
        s
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

impl fmt::Debug for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Compact debug form: SessionId(0123abcd...)
        write!(f, "SessionId(")?;
        fmt::Display::fmt(self, f)?;
        write!(f, ")")
    }
}

impl From<[u8; 16]> for SessionId {
    fn from(v: [u8; 16]) -> Self {
        SessionId(v)
    }
}

impl From<SessionId> for [u8; 16] {
    fn from(id: SessionId) -> Self {
        id.0
    }
}

impl AsRef<[u8]> for SessionId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Borrow<[u8; 16]> for SessionId {
    fn borrow(&self) -> &[u8; 16] {
        &self.0
    }
}

impl Hash for SessionId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.0);
    }
}

impl FromStr for SessionId {
    type Err = SessionIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SessionId::from_hex(s)
    }
}

/// Parsing errors for SessionId hex representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionIdParseError {
    Length,
    Hex,
}

impl fmt::Display for SessionIdParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionIdParseError::Length => write!(f, "invalid length (expected 32 hex chars)"),
            SessionIdParseError::Hex => write!(f, "invalid hex character"),
        }
    }
}

impl std::error::Error for SessionIdParseError {}

fn decode_hex_byte(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(10 + (b - b'a')),
        b'A'..=b'F' => Some(10 + (b - b'A')),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        let id = SessionId::new();
        let h = id.to_hex();
        assert_eq!(h.len(), 32);
        let parsed = SessionId::from_hex(&h).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn parse_rejects_bad_length() {
        assert!(matches!(
            SessionId::from_hex("abcd"),
            Err(SessionIdParseError::Length)
        ));
    }

    #[test]
    fn parse_rejects_bad_chars() {
        let mut s = "00zz00zz00zz00zz00zz00zz00zz00zz".to_string();
        assert!(matches!(
            SessionId::from_hex(&s),
            Err(SessionIdParseError::Hex)
        ));
        s = "g".repeat(32); // 'g' not valid hex
        assert!(matches!(
            SessionId::from_hex(&s),
            Err(SessionIdParseError::Hex)
        ));
    }
}
