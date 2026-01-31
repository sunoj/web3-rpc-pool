//! Tiered RPC pool for priority-based request routing.
//!
//! This module provides a multi-tier RPC pool that routes requests to different
//! endpoint pools based on request priority. This is useful for:
//! - Reserving premium RPC endpoints for critical operations (liquidations, trades)
//! - Using free public RPCs for non-urgent batch operations (historical sync)
//! - Managing RPC costs by routing low-priority requests to free tiers

use crate::endpoint::RpcEndpoint;
use crate::error::RpcPoolError;
use crate::pool::{RpcPool, RpcPoolConfig};
use crate::strategies::{FailoverStrategy, RateAwareStrategy, SelectionStrategy};

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

/// Request priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RequestPriority {
    /// Critical operations: liquidation execution, transaction submission.
    /// Uses premium tier first, falls back to standard.
    Critical,

    /// Normal operations: real-time health checks, price queries.
    /// Uses standard tier first, falls back to free.
    Normal,

    /// Low priority: historical sync, batch queries, background tasks.
    /// Uses free tier only to preserve premium/standard quota.
    Low,
}

impl Default for RequestPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Endpoint tier classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EndpointTier {
    /// Premium tier: paid RPCs with high rate limits (Alchemy, Infura, QuickNode).
    Premium,

    /// Standard tier: reliable public RPCs with moderate limits.
    Standard,

    /// Free tier: public RPCs with low rate limits, best for batch operations.
    Free,
}

impl Default for EndpointTier {
    fn default() -> Self {
        Self::Standard
    }
}

/// Extended endpoint configuration with tier information.
#[derive(Clone, Debug)]
pub struct TieredEndpoint {
    /// Base endpoint configuration.
    pub endpoint: RpcEndpoint,

    /// Tier classification.
    pub tier: EndpointTier,

    /// Rate limit (requests per second), 0 = unlimited.
    pub rate_limit: u32,
}

impl TieredEndpoint {
    /// Create a new tiered endpoint.
    pub fn new(url: impl Into<String>, tier: EndpointTier) -> Self {
        Self {
            endpoint: RpcEndpoint::new(url),
            tier,
            rate_limit: 0,
        }
    }

    /// Set endpoint name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.endpoint = self.endpoint.with_name(name);
        self
    }

    /// Set rate limit.
    pub fn with_rate_limit(mut self, rps: u32) -> Self {
        self.rate_limit = rps;
        self
    }

    /// Set priority within the tier.
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.endpoint = self.endpoint.with_priority(priority);
        self
    }

    /// Set chain ID.
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.endpoint = self.endpoint.with_chain_id(chain_id);
        self
    }
}

/// Configuration for the tiered RPC pool.
pub struct TieredPoolConfig {
    /// Endpoints grouped by tier.
    pub endpoints: Vec<TieredEndpoint>,

    /// Health check interval.
    pub health_check_interval: Duration,

    /// Max consecutive errors before marking unhealthy.
    pub max_consecutive_errors: u32,

    /// Delay before retrying unhealthy endpoint.
    pub retry_delay: Duration,

    /// Whether to allow fallback to lower tiers for critical requests.
    pub allow_critical_fallback: bool,

    /// Whether to allow fallback to higher tiers for low priority requests (not recommended).
    pub allow_low_escalation: bool,
}

impl Default for TieredPoolConfig {
    fn default() -> Self {
        Self {
            endpoints: vec![],
            health_check_interval: Duration::from_secs(60),
            max_consecutive_errors: 3,
            retry_delay: Duration::from_secs(5),
            allow_critical_fallback: true,
            allow_low_escalation: false,
        }
    }
}

/// Multi-tier RPC pool with priority-based routing.
pub struct TieredPool {
    /// Pool for each tier.
    pools: HashMap<EndpointTier, Arc<RpcPool>>,

    /// Fallback configuration.
    allow_critical_fallback: bool,
    allow_low_escalation: bool,
}

