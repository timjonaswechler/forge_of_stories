//! Konfigurationsstrukturen für den Server-Netzwerkstack.

use network_shared::config::ServerNetworkingConfig;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub networking: ServerNetworkingConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            networking: ServerNetworkingConfig::default(),
        }
    }
}
