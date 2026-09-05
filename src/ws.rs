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

#[path = "ws_retry.rs"]
mod ws_retry;
use ws_retry::EndpointRetry;

use crate::endpoint::RpcEndpoint;
use crate::error::RpcPoolError;

use alloy::primitives::B256;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::{Filter, Header, Log};
use alloy::transports::ws::WsConnect;
use futures_util::stream::Stream;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use tracing::{debug, info, warn};

/// A subscription stream that keeps its WS provider alive.
///
/// When the alloy WS provider is dropped, the underlying transport shuts down
/// and the subscription stream immediately closes ("Pubsub service request
/// channel closed"). This wrapper prevents that by co-owning the provider.
struct OwnedStream<T> {
    _owner: Box<dyn std::any::Any + Send>,
    stream: Pin<Box<dyn Stream<Item = T> + Send>>,
    endpoint_failed: Arc<EndpointRetry>,
}

impl<T> Stream for OwnedStream<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let result = self.stream.as_mut().poll_next(cx);
        if matches!(result, Poll::Ready(None)) {
            self.endpoint_failed.failed();
        }
        result
    }
}

/// Wrap a subscription stream with its provider so the provider is not dropped.
fn owned_stream<T: 'static>(
    provider: impl std::any::Any + Send + 'static,
    stream: impl Stream<Item = T> + Send + 'static,
    endpoint_failed: Arc<EndpointRetry>,
) -> BoxSubscriptionStream<T> {
    Box::pin(OwnedStream {
        _owner: Box::new(provider),
        stream: Box::pin(stream),
        endpoint_failed,
    })
}

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
    /// Terminated endpoints cool down before becoming eligible for subscription again.
    endpoint_failures: Arc<Vec<Arc<EndpointRetry>>>,
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

        info!(ws_endpoints = endpoints.len(), "WebSocket pool initialized");
        for ep in &endpoints {
            debug!(
                name = %ep.name,
                ws_url = %ep.ws_url.as_deref().unwrap_or(""),
                priority = ep.priority,
                "Registered WS endpoint"
            );
        }

        let endpoint_failures = Arc::new(
            endpoints
                .iter()
                .map(|_| Arc::new(EndpointRetry::default()))
                .collect(),
        );

        Ok(Self {
            endpoints,
            config,
            shutdown: Arc::new(AtomicBool::new(false)),
            endpoint_failures,
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
    pub async fn subscribe_new_heads(&self) -> Result<BoxSubscriptionStream<Header>, RpcPoolError> {
        let mut last_error = None;

        for (index, endpoint) in self.endpoints.iter().enumerate() {
            if let Some(ws_url) = &endpoint.ws_url {
                if self.endpoint_failures[index].cooling_down() {
                    continue;
                }
                debug!(name = %endpoint.name, ws_url = %ws_url, "Connecting for newHeads subscription");

                match connect_ws_with_timeout(ws_url, self.config.connect_timeout).await {
                    Ok(provider) => match provider.subscribe_blocks().await {
                        Ok(sub) => {
                            info!(name = %endpoint.name, "Subscribed to newHeads");
                            return Ok(owned_stream(
                                provider,
                                sub.into_stream(),
                                Arc::clone(&self.endpoint_failures[index]),
                            ));
                        }
                        Err(e) => {
                            warn!(name = %endpoint.name, error = %e, "Subscribe failed");
                            last_error = Some(RpcPoolError::WebSocketError(format!(
                                "Subscribe failed on {}: {}",
                                endpoint.name, e
                            )));
                        }
                    },
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

        for (index, endpoint) in self.endpoints.iter().enumerate() {
            if let Some(ws_url) = &endpoint.ws_url {
                if self.endpoint_failures[index].cooling_down() {
                    continue;
                }
                debug!(name = %endpoint.name, ws_url = %ws_url, "Connecting for pendingTransactions subscription");

                match connect_ws_with_timeout(ws_url, self.config.connect_timeout).await {
                    Ok(provider) => match provider.subscribe_pending_transactions().await {
                        Ok(sub) => {
                            info!(name = %endpoint.name, "Subscribed to pendingTransactions");
                            return Ok(owned_stream(
                                provider,
                                sub.into_stream(),
                                Arc::clone(&self.endpoint_failures[index]),
                            ));
                        }
                        Err(e) => {
                            warn!(name = %endpoint.name, error = %e, "Subscribe failed");
                            last_error = Some(RpcPoolError::WebSocketError(format!(
                                "Subscribe failed on {}: {}",
                                endpoint.name, e
                            )));
                        }
                    },
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

        for (index, endpoint) in self.endpoints.iter().enumerate() {
            if let Some(ws_url) = &endpoint.ws_url {
                if self.endpoint_failures[index].cooling_down() {
                    continue;
                }
                debug!(name = %endpoint.name, ws_url = %ws_url, "Connecting for logs subscription");

                match connect_ws_with_timeout(ws_url, self.config.connect_timeout).await {
                    Ok(provider) => match provider.subscribe_logs(filter).await {
                        Ok(sub) => {
                            info!(name = %endpoint.name, "Subscribed to logs");
                            return Ok(owned_stream(
                                provider,
                                sub.into_stream(),
                                Arc::clone(&self.endpoint_failures[index]),
                            ));
                        }
                        Err(e) => {
                            warn!(name = %endpoint.name, error = %e, "Subscribe failed");
                            last_error = Some(RpcPoolError::WebSocketError(format!(
                                "Subscribe failed on {}: {}",
                                endpoint.name, e
                            )));
                        }
                    },
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
    let connect = WsConnect::new(ws_url.to_string()).with_max_retries(0);

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
    let connect = WsConnect::new(ws_url.to_string()).with_max_retries(0);

    let provider = ProviderBuilder::new()
        .connect_ws(connect)
        .await
        .map_err(|e| {
            RpcPoolError::WebSocketError(format!("Failed to connect to {}: {}", ws_url, e))
        })?;

    let sub = provider
        .subscribe_blocks()
        .await
        .map_err(|e| RpcPoolError::WebSocketError(format!("Failed to subscribe: {}", e)))?;

    Ok(owned_stream(
        provider,
        sub.into_stream(),
        Arc::new(EndpointRetry::default()),
    ))
}

/// Connect to a WebSocket endpoint and create a log subscription.
///
/// Standalone helper for creating a single subscription without the pool.
pub async fn connect_and_subscribe_logs(
    ws_url: &str,
    filter: &Filter,
) -> Result<BoxSubscriptionStream<Log>, RpcPoolError> {
    let connect = WsConnect::new(ws_url.to_string()).with_max_retries(0);

    let provider = ProviderBuilder::new()
        .connect_ws(connect)
        .await
        .map_err(|e| {
            RpcPoolError::WebSocketError(format!("Failed to connect to {}: {}", ws_url, e))
        })?;

    let sub = provider
        .subscribe_logs(filter)
        .await
        .map_err(|e| RpcPoolError::WebSocketError(format!("Failed to subscribe: {}", e)))?;

    Ok(owned_stream(
        provider,
        sub.into_stream(),
        Arc::new(EndpointRetry::default()),
    ))
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

    #[tokio::test]
    async fn test_owned_stream_keeps_items_flowing() {
        use futures_util::StreamExt;

        // Simulate a provider (any Send + 'static type) with a channel-backed stream.
        // If the owner were dropped, a real WS stream would close immediately.
        let (tx, rx) = tokio::sync::mpsc::channel::<u64>(4);
        let inner = tokio_stream::wrappers::ReceiverStream::new(rx);

        let owner = String::from("fake-provider");
        let retry = Arc::new(EndpointRetry::default());
        let mut stream = owned_stream(owner, inner, Arc::clone(&retry));

        tx.send(1).await.unwrap();
        tx.send(2).await.unwrap();
        drop(tx); // close sender so stream ends

        let mut items = Vec::new();
        while let Some(v) = stream.next().await {
            items.push(v);
        }
        assert_eq!(items, vec![1, 2]);
        assert!(
            retry.cooling_down(),
            "a terminated stream must cool down its endpoint"
        );
    }

    #[tokio::test]
    async fn test_owned_stream_owner_outlives_stream() {
        use futures_util::StreamExt;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let dropped = Arc::new(AtomicBool::new(false));

        struct DropDetector(Arc<AtomicBool>);
        impl Drop for DropDetector {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        let detector = DropDetector(dropped.clone());
        let (tx, rx) = tokio::sync::mpsc::channel::<u64>(4);
        let inner = tokio_stream::wrappers::ReceiverStream::new(rx);

        let mut stream = owned_stream(detector, inner, Arc::new(EndpointRetry::default()));

        // Owner must not be dropped while stream is alive
        assert!(!dropped.load(Ordering::SeqCst));

        tx.send(42).await.unwrap();
        assert_eq!(stream.next().await, Some(42));
        assert!(!dropped.load(Ordering::SeqCst));

        // Drop the stream — now the owner should be dropped too
        drop(stream);
        assert!(dropped.load(Ordering::SeqCst));
    }
}
