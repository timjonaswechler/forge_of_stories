//! Transport orchestrator for creating and managing network transports.
//!
//! Provides a unified API for instantiating different transport types (Loopback, QUIC, Steam)
//! based on configuration, with proper feature-flag gating.

use super::LoopbackPair;

/// Error types for transport orchestration.
#[derive(Debug, thiserror::Error)]
pub enum OrchestratorError {
    #[error("Transport feature not enabled: {0}")]
    FeatureDisabled(&'static str),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Transport initialization failed: {0}")]
    InitializationFailed(String),
}

/// Transport orchestrator - factory for creating transport instances.
///
/// This struct provides centralized transport creation logic, handling:
/// - Feature-flag gating (e.g., steamworks feature)
/// - Configuration validation
/// - Appropriate transport instantiation
pub struct TransportOrchestrator;

impl TransportOrchestrator {
    /// Creates a new loopback transport pair for in-memory communication.
    ///
    /// This is used for singleplayer mode where client and server run in the
    /// same process with zero network overhead.
    ///
    /// # Returns
    /// A `LoopbackPair` containing both client and server transport halves.
    ///
    /// # Example
    /// ```
    /// use shared::transport::TransportOrchestrator;
    ///
    /// let pair = TransportOrchestrator::create_loopback_pair();
    /// // Use pair.client for client-side
    /// // Use pair.server for server-side
    /// ```
    pub fn create_loopback_pair() -> LoopbackPair {
        LoopbackPair::new()
    }

    /// Creates a QUIC server transport (placeholder for future implementation).
    ///
    /// # Arguments
    /// * `bind_address` - IP address to bind to (e.g., "0.0.0.0" or "127.0.0.1")
    /// * `port` - UDP port to listen on (e.g., 7777)
    ///
    /// # Errors
    /// Returns `OrchestratorError::InitializationFailed` - QUIC transport not yet implemented.
    #[allow(unused_variables)]
    pub fn create_quic_server(bind_address: &str, port: u16) -> Result<(), OrchestratorError> {
        Err(OrchestratorError::InitializationFailed(
            "QUIC server transport not yet implemented in orchestrator".to_string(),
        ))
    }

    /// Creates a QUIC client transport (placeholder for future implementation).
    ///
    /// # Arguments
    /// * `host` - Server hostname or IP address
    /// * `port` - Server port
    ///
    /// # Errors
    /// Returns `OrchestratorError::InitializationFailed` - QUIC transport not yet implemented.
    #[allow(unused_variables)]
    pub fn create_quic_client(host: &str, port: u16) -> Result<(), OrchestratorError> {
        Err(OrchestratorError::InitializationFailed(
            "QUIC client transport not yet implemented in orchestrator".to_string(),
        ))
    }

    /// Creates a Steam server transport (placeholder, feature-gated).
    ///
    /// # Errors
    /// - `OrchestratorError::FeatureDisabled` if steamworks feature is not enabled
    /// - `OrchestratorError::InitializationFailed` if initialization fails
    #[cfg(feature = "steamworks")]
    pub fn create_steam_server() -> Result<(), OrchestratorError> {
        Err(OrchestratorError::InitializationFailed(
            "Steam server transport not yet implemented in orchestrator".to_string(),
        ))
    }

    #[cfg(not(feature = "steamworks"))]
    pub fn create_steam_server() -> Result<(), OrchestratorError> {
        Err(OrchestratorError::FeatureDisabled("steamworks"))
    }

    /// Creates a Steam client transport (placeholder, feature-gated).
    ///
    /// # Errors
    /// - `OrchestratorError::FeatureDisabled` if steamworks feature is not enabled
    /// - `OrchestratorError::InitializationFailed` if initialization fails
    #[cfg(feature = "steamworks")]
    #[allow(unused_variables)]
    pub fn create_steam_client(lobby_id: u64) -> Result<(), OrchestratorError> {
        Err(OrchestratorError::InitializationFailed(
            "Steam client transport not yet implemented in orchestrator".to_string(),
        ))
    }

    #[cfg(not(feature = "steamworks"))]
    #[allow(unused_variables)]
    pub fn create_steam_client(lobby_id: u64) -> Result<(), OrchestratorError> {
        Err(OrchestratorError::FeatureDisabled("steamworks"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_loopback_pair() {
        let pair = TransportOrchestrator::create_loopback_pair();

        // Verify pair was created successfully
        // (The LoopbackPair struct exists and can be used)
        let _ = pair.client;
        let _ = pair.server;
    }

    #[test]
    fn test_quic_server_not_implemented() {
        let result = TransportOrchestrator::create_quic_server("127.0.0.1", 7777);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            OrchestratorError::InitializationFailed(_)
        ));
    }

    #[test]
    fn test_quic_client_not_implemented() {
        let result = TransportOrchestrator::create_quic_client("127.0.0.1", 7777);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            OrchestratorError::InitializationFailed(_)
        ));
    }

    #[test]
    fn test_steam_server_feature_gating() {
        let result = TransportOrchestrator::create_steam_server();
        assert!(result.is_err());

        #[cfg(not(feature = "steamworks"))]
        assert!(matches!(
            result.unwrap_err(),
            OrchestratorError::FeatureDisabled("steamworks")
        ));
    }

    #[test]
    fn test_steam_client_feature_gating() {
        let result = TransportOrchestrator::create_steam_client(12345);
        assert!(result.is_err());

        #[cfg(not(feature = "steamworks"))]
        assert!(matches!(
            result.unwrap_err(),
            OrchestratorError::FeatureDisabled("steamworks")
        ));
    }
}
