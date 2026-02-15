//! WebSocket subscription pool with automatic failover and reconnection.
//!
//! Provides subscription-based streaming over WebSocket connections with
//! automatic endpoint failover when connections drop.
//!
//! # Example
//!
//! ```rust,no_run
//! use web3_rpc_pool::ws::WsPool;
//! use web3_rpc_pool::presets;
//! use futures_util::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let pool = WsPool::new(presets::ethereum_endpoints())?;
//!
//!     let mut stream = pool.subscribe_new_heads().await?;
//!     while let Some(header) = stream.next().await {
//!         println!("New block: {:?}", header.number);
//!     }
//!     Ok(())
//! }
//! ```

use crate::endpoint::RpcEndpoint;
use crate::error::RpcPoolError;

use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::{Filter, Header, Log};
use alloy::primitives::B256;
use alloy::transports::ws::WsConnect;
use futures_util::stream::Stream;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Default connection timeout for WebSocket endpoints.
const DEFAULT_WS_CONNECT_TIMEOUT_SECS: u64 = 15;

/// Default delay between reconnection attempts.
const DEFAULT_RECONNECT_DELAY_MS: u64 = 1000;

/// Maximum reconnection delay (with exponential backoff).
const MAX_RECONNECT_DELAY_MS: u64 = 30_000;

/// Configuration for the WebSocket pool.
#[derive(Clone)]
pub struct WsPoolConfig {
    /// Connection timeout for WebSocket endpoints.
    pub connect_timeout: Duration,
    /// Base delay between reconnection attempts.
    pub reconnect_delay: Duration,
    /// Maximum reconnection delay (exponential backoff cap).
    pub max_reconnect_delay: Duration,
}

impl Default for WsPoolConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(DEFAULT_WS_CONNECT_TIMEOUT_SECS),
            reconnect_delay: Duration::from_millis(DEFAULT_RECONNECT_DELAY_MS),
            max_reconnect_delay: Duration::from_millis(MAX_RECONNECT_DELAY_MS),
        }
    }
}

/// A boxed stream type for subscription items.
pub type BoxSubscriptionStream<T> = Pin<Box<dyn Stream<Item = T> + Send>>;

/// WebSocket subscription pool with automatic failover.
///
/// Manages WebSocket connections to multiple RPC endpoints and provides
/// subscription streams that automatically reconnect and failover on errors.
pub struct WsPool {
    /// Endpoints sorted by priority (only those with ws_url).
    endpoints: Vec<RpcEndpoint>,
    /// Configuration.
    config: WsPoolConfig,
    /// Shutdown flag.
    shutdown: Arc<AtomicBool>,
}

impl WsPool {
    /// Create a new WebSocket pool from endpoints.
    ///
    /// Filters endpoints to only those with `ws_url` configured and sorts by priority.
    pub fn new(endpoints: Vec<RpcEndpoint>) -> Result<Self, RpcPoolError> {
        Self::with_config(endpoints, WsPoolConfig::default())
    }

