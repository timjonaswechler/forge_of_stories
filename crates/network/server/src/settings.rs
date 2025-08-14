use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub(crate) struct ServerConfig {
    pub(crate) generic: GenericConfig,
    pub(crate) network: NetworkConfig,
    pub(crate) security: SecurityConfig,
    pub(crate) game: GameConfig,
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
    YES,
    NONE, // local only
}

impl Default for ServerManagmentMode {
    fn default() -> Self {
        // Wähle die Default-Variante je nach Feature:
        #[cfg(feature = "dedicated_server")]
        {
            ServerManagmentMode::YES
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
