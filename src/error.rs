//! Error types for the RPC pool.

use thiserror::Error;

/// Errors that can occur during RPC pool operations.
#[derive(Error, Debug)]
pub enum RpcPoolError {
    /// All configured endpoints have failed.
    #[error("All RPC endpoints failed: {0}")]
    AllEndpointsFailed(String),

    /// No endpoints are configured.
    #[error("No RPC endpoints configured")]
    NoEndpointsConfigured,

    /// No healthy endpoints are available.
    #[error("No healthy RPC endpoints available")]
    NoHealthyEndpoints,

    /// Failed to create RPC client.
    #[error("Failed to create RPC client: {0}")]
    ClientCreationFailed(String),

    /// Transport error during RPC call.
    #[error("RPC transport error: {0}")]
    TransportError(String),

    /// Invalid endpoint URL.
    #[error("Invalid endpoint URL: {0}")]
    InvalidUrl(String),

    /// Timeout waiting for response.
    #[error("Request timeout after {0}ms")]
    Timeout(u64),

    /// Pool has been shut down.
    #[error("RPC pool has been shut down")]
    PoolShutdown,

    /// No WebSocket-capable endpoints configured.
    #[error("No WebSocket-capable endpoints configured")]
    NoWebSocketEndpoints,

    /// WebSocket connection or subscription error.
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
}

impl From<url::ParseError> for RpcPoolError {
    fn from(err: url::ParseError) -> Self {
        RpcPoolError::InvalidUrl(err.to_string())
    }
}
