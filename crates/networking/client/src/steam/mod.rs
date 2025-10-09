#[cfg(feature = "steamworks")]
pub mod discovery;
#[cfg(not(feature = "steamworks"))]
pub mod discovery_stub;
mod stub;

pub use stub::{SteamClientTransport, SteamTransportError};
