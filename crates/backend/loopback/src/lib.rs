#[cfg(feature = "client")]
mod client;
#[cfg(feature = "server")]
mod server;
mod tcp;

#[cfg(feature = "client")]
pub use client::*;
#[cfg(feature = "server")]
pub use server::*;

pub const BYTES_PER_SEC_PERIOD: f64 = 0.1;
use bevy::{app::PluginGroupBuilder, prelude::*};
/// Plugin group for all replicon example backend plugins.
///
/// Contains the following:
/// * [`LoopbackServerPlugin`] - with feature `server`.
/// * [`LoopbackClientPlugin`] - with feature `client`.
pub struct LoopbackBackendPlugins;

impl PluginGroup for LoopbackBackendPlugins {
    fn build(self) -> PluginGroupBuilder {
        let mut group = PluginGroupBuilder::start::<Self>();

        #[cfg(feature = "server")]
        {
            group = group.add(LoopbackServerPlugin);
        }

        #[cfg(feature = "client")]
        {
            group = group.add(LoopbackClientPlugin);
        }

        group
    }
}
