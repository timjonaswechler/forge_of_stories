#[cfg(feature = "steamworks")]
pub mod discovery;
#[cfg(not(feature = "steamworks"))]
pub mod discovery_stub;
#[cfg(feature = "steamworks")]
mod real;
#[cfg(not(feature = "steamworks"))]
mod stub;

#[cfg(feature = "steamworks")]
pub use real::{SteamClientTransport, SteamTransportError};
#[cfg(not(feature = "steamworks"))]
pub use stub::{SteamClientTransport, SteamTransportError};
