//! Konfigurationsstrukturen für den Client-Netzwerkstack.

use network_shared::config::ClientNetworkingConfig;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub networking: ClientNetworkingConfig,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            networking: ClientNetworkingConfig::default(),
        }
    }
}
