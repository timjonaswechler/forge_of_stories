pub mod certificate;
pub mod config;
pub mod connection;
pub mod messaging;
pub mod runtime;
pub mod session;
use color_eyre::Result;

// Re-export wichtiger Module für externe Nutzung

// Hauptfunktionen für externe Verwendung
pub fn start_client(config: ClientConfig, server_address: &str) -> Result<()> {
    println!("🎮 Starting FOS Client...");
    println!("📡 Connecting to: {}", server_address);
    // Hier würde die Client-Logik implementiert
    Ok(())
}

// Placeholder für ClientConfig
pub struct ClientConfig {
    // Konfigurationsfelder
}

impl ClientConfig {
    pub fn from_file(path: &str) -> Result<Self> {
        println!("📄 Loading client config from: {}", path);
        // Hier würde die Konfiguration geladen
        Ok(ClientConfig {})
    }
}
