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
use crate::presets;
use crate::strategies::{FailoverStrategy, RateAwareStrategy, SelectionStrategy};

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

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

    /// Whether this tier should bypass pool health checks and retries.
    pub trusted: bool,
}

impl TieredEndpoint {
    /// Create a new tiered endpoint.
    pub fn new(url: impl Into<String>, tier: EndpointTier) -> Self {
        Self {
            endpoint: RpcEndpoint::new(url),
            tier,
            rate_limit: 0,
            trusted: false,
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

    /// Mark this endpoint as preferred (fast, unlimited, co-located node). Selection
    /// strategies pick it first within its tier and never rate-shed onto it.
    pub fn with_preferred(mut self, preferred: bool) -> Self {
        self.endpoint = self.endpoint.with_preferred(preferred);
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

    /// Trusted tier URLs that bypass pool health checks and retries.
    trusted_urls: HashMap<EndpointTier, url::Url>,

    /// Fallback configuration.
    allow_critical_fallback: bool,
    allow_low_escalation: bool,
}

impl TieredPool {
    /// Create a new tiered pool from configuration.
    pub fn new(config: TieredPoolConfig) -> Result<Self, RpcPoolError> {
        let mut tier_endpoints: HashMap<EndpointTier, Vec<TieredEndpoint>> = HashMap::new();

        // Group endpoints by tier
        for te in config.endpoints {
            tier_endpoints.entry(te.tier).or_default().push(te);
        }

        if tier_endpoints.is_empty() {
            return Err(RpcPoolError::NoEndpointsConfigured);
        }

        // Create a pool for each tier
        let mut pools = HashMap::new();
        let mut trusted_urls = HashMap::new();

        for (tier, endpoints) in tier_endpoints {
            if endpoints.is_empty() {
                continue;
            }

            if endpoints.iter().all(|endpoint| endpoint.trusted) {
                let url = url::Url::parse(&endpoints[0].endpoint.url)?;
                info!(tier = ?tier, url = %url, "Configured trusted tier URL");
                trusted_urls.insert(tier, url);
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

            let pool_config = RpcPoolConfig::new()
                .with_endpoints(endpoints.into_iter().map(|endpoint| endpoint.endpoint).collect())
                .with_strategy(strategy)
                .with_health_check_interval(config.health_check_interval)
                .with_max_consecutive_errors(config.max_consecutive_errors)
                .with_retry_delay(config.retry_delay);

            let pool = RpcPool::new(pool_config)?;
            info!(tier = ?tier, "Created RPC pool for tier");
            pools.insert(tier, Arc::new(pool));
        }

        Ok(Self {
            pools,
            trusted_urls,
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
        let mut tried_tiers = Vec::new();

        for tier in &tiers {
            if let Some(url) = self.trusted_urls.get(tier) {
                debug!(priority = ?priority, tier = ?tier, "Attempting trusted tier");
                tried_tiers.push(*tier);

                match tokio::time::timeout(Duration::from_secs(30), f(url.clone())).await {
                    Ok(Ok(result)) => return Ok(result),
                    Ok(Err(e)) => {
                        warn!(tier = ?tier, error = %e, "Trusted tier failed, falling back to next tier");
                        last_error = Some(RpcPoolError::AllEndpointsFailed(e.to_string()));
                    }
                    Err(_) => {
                        warn!(tier = ?tier, "Trusted tier timed out");
                        return Err(RpcPoolError::AllEndpointsFailed(
                            "Trusted endpoint request timed out after 30s".to_string(),
                        ));
                    }
                }

                continue;
            }

            if let Some(pool) = self.pools.get(tier) {
                debug!(priority = ?priority, tier = ?tier, "Attempting tier");
                tried_tiers.push(*tier);

                match pool.execute(f.clone()).await {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        warn!(tier = ?tier, error = %e, "Tier failed, falling back to next tier");
                        last_error = Some(e);
                    }
                }
            } else {
                debug!(tier = ?tier, "Tier not configured, skipping");
            }
        }

        warn!(
            priority = ?priority,
            tried_tiers = ?tried_tiers,
            available_tiers = ?tiers,
            error = ?last_error,
            "All tiers failed"
        );
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
        let mut tried_tiers = Vec::new();

        for tier in &tiers {
            if let Some(url) = self.trusted_urls.get(tier) {
                debug!(priority = ?priority, tier = ?tier, "Attempting trusted tier with URL string");
                tried_tiers.push(*tier);

                match tokio::time::timeout(Duration::from_secs(30), f(url.to_string())).await {
                    Ok(Ok(result)) => return Ok(result),
                    Ok(Err(e)) => {
                        warn!(tier = ?tier, error = %e, "Trusted tier failed, falling back to next tier");
                        last_error = Some(RpcPoolError::AllEndpointsFailed(e.to_string()));
                    }
                    Err(_) => {
                        warn!(tier = ?tier, "Trusted tier timed out");
                        return Err(RpcPoolError::AllEndpointsFailed(
                            "Trusted endpoint request timed out after 30s".to_string(),
                        ));
                    }
                }

                continue;
            }

            if let Some(pool) = self.pools.get(tier) {
                debug!(priority = ?priority, tier = ?tier, "Attempting tier with URL string");
                tried_tiers.push(*tier);

                match pool.execute_with_url(f.clone()).await {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        warn!(tier = ?tier, error = %e, "Tier failed, falling back to next tier");
                        last_error = Some(e);
                    }
                }
            } else {
                debug!(tier = ?tier, "Tier not configured, skipping");
            }
        }

        warn!(
            priority = ?priority,
            tried_tiers = ?tried_tiers,
            available_tiers = ?tiers,
            error = ?last_error,
            "All tiers failed"
        );
        Err(last_error.unwrap_or(RpcPoolError::NoEndpointsConfigured))
    }

    /// Get pool for a specific tier.
    pub fn get_tier_pool(&self, tier: EndpointTier) -> Option<&Arc<RpcPool>> {
        self.pools.get(&tier)
    }

    /// Check if a tier is available.
    pub fn has_tier(&self, tier: EndpointTier) -> bool {
        self.pools.contains_key(&tier) || self.trusted_urls.contains_key(&tier)
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
        let mut tiers: HashSet<_> = self.pools.keys().copied().collect();
        tiers.extend(self.trusted_urls.keys().copied());
        let mut tiers: Vec<_> = tiers.into_iter().collect();
        tiers.sort();
        tiers
    }

    /// Get endpoint count for each tier (useful for debugging).
    pub fn tier_endpoint_counts(&self) -> HashMap<EndpointTier, usize> {
        let mut counts: HashMap<_, _> = self
            .pools
            .iter()
            .map(|(tier, pool)| (*tier, pool.get_all_urls().len()))
            .collect();
        for tier in self.trusted_urls.keys() {
            counts.insert(*tier, 1);
        }
        counts
    }

    /// Get the trusted URL for the highest-priority matching tier.
    pub fn trusted_url(&self, priority: RequestPriority) -> Option<url::Url> {
        self.tier_order(priority)
            .into_iter()
            .find_map(|tier| self.trusted_urls.get(&tier).cloned())
    }

    /// Log current tier configuration for debugging.
    pub fn log_tier_info(&self) {
        let counts = self.tier_endpoint_counts();
        for tier in [EndpointTier::Premium, EndpointTier::Standard, EndpointTier::Free] {
            if let Some(count) = counts.get(&tier) {
                info!(tier = ?tier, endpoint_count = count, "Tier configuration");
            } else {
                debug!(tier = ?tier, "Tier not configured");
            }
        }
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

    /// Add a PREFERRED endpoint to a tier — a fast, unlimited, co-located node that
    /// the tier's selection strategy picks first and never rate-sheds. Use the SAME
    /// node URL across multiple tiers (e.g. Standard + Free); the per-(tier,url) dedup
    /// keeps each, so the node leads both the Normal and Low read paths.
    pub fn add_preferred(
        mut self,
        url: impl Into<String>,
        name: impl Into<String>,
        tier: EndpointTier,
    ) -> Self {
        self.endpoints.push(
            TieredEndpoint::new(url, tier)
                .with_name(name)
                .with_priority(0)
                .with_preferred(true),
        );
        self
    }

    /// Add a trusted premium endpoint.
    pub fn add_premium_trusted(mut self, url: impl Into<String>, name: impl Into<String>) -> Self {
        let mut endpoint = TieredEndpoint::new(url, EndpointTier::Premium)
            .with_name(name)
            .with_priority(10);
        endpoint.trusted = true;
        self.endpoints.push(endpoint);
        self
    }

    /// Add a trusted standard endpoint.
    pub fn add_standard_trusted(
        mut self,
        url: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        let mut endpoint = TieredEndpoint::new(url, EndpointTier::Standard)
            .with_name(name)
            .with_priority(50);
        endpoint.trusted = true;
        self.endpoints.push(endpoint);
        self
    }

    /// Add a trusted free endpoint.
    pub fn add_free_trusted(mut self, url: impl Into<String>, name: impl Into<String>) -> Self {
        let mut endpoint = TieredEndpoint::new(url, EndpointTier::Free)
            .with_name(name)
            .with_priority(100);
        endpoint.trusted = true;
        self.endpoints.push(endpoint);
        self
    }

    /// Add a custom tiered endpoint.
    pub fn add_endpoint(mut self, endpoint: TieredEndpoint) -> Self {
        self.endpoints.push(endpoint);
        self
    }

    /// Add multiple free endpoints from a preset.
    ///
    /// Automatically adjusts endpoint priority based on capability grades
    /// when capability data is available (unknown capabilities = no adjustment).
    pub fn add_free_endpoints(mut self, endpoints: Vec<RpcEndpoint>) -> Self {
        for mut e in endpoints {
            let adjustment = e.capabilities.priority_adjustment();
            if adjustment != 0 {
                let new_priority = (e.priority as i32 + adjustment).max(0) as u32;
                debug!(
                    name = %e.name,
                    grade = %e.capabilities.grade(),
                    old_priority = e.priority,
                    new_priority = new_priority,
                    "Adjusting endpoint priority based on capabilities"
                );
                e.priority = new_priority;
            }
            self.endpoints.push(TieredEndpoint {
                endpoint: e,
                tier: EndpointTier::Free,
                rate_limit: 0,
                trusted: false,
            });
        }
        self
    }

    /// Automatically load built-in free public RPC endpoints for a chain.
    ///
    /// This adds all verified public endpoints from `presets::default_endpoints(chain_id)`
    /// to the Free tier. Call this to ensure you have fallback endpoints.
    ///
    /// # Example
    /// ```ignore
    /// let pool = TieredPoolBuilder::new()
    ///     .add_premium("https://my-alchemy.com/v2/key", "Alchemy")
    ///     .with_default_free_endpoints(42161)  // Load Arbitrum free endpoints
    ///     .build()?;
    /// ```
    pub fn with_default_free_endpoints(self, chain_id: u64) -> Self {
        let endpoints = presets::default_endpoints(chain_id);
        let count = endpoints.len();
        if count > 0 {
            info!(
                chain_id = chain_id,
                endpoint_count = count,
                "Loading default free endpoints for chain"
            );
        } else {
            warn!(
                chain_id = chain_id,
                "No default free endpoints found for chain"
            );
        }
        self.add_free_endpoints(endpoints)
    }

    /// Automatically load built-in free endpoints for multiple chains.
    pub fn with_default_free_endpoints_for_chains(mut self, chain_ids: &[u64]) -> Self {
        for &chain_id in chain_ids {
            self = self.with_default_free_endpoints(chain_id);
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
        // Deduplicate endpoints by (tier, URL). The same endpoint legitimately serves
        // multiple priority classes — e.g. a self-hosted node in BOTH Standard (Normal
        // reads) and Free (Low/background reads) — so dedup is per-tier, not global. A
        // global URL dedup silently dropped such an endpoint from the lower tier.
        let mut seen = HashSet::new();
        let mut deduped = Vec::with_capacity(self.endpoints.len());
        for ep in self.endpoints {
            if !seen.insert((ep.tier, ep.endpoint.url.clone())) {
                warn!(
                    url = %ep.endpoint.url,
                    name = %ep.endpoint.name,
                    tier = ?ep.tier,
                    "Duplicate endpoint removed during pool build (same url+tier)"
                );
                continue;
            }
            deduped.push(ep);
        }

        TieredPool::new(TieredPoolConfig {
            endpoints: deduped,
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

    #[test]
    fn test_tier_endpoint_counts() {
        let pool = TieredPoolBuilder::new()
            .add_premium("https://premium1.example.com", "Premium1")
            .add_standard("https://standard1.example.com", "Standard1")
            .add_standard("https://standard2.example.com", "Standard2")
            .add_free("https://free1.example.com", "Free1")
            .add_free("https://free2.example.com", "Free2")
            .add_free("https://free3.example.com", "Free3")
            .build()
            .unwrap();

        let counts = pool.tier_endpoint_counts();
        assert_eq!(counts.get(&EndpointTier::Premium), Some(&1));
        assert_eq!(counts.get(&EndpointTier::Standard), Some(&2));
        assert_eq!(counts.get(&EndpointTier::Free), Some(&3));
    }

    #[test]
    fn test_trusted_endpoint_creation() {
        let mut endpoint = TieredEndpoint::new("https://trusted.example.com", EndpointTier::Premium);
        assert!(!endpoint.trusted);

        endpoint.trusted = true;
        assert!(endpoint.trusted);
    }

    #[test]
    fn test_trusted_url_returns_url() {
        let pool = TieredPoolBuilder::new()
            .add_premium_trusted("https://trusted-premium.example.com", "Trusted Premium")
            .build()
            .unwrap();

        assert_eq!(
            pool.trusted_url(RequestPriority::Critical)
                .map(|url| url.to_string()),
            Some("https://trusted-premium.example.com/".to_string())
        );
    }

    #[test]
    fn test_trusted_url_returns_none() {
        let pool = TieredPoolBuilder::new()
            .add_premium("https://premium.example.com", "Premium")
            .build()
            .unwrap();

        assert!(pool.trusted_url(RequestPriority::Critical).is_none());
    }

    #[tokio::test]
    async fn test_mixed_trusted_and_pool() {
        let pool = TieredPoolBuilder::new()
            .add_premium_trusted("https://trusted-premium.example.com", "Trusted Premium")
            .add_free("https://free.example.com", "Free")
            .build()
            .unwrap();

        assert_eq!(
            pool.execute(RequestPriority::Critical, |url| async move {
                Ok::<_, std::io::Error>(url.to_string())
            })
            .await
            .unwrap(),
            "https://trusted-premium.example.com/"
        );

        assert_eq!(
            pool.execute(RequestPriority::Low, |url| async move {
                Ok::<_, std::io::Error>(url.to_string())
            })
            .await
            .unwrap(),
            "https://free.example.com/"
        );
    }

    #[test]
    fn test_trusted_tier_no_pool_created() {
        let pool = TieredPoolBuilder::new()
            .add_standard_trusted("https://trusted-standard.example.com", "Trusted Standard")
            .build()
            .unwrap();

        assert!(pool.get_tier_pool(EndpointTier::Standard).is_none());
        assert_eq!(
            pool.trusted_url(RequestPriority::Normal)
                .map(|url| url.to_string()),
            Some("https://trusted-standard.example.com/".to_string())
        );
    }

    #[test]
    fn test_normal_priority_includes_standard_and_free() {
        // Build pool with 2 Standard and 19 Free endpoints (similar to user's case)
        let mut builder = TieredPoolBuilder::new()
            .add_standard("https://standard1.example.com", "Standard1")
            .add_standard("https://standard2.example.com", "Standard2");

        // Add 19 Free endpoints
        for i in 1..=19 {
            builder = builder.add_free(
                format!("https://free{}.example.com", i),
                format!("Free{}", i),
            );
        }

        let pool = builder.build().unwrap();

        // Verify tier configuration
        let counts = pool.tier_endpoint_counts();
        assert_eq!(counts.get(&EndpointTier::Standard), Some(&2));
        assert_eq!(counts.get(&EndpointTier::Free), Some(&19));

        // Verify Normal priority will try both Standard and Free
        let tiers = pool.tier_order(RequestPriority::Normal);
        assert_eq!(tiers.len(), 2);
        assert_eq!(tiers[0], EndpointTier::Standard);
        assert_eq!(tiers[1], EndpointTier::Free);

        // Verify both tier pools exist
        assert!(pool.has_tier(EndpointTier::Standard));
        assert!(pool.has_tier(EndpointTier::Free));
    }

    #[test]
    fn test_with_default_free_endpoints() {
        use crate::presets::chain_id;

        let pool = TieredPoolBuilder::new()
            .add_premium("https://premium.example.com", "Premium")
            .with_default_free_endpoints(chain_id::ARBITRUM_ONE)
            .build()
            .unwrap();

        // Verify Free tier was created with Arbitrum endpoints
        assert!(pool.has_tier(EndpointTier::Free));
        let counts = pool.tier_endpoint_counts();
        let free_count = counts.get(&EndpointTier::Free).unwrap_or(&0);
        assert!(*free_count > 0, "Free tier should have endpoints loaded");

        // Should match the number from presets
        let preset_count = crate::presets::arbitrum_endpoints().len();
        assert_eq!(*free_count, preset_count);
    }

    #[test]
    fn test_with_default_free_endpoints_for_chains() {
        use crate::presets::chain_id;

        let pool = TieredPoolBuilder::new()
            .add_premium("https://premium.example.com", "Premium")
            .with_default_free_endpoints_for_chains(&[
                chain_id::ETHEREUM,
                chain_id::ARBITRUM_ONE,
            ])
            .build()
            .unwrap();

        let counts = pool.tier_endpoint_counts();
        let free_count = counts.get(&EndpointTier::Free).unwrap_or(&0);

        let expected_count = crate::presets::ethereum_endpoints().len()
            + crate::presets::arbitrum_endpoints().len();
        assert_eq!(*free_count, expected_count);
    }

    #[test]
    fn test_same_url_kept_across_tiers_deduped_within_tier() {
        let node = "http://node.local:8545";
        let pool = TieredPoolBuilder::new()
            .add_premium("https://premium.example.com", "Premium")
            .add_standard(node, "Node-Std")
            .add_free(node, "Node-Free") // same URL, different tier -> kept
            .add_free(node, "Node-Free-dup") // same URL, same tier -> deduped
            .add_free("https://other-free.example.com", "Other")
            .build()
            .unwrap();
        let counts = pool.tier_endpoint_counts();
        // Node survives in BOTH Standard and Free (cross-tier dedup removed).
        assert_eq!(*counts.get(&EndpointTier::Standard).unwrap(), 1);
        // Free has the node once (same-tier dup removed) + the other endpoint = 2.
        assert_eq!(*counts.get(&EndpointTier::Free).unwrap(), 2);
    }

    #[test]
    fn test_add_preferred_marks_endpoint_and_top_priority() {
        // with_preferred only sets the flag.
        let ep = TieredEndpoint::new("http://node.local:8545", EndpointTier::Free)
            .with_preferred(true);
        assert!(ep.endpoint.preferred);

        // The add_preferred builder convenience also sets top priority (0).
        let built = TieredPoolBuilder::new()
            .add_preferred("http://node.local:8545", "Node", EndpointTier::Standard)
            .endpoints[0]
            .clone();
        assert!(built.endpoint.preferred);
        assert_eq!(built.endpoint.priority, 0);
        assert_eq!(built.tier, EndpointTier::Standard);
    }
}