impl TieredPool {
    /// Create a new tiered pool from configuration.
    pub fn new(config: TieredPoolConfig) -> Result<Self, RpcPoolError> {
        let mut tier_endpoints: HashMap<EndpointTier, Vec<RpcEndpoint>> = HashMap::new();

        // Group endpoints by tier
        for te in config.endpoints {
            tier_endpoints
                .entry(te.tier)
                .or_default()
                .push(te.endpoint);
        }

        if tier_endpoints.is_empty() {
            return Err(RpcPoolError::NoEndpointsConfigured);
        }

        // Create a pool for each tier
        let mut pools = HashMap::new();

        for (tier, endpoints) in tier_endpoints {
            if endpoints.is_empty() {
                continue;
            }

            let strategy: Box<dyn SelectionStrategy> = match tier {
                // Premium: use failover to maximize success rate
                EndpointTier::Premium => Box::new(FailoverStrategy),
                // Standard: use failover (paid RPCs, prefer reliability)
                EndpointTier::Standard => Box::new(FailoverStrategy),
                // Free: use rate-aware to distribute load across all providers
                // This tracks last request time per endpoint and selects the
                // one that has been idle longest, naturally staying within rate limits
                EndpointTier::Free => Box::new(RateAwareStrategy::new()),
            };

            let pool_config = RpcPoolConfig {
                endpoints,
                strategy,
                health_check_interval: config.health_check_interval,
                max_consecutive_errors: config.max_consecutive_errors,
                retry_delay: config.retry_delay,
            };

            let pool = RpcPool::new(pool_config)?;
            info!(tier = ?tier, "Created RPC pool for tier");
            pools.insert(tier, Arc::new(pool));
        }

        Ok(Self {
            pools,
            allow_critical_fallback: config.allow_critical_fallback,
            allow_low_escalation: config.allow_low_escalation,
        })
    }

    /// Get the tier order for a given priority.
    fn tier_order(&self, priority: RequestPriority) -> Vec<EndpointTier> {
        match priority {
            RequestPriority::Critical => {
                if self.allow_critical_fallback {
                    vec![
                        EndpointTier::Premium,
                        EndpointTier::Standard,
                        EndpointTier::Free,
                    ]
                } else {
                    vec![EndpointTier::Premium]
                }
            }
            RequestPriority::Normal => {
                vec![EndpointTier::Standard, EndpointTier::Free]
            }
            RequestPriority::Low => {
                if self.allow_low_escalation {
                    vec![
                        EndpointTier::Free,
                        EndpointTier::Standard,
                        EndpointTier::Premium,
                    ]
                } else {
                    vec![EndpointTier::Free]
                }
            }
        }
    }

    /// Execute a request with the specified priority.
    pub async fn execute<T, E, F, Fut>(
        &self,
        priority: RequestPriority,
        f: F,
    ) -> Result<T, RpcPoolError>
    where
        F: Fn(url::Url) -> Fut + Clone,
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        let tiers = self.tier_order(priority);
        let mut last_error = None;

        for tier in tiers {
            if let Some(pool) = self.pools.get(&tier) {
                debug!(priority = ?priority, tier = ?tier, "Attempting request");

                match pool.execute(f.clone()).await {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        debug!(tier = ?tier, error = %e, "Tier failed, trying next");
                        last_error = Some(e);
                    }
                }
            }
        }

        Err(last_error.unwrap_or(RpcPoolError::NoEndpointsConfigured))
    }

    /// Execute with URL string instead of parsed URL.
    pub async fn execute_with_url<T, E, F, Fut>(
        &self,
        priority: RequestPriority,
        f: F,
    ) -> Result<T, RpcPoolError>
    where
        F: Fn(String) -> Fut + Clone,
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        let tiers = self.tier_order(priority);
        let mut last_error = None;

        for tier in tiers {
            if let Some(pool) = self.pools.get(&tier) {
                debug!(priority = ?priority, tier = ?tier, "Attempting request with URL string");

                match pool.execute_with_url(f.clone()).await {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        debug!(tier = ?tier, error = %e, "Tier failed, trying next");
                        last_error = Some(e);
                    }
                }
            }
        }

        Err(last_error.unwrap_or(RpcPoolError::NoEndpointsConfigured))
    }

    /// Get pool for a specific tier.
    pub fn get_tier_pool(&self, tier: EndpointTier) -> Option<&Arc<RpcPool>> {
        self.pools.get(&tier)
    }

    /// Check if a tier is available.
    pub fn has_tier(&self, tier: EndpointTier) -> bool {
        self.pools.contains_key(&tier)
    }

    /// Start health checks for all tiers.
    pub fn start_health_checks(&self) -> Vec<tokio::task::JoinHandle<()>> {
        self.pools
            .values()
            .map(|pool| pool.start_health_check())
            .collect()
    }

    /// Get all available tiers.
    pub fn available_tiers(&self) -> Vec<EndpointTier> {
        let mut tiers: Vec<_> = self.pools.keys().copied().collect();
        tiers.sort();
        tiers
    }
}

