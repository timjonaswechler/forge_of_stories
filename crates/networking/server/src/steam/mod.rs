// TODO: Re-enable the real Steam transport once migrated to the new polling API.
#[cfg(feature = "steamworks")]
mod auth;
#[cfg(feature = "steamworks")]
pub mod discovery;
#[cfg(not(feature = "steamworks"))]
pub mod discovery_stub;
mod stub;

#[cfg(feature = "steamworks")]
pub use discovery::{SteamLobbyConfig, SteamLobbyError, SteamLobbyHost};
#[cfg(not(feature = "steamworks"))]
pub use discovery_stub::{SteamLobbyConfig, SteamLobbyError, SteamLobbyHost};
pub use stub::{SteamServerTransport, SteamServerTransportError};