    /// Create a new WebSocket pool with custom configuration.
    pub fn with_config(
        mut endpoints: Vec<RpcEndpoint>,
        config: WsPoolConfig,
    ) -> Result<Self, RpcPoolError> {
        // Filter to only endpoints with ws_url
        endpoints.retain(|e| e.ws_url.is_some());

        if endpoints.is_empty() {
            return Err(RpcPoolError::NoWebSocketEndpoints);
        }

        // Sort by priority (lower = higher priority)
        endpoints.sort_by_key(|e| e.priority);

        info!(
            ws_endpoints = endpoints.len(),
            "WebSocket pool initialized"
        );
        for ep in &endpoints {
            debug!(
                name = %ep.name,
                ws_url = %ep.ws_url.as_deref().unwrap_or(""),
                priority = ep.priority,
                "Registered WS endpoint"
            );
        }

        Ok(Self {
            endpoints,
            config,
            shutdown: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Get the number of WebSocket-capable endpoints.
    pub fn endpoint_count(&self) -> usize {
        self.endpoints.len()
    }

    /// Get all WebSocket URLs.
    pub fn ws_urls(&self) -> Vec<String> {
        self.endpoints
            .iter()
            .filter_map(|e| e.ws_url.clone())
            .collect()
    }

    /// Subscribe to new block headers with automatic failover.
    ///
    /// Tries each WebSocket endpoint in priority order until one connects
    /// and establishes a subscription. Returns a stream of block headers.
    pub async fn subscribe_new_heads(
        &self,
    ) -> Result<BoxSubscriptionStream<Header>, RpcPoolError> {
        let mut last_error = None;

        for endpoint in &self.endpoints {
            if let Some(ws_url) = &endpoint.ws_url {
                debug!(name = %endpoint.name, ws_url = %ws_url, "Connecting for newHeads subscription");

                match connect_ws_with_timeout(ws_url, self.config.connect_timeout).await {
                    Ok(provider) => {
                        match provider.subscribe_blocks().await {
                            Ok(sub) => {
                                info!(name = %endpoint.name, "Subscribed to newHeads");
                                return Ok(Box::pin(sub.into_stream()));
                            }
                            Err(e) => {
                                warn!(name = %endpoint.name, error = %e, "Subscribe failed");
                                last_error = Some(RpcPoolError::WebSocketError(
                                    format!("Subscribe failed on {}: {}", endpoint.name, e),
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        warn!(name = %endpoint.name, error = %e, "WS connect failed");
                        last_error = Some(e);
                    }
                }
            }
        }

        Err(last_error.unwrap_or(RpcPoolError::NoWebSocketEndpoints))
    }

    /// Subscribe to pending transaction hashes with automatic failover.
    ///
    /// Tries each WebSocket endpoint in priority order until one connects
    /// and establishes a subscription. Returns a stream of transaction hashes.
    pub async fn subscribe_pending_transactions(
        &self,
    ) -> Result<BoxSubscriptionStream<B256>, RpcPoolError> {
        let mut last_error = None;

        for endpoint in &self.endpoints {
            if let Some(ws_url) = &endpoint.ws_url {
                debug!(name = %endpoint.name, ws_url = %ws_url, "Connecting for pendingTransactions subscription");

                match connect_ws_with_timeout(ws_url, self.config.connect_timeout).await {
                    Ok(provider) => {
                        match provider.subscribe_pending_transactions().await {
                            Ok(sub) => {
                                info!(name = %endpoint.name, "Subscribed to pendingTransactions");
                                return Ok(Box::pin(sub.into_stream()));
                            }
                            Err(e) => {
                                warn!(name = %endpoint.name, error = %e, "Subscribe failed");
                                last_error = Some(RpcPoolError::WebSocketError(
                                    format!("Subscribe failed on {}: {}", endpoint.name, e),
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        warn!(name = %endpoint.name, error = %e, "WS connect failed");
                        last_error = Some(e);
                    }
                }
            }
        }

        Err(last_error.unwrap_or(RpcPoolError::NoWebSocketEndpoints))
    }

    /// Subscribe to log events matching a filter with automatic failover.
    ///
    /// Tries each WebSocket endpoint in priority order until one connects
    /// and establishes a subscription. Returns a stream of matching logs.
    pub async fn subscribe_logs(
        &self,
        filter: &Filter,
    ) -> Result<BoxSubscriptionStream<Log>, RpcPoolError> {
        let mut last_error = None;

        for endpoint in &self.endpoints {
            if let Some(ws_url) = &endpoint.ws_url {
                debug!(name = %endpoint.name, ws_url = %ws_url, "Connecting for logs subscription");

                match connect_ws_with_timeout(ws_url, self.config.connect_timeout).await {
                    Ok(provider) => {
                        match provider.subscribe_logs(filter).await {
                            Ok(sub) => {
                                info!(name = %endpoint.name, "Subscribed to logs");
                                return Ok(Box::pin(sub.into_stream()));
                            }
                            Err(e) => {
                                warn!(name = %endpoint.name, error = %e, "Subscribe failed");
                                last_error = Some(RpcPoolError::WebSocketError(
                                    format!("Subscribe failed on {}: {}", endpoint.name, e),
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        warn!(name = %endpoint.name, error = %e, "WS connect failed");
                        last_error = Some(e);
                    }
                }
            }
        }

        Err(last_error.unwrap_or(RpcPoolError::NoWebSocketEndpoints))
    }

    /// Shutdown the WebSocket pool.
    ///
    /// Signals all active subscription streams to stop reconnecting.
    pub fn shutdown(&self) {
        info!("WebSocket pool shutting down");
        self.shutdown.store(true, Ordering::Release);
    }

    /// Check if the pool has been shut down.
    pub fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Acquire)
    }
}

/// Connect to a WebSocket endpoint with timeout.
async fn connect_ws_with_timeout(
    ws_url: &str,
    timeout: Duration,
) -> Result<impl Provider, RpcPoolError> {
    let connect = WsConnect::new(ws_url.to_string());

    let provider = tokio::time::timeout(timeout, ProviderBuilder::new().connect_ws(connect))
        .await
        .map_err(|_| {
            RpcPoolError::WebSocketError(format!(
                "Connection timeout after {}ms to {}",
                timeout.as_millis(),
                ws_url
            ))
        })?
        .map_err(|e| {
            RpcPoolError::WebSocketError(format!("Failed to connect to {}: {}", ws_url, e))
        })?;

    Ok(provider)
}

/// Connect to a WebSocket endpoint and create a block header subscription.
///
/// Standalone helper for creating a single subscription without the pool.
pub async fn connect_and_subscribe_blocks(
    ws_url: &str,
) -> Result<BoxSubscriptionStream<Header>, RpcPoolError> {
    let connect = WsConnect::new(ws_url.to_string());

    let provider = ProviderBuilder::new()
        .connect_ws(connect)
        .await
        .map_err(|e| {
            RpcPoolError::WebSocketError(format!("Failed to connect to {}: {}", ws_url, e))
        })?;

    let sub = provider.subscribe_blocks().await.map_err(|e| {
        RpcPoolError::WebSocketError(format!("Failed to subscribe: {}", e))
    })?;

    Ok(Box::pin(sub.into_stream()))
}

/// Connect to a WebSocket endpoint and create a log subscription.
///
/// Standalone helper for creating a single subscription without the pool.
pub async fn connect_and_subscribe_logs(
    ws_url: &str,
    filter: &Filter,
) -> Result<BoxSubscriptionStream<Log>, RpcPoolError> {
    let connect = WsConnect::new(ws_url.to_string());

    let provider = ProviderBuilder::new()
        .connect_ws(connect)
        .await
        .map_err(|e| {
            RpcPoolError::WebSocketError(format!("Failed to connect to {}: {}", ws_url, e))
        })?;

    let sub = provider.subscribe_logs(filter).await.map_err(|e| {
        RpcPoolError::WebSocketError(format!("Failed to subscribe: {}", e))
    })?;

    Ok(Box::pin(sub.into_stream()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_ws_endpoints() -> Vec<RpcEndpoint> {
        vec![
            RpcEndpoint::new("https://rpc1.example.com")
                .with_name("Test1")
                .with_ws_url("wss://ws1.example.com")
                .with_priority(10),
            RpcEndpoint::new("https://rpc2.example.com")
                .with_name("Test2")
                .with_ws_url("wss://ws2.example.com")
                .with_priority(50),
        ]
    }

    #[test]
    fn test_ws_pool_creation() {
        let pool = WsPool::new(create_ws_endpoints());
        assert!(pool.is_ok());
        let pool = pool.unwrap();
        assert_eq!(pool.endpoint_count(), 2);
    }

    #[test]
    fn test_ws_pool_filters_non_ws_endpoints() {
        let endpoints = vec![
            RpcEndpoint::new("https://rpc1.example.com")
                .with_name("WithWS")
                .with_ws_url("wss://ws1.example.com"),
            RpcEndpoint::new("https://rpc2.example.com").with_name("WithoutWS"),
        ];

        let pool = WsPool::new(endpoints).unwrap();
        assert_eq!(pool.endpoint_count(), 1);
    }

    #[test]
    fn test_ws_pool_no_ws_endpoints() {
        let endpoints = vec![
            RpcEndpoint::new("https://rpc1.example.com").with_name("NoWS1"),
            RpcEndpoint::new("https://rpc2.example.com").with_name("NoWS2"),
        ];

        let pool = WsPool::new(endpoints);
        assert!(matches!(pool, Err(RpcPoolError::NoWebSocketEndpoints)));
    }

    #[test]
    fn test_ws_pool_urls() {
        let pool = WsPool::new(create_ws_endpoints()).unwrap();
        let urls = pool.ws_urls();
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "wss://ws1.example.com");
        assert_eq!(urls[1], "wss://ws2.example.com");
    }

    #[test]
    fn test_ws_pool_priority_sorting() {
        let endpoints = vec![
            RpcEndpoint::new("https://rpc1.example.com")
                .with_name("Low Priority")
                .with_ws_url("wss://ws1.example.com")
                .with_priority(100),
            RpcEndpoint::new("https://rpc2.example.com")
                .with_name("High Priority")
                .with_ws_url("wss://ws2.example.com")
                .with_priority(10),
        ];

        let pool = WsPool::new(endpoints).unwrap();
        let urls = pool.ws_urls();
        // Higher priority (lower number) should come first
        assert_eq!(urls[0], "wss://ws2.example.com");
        assert_eq!(urls[1], "wss://ws1.example.com");
    }

    #[test]
    fn test_ws_pool_shutdown() {
        let pool = WsPool::new(create_ws_endpoints()).unwrap();
        assert!(!pool.is_shutdown());
        pool.shutdown();
        assert!(pool.is_shutdown());
    }

    #[test]
    fn test_ws_pool_config() {
        let config = WsPoolConfig {
            connect_timeout: Duration::from_secs(5),
            reconnect_delay: Duration::from_millis(500),
            max_reconnect_delay: Duration::from_secs(10),
        };

        let pool = WsPool::with_config(create_ws_endpoints(), config).unwrap();
        assert_eq!(pool.endpoint_count(), 2);
    }
}