/// Builder for creating tiered pool configurations.
pub struct TieredPoolBuilder {
    endpoints: Vec<TieredEndpoint>,
    health_check_interval: Duration,
    max_consecutive_errors: u32,
    retry_delay: Duration,
    allow_critical_fallback: bool,
    allow_low_escalation: bool,
}

impl Default for TieredPoolBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TieredPoolBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            endpoints: vec![],
            health_check_interval: Duration::from_secs(60),
            max_consecutive_errors: 3,
            retry_delay: Duration::from_secs(5),
            allow_critical_fallback: true,
            allow_low_escalation: false,
        }
    }

    /// Add a premium endpoint.
    pub fn add_premium(mut self, url: impl Into<String>, name: impl Into<String>) -> Self {
        self.endpoints.push(
            TieredEndpoint::new(url, EndpointTier::Premium)
                .with_name(name)
                .with_priority(10),
        );
        self
    }

    /// Add a standard endpoint.
    pub fn add_standard(mut self, url: impl Into<String>, name: impl Into<String>) -> Self {
        self.endpoints.push(
            TieredEndpoint::new(url, EndpointTier::Standard)
                .with_name(name)
                .with_priority(50),
        );
        self
    }

    /// Add a free endpoint.
    pub fn add_free(mut self, url: impl Into<String>, name: impl Into<String>) -> Self {
        self.endpoints.push(
            TieredEndpoint::new(url, EndpointTier::Free)
                .with_name(name)
                .with_priority(100),
        );
        self
    }

    /// Add a custom tiered endpoint.
    pub fn add_endpoint(mut self, endpoint: TieredEndpoint) -> Self {
        self.endpoints.push(endpoint);
        self
    }

    /// Add multiple free endpoints from a preset.
    pub fn add_free_endpoints(mut self, endpoints: Vec<RpcEndpoint>) -> Self {
        for e in endpoints {
            self.endpoints.push(TieredEndpoint {
                endpoint: e,
                tier: EndpointTier::Free,
                rate_limit: 0,
            });
        }
        self
    }

    /// Set health check interval.
    pub fn health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = interval;
        self
    }

    /// Allow critical requests to fall back to lower tiers.
    pub fn allow_critical_fallback(mut self, allow: bool) -> Self {
        self.allow_critical_fallback = allow;
        self
    }

    /// Allow low priority requests to escalate to higher tiers (not recommended).
    pub fn allow_low_escalation(mut self, allow: bool) -> Self {
        self.allow_low_escalation = allow;
        self
    }

    /// Build the tiered pool.
    pub fn build(self) -> Result<TieredPool, RpcPoolError> {
        TieredPool::new(TieredPoolConfig {
            endpoints: self.endpoints,
            health_check_interval: self.health_check_interval,
            max_consecutive_errors: self.max_consecutive_errors,
            retry_delay: self.retry_delay,
            allow_critical_fallback: self.allow_critical_fallback,
            allow_low_escalation: self.allow_low_escalation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_order_critical() {
        let pool = TieredPoolBuilder::new()
            .add_premium("https://premium.example.com", "Premium")
            .add_free("https://free.example.com", "Free")
            .build()
            .unwrap();

        let tiers = pool.tier_order(RequestPriority::Critical);
        assert_eq!(tiers[0], EndpointTier::Premium);
    }

    #[test]
    fn test_tier_order_low() {
        let pool = TieredPoolBuilder::new()
            .add_premium("https://premium.example.com", "Premium")
            .add_free("https://free.example.com", "Free")
            .build()
            .unwrap();

        let tiers = pool.tier_order(RequestPriority::Low);
        assert_eq!(tiers.len(), 1);
        assert_eq!(tiers[0], EndpointTier::Free);
    }

    #[test]
    fn test_builder() {
        let pool = TieredPoolBuilder::new()
            .add_premium("https://premium.example.com", "Premium")
            .add_standard("https://standard.example.com", "Standard")
            .add_free("https://free.example.com", "Free")
            .health_check_interval(Duration::from_secs(30))
            .allow_critical_fallback(true)
            .allow_low_escalation(false)
            .build();

        assert!(pool.is_ok());
        let pool = pool.unwrap();
        assert!(pool.has_tier(EndpointTier::Premium));
        assert!(pool.has_tier(EndpointTier::Standard));
        assert!(pool.has_tier(EndpointTier::Free));
    }
}
