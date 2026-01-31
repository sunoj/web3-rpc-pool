//! Rate-aware strategy - distributes load while respecting rate limits.
//!
//! This strategy tracks the last request time for each endpoint and
//! selects the endpoint that has been idle the longest, naturally
//! distributing load across all available endpoints.

use super::SelectionStrategy;
use crate::endpoint::{EndpointStats, RpcEndpoint};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// Rate-aware selection strategy.
///
/// Tracks last request time per endpoint and selects the one that
/// has been idle longest. This naturally distributes load across
/// all endpoints and helps stay within rate limits.
///
/// Best for: Free tier RPCs where you want to maximize throughput
/// while staying within each provider's rate limits.
pub struct RateAwareStrategy {
    /// Last request time for each endpoint URL.
    last_request: RwLock<HashMap<String, Instant>>,

    /// Minimum interval between requests to the same endpoint.
    /// Default: 1 second (allows 1 req/s per endpoint).
    min_interval: Duration,
}

impl Default for RateAwareStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RateAwareStrategy {
    /// Create a new rate-aware strategy with default 1 second interval.
    pub fn new() -> Self {
        Self {
            last_request: RwLock::new(HashMap::new()),
            min_interval: Duration::from_secs(1),
        }
    }

    /// Create with custom minimum interval between requests to same endpoint.
    ///
    /// For example, if you have 10 endpoints and want 10 req/s total,
    /// set interval to 1 second (each endpoint gets 1 req/s).
    pub fn with_min_interval(min_interval: Duration) -> Self {
        Self {
            last_request: RwLock::new(HashMap::new()),
            min_interval,
        }
    }

    /// Record that a request was made to an endpoint.
    pub fn record_request(&self, url: &str) {
        self.last_request
            .write()
            .insert(url.to_string(), Instant::now());
    }

    /// Get time since last request to an endpoint.
    fn time_since_last(&self, url: &str) -> Duration {
        self.last_request
            .read()
            .get(url)
            .map(|t| t.elapsed())
            .unwrap_or(Duration::MAX) // Never used = maximum idle time
    }

    /// Check if endpoint is ready (enough time passed since last request).
    fn is_ready(&self, url: &str) -> bool {
        self.time_since_last(url) >= self.min_interval
    }
}

impl SelectionStrategy for RateAwareStrategy {
    fn select<'a>(
        &mut self,
        endpoints: &'a [RpcEndpoint],
        stats: &HashMap<String, EndpointStats>,
        exclude: &HashSet<String>,
    ) -> Option<&'a RpcEndpoint> {
        // Collect healthy, non-excluded endpoints with their idle time
        let mut candidates: Vec<_> = endpoints
            .iter()
            .filter(|e| !exclude.contains(&e.url))
            .filter(|e| stats.get(&e.url).map(|s| s.is_healthy).unwrap_or(true))
            .map(|e| (e, self.time_since_last(&e.url)))
            .collect();

        if candidates.is_empty() {
            // Fallback: any non-excluded endpoint
            return endpoints.iter().find(|e| !exclude.contains(&e.url));
        }

        // Sort by idle time descending (longest idle first)
        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        // Select the endpoint that has been idle longest
        let selected = candidates.first().map(|(e, _)| *e)?;

        // Record this selection
        self.record_request(&selected.url);

        Some(selected)
    }

    fn name(&self) -> &'static str {
        "rate-aware"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    fn create_test_endpoints() -> Vec<RpcEndpoint> {
        vec![
            RpcEndpoint::new("https://rpc1.example.com").with_name("RPC1"),
            RpcEndpoint::new("https://rpc2.example.com").with_name("RPC2"),
            RpcEndpoint::new("https://rpc3.example.com").with_name("RPC3"),
        ]
    }

    fn create_stats(endpoints: &[RpcEndpoint]) -> HashMap<String, EndpointStats> {
        endpoints
            .iter()
            .map(|e| (e.url.clone(), EndpointStats::new(e)))
            .collect()
    }

    #[test]
    fn test_selects_idle_endpoint() {
        let mut strategy = RateAwareStrategy::with_min_interval(Duration::from_millis(10));
        let endpoints = create_test_endpoints();
        let stats = create_stats(&endpoints);
        let exclude = HashSet::new();

        // First selection - any endpoint (all have max idle time)
        let first = strategy.select(&endpoints, &stats, &exclude).unwrap();
        let first_url = first.url.clone();

        // Second selection should pick a different endpoint (first one just used)
        let second = strategy.select(&endpoints, &stats, &exclude).unwrap();
        assert_ne!(first_url, second.url);

        // Third selection should pick the remaining endpoint
        let third = strategy.select(&endpoints, &stats, &exclude).unwrap();
        assert_ne!(first_url, third.url);
        assert_ne!(second.url, third.url);
    }

    #[test]
    fn test_respects_min_interval() {
        let mut strategy = RateAwareStrategy::with_min_interval(Duration::from_millis(50));
        let endpoints = vec![RpcEndpoint::new("https://rpc1.example.com")];
        let stats = create_stats(&endpoints);
        let exclude = HashSet::new();

        // First request
        strategy.select(&endpoints, &stats, &exclude);

        // Check readiness
        assert!(!strategy.is_ready("https://rpc1.example.com"));

        // Wait for interval
        sleep(Duration::from_millis(60));

        assert!(strategy.is_ready("https://rpc1.example.com"));
    }

    #[test]
    fn test_cycles_through_all_endpoints() {
        let mut strategy = RateAwareStrategy::with_min_interval(Duration::from_millis(1));
        let endpoints = create_test_endpoints();
        let stats = create_stats(&endpoints);
        let exclude = HashSet::new();

        let mut seen = HashSet::new();

        // Make 3 selections
        for _ in 0..3 {
            let selected = strategy.select(&endpoints, &stats, &exclude).unwrap();
            seen.insert(selected.url.clone());
            sleep(Duration::from_millis(2)); // Small delay to ensure ordering
        }

        // Should have used all 3 endpoints
        assert_eq!(seen.len(), 3);
    }
}
