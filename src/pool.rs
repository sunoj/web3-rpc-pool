//! Core RPC pool implementation.

use crate::endpoint::{EndpointStats, RpcEndpoint};
use crate::error::RpcPoolError;
use crate::metrics::{EndpointMetrics, RpcPoolMetrics};
use crate::strategies::SelectionStrategy;

use alloy::providers::{Provider, ProviderBuilder};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::HashSet;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Configuration for the RPC pool.
pub struct RpcPoolConfig {
    /// List of RPC endpoints (will be sorted by priority).
    pub endpoints: Vec<RpcEndpoint>,

    /// Strategy for selecting endpoints.
    pub strategy: Box<dyn SelectionStrategy>,

    /// Interval between health checks.
    pub health_check_interval: Duration,

    /// Number of consecutive errors before marking an endpoint unhealthy.
    pub max_consecutive_errors: u32,

    /// Delay before retrying an unhealthy endpoint.
    pub retry_delay: Duration,
}

impl Default for RpcPoolConfig {
    fn default() -> Self {
        Self {
            endpoints: vec![],
            strategy: Box::new(crate::strategies::FailoverStrategy),
            health_check_interval: Duration::from_secs(60),
            max_consecutive_errors: 3,
            retry_delay: Duration::from_secs(5),
        }
    }
}

/// High-availability RPC connection pool with automatic failover.
pub struct RpcPool {
    /// Configured endpoints (sorted by priority).
    endpoints: Vec<RpcEndpoint>,

    /// Statistics for each endpoint.
    stats: DashMap<String, EndpointStats>,

    /// Selection strategy.
    strategy: RwLock<Box<dyn SelectionStrategy>>,

    /// Configuration.
    max_consecutive_errors: u32,
    retry_delay: Duration,
    health_check_interval: Duration,

    /// Aggregated metrics.
    total_requests: AtomicU64,
    failovers: AtomicU64,
}

impl RpcPool {
    /// Create a new RPC pool with the given configuration.
    pub fn new(mut config: RpcPoolConfig) -> Result<Self, RpcPoolError> {
        if config.endpoints.is_empty() {
            return Err(RpcPoolError::NoEndpointsConfigured);
        }

        // Sort endpoints by priority (lower = higher priority)
        config.endpoints.sort_by_key(|e| e.priority);

        // Initialize stats for each endpoint
        let stats = DashMap::new();
        for endpoint in &config.endpoints {
            stats.insert(endpoint.url.clone(), EndpointStats::new(endpoint));
        }

        info!(
            endpoints = config.endpoints.len(),
            strategy = config.strategy.name(),
            "RPC pool initialized"
        );

        Ok(Self {
            endpoints: config.endpoints,
            stats,
            strategy: RwLock::new(config.strategy),
            max_consecutive_errors: config.max_consecutive_errors,
            retry_delay: config.retry_delay,
            health_check_interval: config.health_check_interval,
            total_requests: AtomicU64::new(0),
            failovers: AtomicU64::new(0),
        })
    }

    /// Get the URL of the currently selected endpoint.
    pub fn get_current_url(&self) -> Option<String> {
        let stats_map: std::collections::HashMap<_, _> = self
            .stats
            .iter()
            .map(|r| (r.key().clone(), r.value().clone()))
            .collect();
        let mut strategy = self.strategy.write();
        strategy
            .select(&self.endpoints, &stats_map, &HashSet::new())
            .map(|e| e.url.clone())
    }

    /// Get all configured RPC URLs.
    pub fn get_all_urls(&self) -> Vec<String> {
        self.endpoints.iter().map(|e| e.url.clone()).collect()
    }

    /// Execute a function with automatic failover across endpoints.
    ///
    /// The provided function receives the endpoint URL and should create
    /// and use its own provider instance.
    pub async fn execute_with_url<F, Fut, T, E>(&self, f: F) -> Result<T, RpcPoolError>
    where
        F: Fn(String) -> Fut + Clone,
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        let mut tried = HashSet::new();
        let mut last_error = None;

        for _ in 0..self.endpoints.len() {
            // Select endpoint
            let endpoint = {
                let stats_map: std::collections::HashMap<_, _> = self
                    .stats
                    .iter()
                    .map(|r| (r.key().clone(), r.value().clone()))
                    .collect();
                let mut strategy = self.strategy.write();
                strategy.select(&self.endpoints, &stats_map, &tried).cloned()
            };

            let endpoint = match endpoint {
                Some(e) => e,
                None => break,
            };

            tried.insert(endpoint.url.clone());

            // Execute request
            let start = Instant::now();
            match f(endpoint.url.clone()).await {
                Ok(result) => {
                    let latency = start.elapsed().as_millis() as u64;
                    if let Some(mut stats) = self.stats.get_mut(&endpoint.url) {
                        stats.record_success(latency);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    if let Some(mut stats) = self.stats.get_mut(&endpoint.url) {
                        let marked_unhealthy =
                            stats.record_failure(error_msg.clone(), self.max_consecutive_errors);
                        if marked_unhealthy {
                            warn!(
                                endpoint = %endpoint.name,
                                consecutive_errors = stats.consecutive_errors,
                                "Endpoint marked unhealthy"
                            );
                        }
                    }

                    self.failovers.fetch_add(1, Ordering::Relaxed);
                    last_error = Some(error_msg);

                    debug!(
                        endpoint = %endpoint.name,
                        error = %e,
                        "Request failed, trying next endpoint"
                    );
                }
            }
        }

        Err(RpcPoolError::AllEndpointsFailed(
            last_error.unwrap_or_else(|| "Unknown error".to_string()),
        ))
    }

