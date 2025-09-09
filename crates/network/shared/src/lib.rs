//! Shared networking types for Forge of Stories.
//!
//! This crate hosts protocol primitives shared between client & server:
//! - messaging: message/frame abstractions (codec, stream) (to be expanded)
//! - protocol: versioning, higher-level protocol enums (empty stub for now)
//! - certificate / pki: TLS & key material helpers (stubs)
//! - session: session identity / metadata (stub)
//!
//! Keep this crate lean (no Bevy by default). Bevy-specific adapters are feature gated.

pub mod certificate;
pub mod messaging;
pub mod pki;
pub mod protocol;
pub mod session;

/// ALPN identifier used during QUIC/TLS handshake.
pub const AETHER_ALPN: &str = "aether/0.1";

/// Supported protocol versions (handshake negotiation baseline).
pub const SUPPORTED_VERSIONS: &[u16] = &[1];

/// Unified handshake error codes shared across client/server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandshakeErrorCode {
    VersionMismatch,
    AuthFailed,
    RateLimited,
    Internal,
}

/// Convenience prelude for downstream crates.
pub mod prelude {
    pub use crate::AETHER_ALPN;
    pub use crate::HandshakeErrorCode;
    pub use crate::SUPPORTED_VERSIONS;
    pub use crate::messaging;
    pub use crate::protocol;
}

/// Bevy-related re-exports (only when the settings crate was built with bevy feature).
#[cfg(feature = "bevy")]
pub mod bevy_integration {
    //! Bevy integration helpers (thin re-export layer).
    pub use settings::bevy_adapter::SettingsArc;
}
