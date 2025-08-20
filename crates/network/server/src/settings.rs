use serde::{Deserialize, Serialize};
use settings::SettingSource;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub(crate) struct ServerConfig {
    pub(crate) generic: GenericConfig,
    pub(crate) network: NetworkConfig,
    pub(crate) security: SecurityConfig,
    pub(crate) game: GameConfig,
}
impl ServerConfig {
    fn default() -> Self {
        ServerConfig {
            generic: GenericConfig::default(),
            network: NetworkConfig::default(),
            security: SecurityConfig::default(),
            game: GameConfig::default(),
        }
    }
}
impl SettingSource for ServerConfig {
    fn kind(&self) -> settings::SourceKind {
        settings::SourceKind::Server
    }

    fn precedence(&self) -> i32 {
        40 // Default precedence for server settings
    }

    fn is_writable(&self) -> bool {
        true // Server config is writable
    }

    fn load(&mut self) -> Result<(), settings::errors::SettingError> {
        // In a real implementation, this would load from a file or other source.
        Ok(())
    }

    fn get(
        &self,
        key_path: &settings::KeyPath,
    ) -> Result<Option<toml_edit::Item>, settings::errors::SettingError> {
        // Implement logic to retrieve a setting by key path
        Ok(None)
    }

    fn set(
        &mut self,
        key_path: &settings::KeyPath,
        value: toml_edit::Item,
    ) -> Result<(), settings::errors::SettingError> {
        // Implement logic to set a setting by key path
        Ok(())
    }

    fn persist(&self) -> Result<(), settings::errors::SettingError> {
        // Implement logic to persist the configuration to a file or other storage
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub(crate) struct GenericConfig {
    pub(crate) server_managment_mode: ServerManagmentMode,
}

impl Default for GenericConfig {
    fn default() -> Self {
        GenericConfig {
            server_managment_mode: ServerManagmentMode::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct NetworkConfig {
    pub(crate) quic_port: u16,  // for QUIC Connections
    pub(crate) admin_port: u16, // for Admin Connections
    pub(crate) bind_address: String,
    pub(crate) max_connections: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            quic_port: 4433, //or 28015
            admin_port: 8443,
            bind_address: "127.0.0.1".to_string(),
            max_connections: 100,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct SecurityConfig {
    pub(crate) crl_update_periode: u64, // in seconds
    pub(crate) cert_dir: String,
    pub(crate) ca_name: String,
    pub(crate) client_cert_name: String,
    pub(crate) client_key_name: String,
    pub(crate) crl_name: String,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        SecurityConfig {
            crl_update_periode: 10,
            cert_dir: "certs".to_string(),
            ca_name: "ca-cert.pem".to_string(),
            client_cert_name: "client-cert.pem".to_string(),
            client_key_name: "client-key.pem".to_string(),
            crl_name: "crl.der".to_string(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct GameConfig {
    pub(crate) game_name: String,
    pub(crate) game_version: String,
}
impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            game_name: "Forge of Stories".to_string(),
            game_version: "1.0.0".to_string(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub(crate) enum ServerManagmentMode {
    #[cfg(feature = "dedicated_server")]
    TUI, // TUI for dedicated server management
    #[cfg(feature = "dedicated_server")]
    WEBANDTUI,
    NONE, // local only
}

impl Default for ServerManagmentMode {
    fn default() -> Self {
        // Wähle die Default-Variante je nach Feature:
        #[cfg(feature = "dedicated_server")]
        {
            ServerManagmentMode::TUI
        }
        #[cfg(all(not(feature = "dedicated_server"), feature = "local_server"))]
        {
            ServerManagmentMode::NONE
        }
        #[cfg(all(not(feature = "dedicated_server"), not(feature = "local_server")))]
        {
            ServerManagmentMode::NONE // oder eine andere sinnvolle Default-Variante
        }
    }
}
