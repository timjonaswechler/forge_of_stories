//! Shared types and helpers for Steam-based networking transports.

use std::time::Duration;

/// Timeout to wait for Steam relay connections before giving up.
pub const RELAY_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Size of the temporary buffer used when reading Steam networking messages.
pub const MAX_STEAM_PACKET_SIZE: usize = 1_200;

/// Identifier used to tag messages transmitted via Steam transport.
pub const STEAM_CHANNEL_CONTROL: u8 = 0;

/// Wrapper around Steamworks App ID to make intent explicit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SteamAppId(pub u32);

impl SteamAppId {
    pub const fn development() -> Self {
        Self(super::STEAM_APP_ID)
    }
}
