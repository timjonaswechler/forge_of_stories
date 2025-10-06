#[cfg(feature = "steamworks")]
mod auth;
#[cfg(feature = "steamworks")]
pub mod discovery;
#[cfg(not(feature = "steamworks"))]
pub mod discovery_stub;
#[cfg(feature = "steamworks")]
mod real;
#[cfg(not(feature = "steamworks"))]
mod stub;

#[cfg(feature = "steamworks")]
pub use discovery::{SteamLobbyConfig, SteamLobbyError, SteamLobbyHost};
#[cfg(not(feature = "steamworks"))]
pub use discovery_stub::{SteamLobbyConfig, SteamLobbyError, SteamLobbyHost};
#[cfg(feature = "steamworks")]
pub use real::{SteamServerTransport, SteamServerTransportError};
#[cfg(not(feature = "steamworks"))]
pub use stub::{SteamServerTransport, SteamServerTransportError};