    /// Execute a request with automatic failover using a pre-built provider.
    ///
    /// Creates a new provider for each attempt (recommended for most use cases).
    pub async fn execute<T, E, F, Fut>(&self, f: F) -> Result<T, RpcPoolError>
    where
        F: Fn(url::Url) -> Fut + Clone,
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        self.execute_with_url(|url_str| {
            let f = f.clone();
            async move {
                let url: url::Url = url_str.parse().map_err(|e: url::ParseError| {
                    std::io::Error::other(format!("Invalid URL: {}", e))
                })?;
                f(url).await.map_err(|e| std::io::Error::other(e.to_string()))
            }
        })
        .await
    }

    /// Start background health check task.
    ///
    /// Returns a handle that can be used to abort the task.
    pub fn start_health_check(self: &Arc<Self>) -> tokio::task::JoinHandle<()> {
        let pool = Arc::clone(self);
        let interval = self.health_check_interval;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                pool.check_health().await;
            }
        })
    }

    /// Perform health check on all endpoints.
    async fn check_health(&self) {
        for endpoint in &self.endpoints {
            let should_check = {
                let stats = self.stats.get(&endpoint.url);
                match stats {
                    Some(s) => {
                        // Only check unhealthy endpoints
                        if s.is_healthy {
                            false
                        } else {
                            s.can_retry(self.retry_delay)
                        }
                    }
                    None => true,
                }
            };

            if !should_check {
                continue;
            }

            // Try to recover with a simple probe
            let url: Result<url::Url, _> = endpoint.url.parse();
            if let Ok(url) = url {
                let provider = ProviderBuilder::new().connect_http(url);

                match provider.get_block_number().await {
                    Ok(_) => {
                        if let Some(mut stats) = self.stats.get_mut(&endpoint.url) {
                            stats.mark_recovered();
                            info!(endpoint = %endpoint.name, "Endpoint recovered");
                        }
                    }
                    Err(_) => {
                        // Still unhealthy, update error time
                        if let Some(mut stats) = self.stats.get_mut(&endpoint.url) {
                            stats.last_error_time = Some(Instant::now());
                        }
                    }
                }
            }
        }
    }

    /// Manually mark an endpoint as unhealthy.
    pub fn mark_unhealthy(&self, url: &str) {
        if let Some(mut stats) = self.stats.get_mut(url) {
            stats.is_healthy = false;
            stats.last_error_time = Some(Instant::now());
        }
    }

    /// Get current metrics.
    pub fn metrics(&self) -> RpcPoolMetrics {
        let endpoints: Vec<EndpointMetrics> = self
            .stats
            .iter()
            .map(|r| EndpointMetrics::from(r.value()))
            .collect();

        let current_endpoint = {
            let stats_map: std::collections::HashMap<_, _> = self
                .stats
                .iter()
                .map(|r| (r.key().clone(), r.value().clone()))
                .collect();
            let mut strategy = self.strategy.write();
            strategy
                .select(&self.endpoints, &stats_map, &HashSet::new())
                .map(|e| e.name.clone())
                .unwrap_or_else(|| "none".to_string())
        };

        RpcPoolMetrics {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            failovers: self.failovers.load(Ordering::Relaxed),
            current_endpoint,
            endpoints,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategies::FailoverStrategy;

    #[test]
    fn test_pool_creation() {
        let config = RpcPoolConfig {
            endpoints: vec![
                RpcEndpoint::new("https://rpc1.example.com"),
                RpcEndpoint::new("https://rpc2.example.com"),
            ],
            strategy: Box::new(FailoverStrategy),
            ..Default::default()
        };

        let pool = RpcPool::new(config);
        assert!(pool.is_ok());
    }

    #[test]
    fn test_empty_endpoints() {
        let config = RpcPoolConfig {
            endpoints: vec![],
            strategy: Box::new(FailoverStrategy),
            ..Default::default()
        };

        let pool = RpcPool::new(config);
        assert!(matches!(pool, Err(RpcPoolError::NoEndpointsConfigured)));
    }

    #[test]
    fn test_get_urls() {
        let config = RpcPoolConfig {
            endpoints: vec![
                RpcEndpoint::new("https://rpc1.example.com").with_priority(10),
                RpcEndpoint::new("https://rpc2.example.com").with_priority(50),
            ],
            strategy: Box::new(FailoverStrategy),
            ..Default::default()
        };

        let pool = RpcPool::new(config).unwrap();

        // Should return highest priority (lowest number) endpoint
        assert_eq!(
            pool.get_current_url(),
            Some("https://rpc1.example.com".to_string())
        );

        // Should return all URLs
        let all = pool.get_all_urls();
        assert_eq!(all.len(), 2);
    }
}
