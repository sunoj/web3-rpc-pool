//! Round-robin strategy - cycles through endpoints evenly.

use super::SelectionStrategy;
use crate::endpoint::{EndpointStats, RpcEndpoint};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Round-robin selection strategy.
///
/// Cycles through healthy endpoints to distribute load evenly.
/// Skips unhealthy endpoints.
///
/// Best for: Load balancing across multiple equivalent endpoints.
#[derive(Debug, Default)]
pub struct RoundRobinStrategy {
    current_index: AtomicUsize,
}

impl RoundRobinStrategy {
    /// Create a new round-robin strategy.
    pub fn new() -> Self {
        Self {
            current_index: AtomicUsize::new(0),
        }
    }
}

impl SelectionStrategy for RoundRobinStrategy {
    fn select<'a>(
        &mut self,
        endpoints: &'a [RpcEndpoint],
        stats: &HashMap<String, EndpointStats>,
        exclude: &HashSet<String>,
    ) -> Option<&'a RpcEndpoint> {
        // Collect healthy, non-excluded endpoints
        let healthy: Vec<_> = endpoints
            .iter()
            .filter(|e| !exclude.contains(&e.url))
            .filter(|e| stats.get(&e.url).map(|s| s.is_healthy).unwrap_or(true))
            .collect();

        if healthy.is_empty() {
            // Fallback: any non-excluded endpoint
            return endpoints.iter().find(|e| !exclude.contains(&e.url));
        }

        // Get and increment index atomically
        let idx = self.current_index.fetch_add(1, Ordering::Relaxed) % healthy.len();
        Some(healthy[idx])
    }

    fn name(&self) -> &'static str {
        "round-robin"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_endpoints() -> Vec<RpcEndpoint> {
        vec![
            RpcEndpoint::new("https://rpc1.example.com"),
            RpcEndpoint::new("https://rpc2.example.com"),
            RpcEndpoint::new("https://rpc3.example.com"),
        ]
    }

    fn create_stats(endpoints: &[RpcEndpoint]) -> HashMap<String, EndpointStats> {
        endpoints
            .iter()
            .map(|e| (e.url.clone(), EndpointStats::new(e)))
            .collect()
    }

    #[test]
    fn test_cycles_through_endpoints() {
        let mut strategy = RoundRobinStrategy::new();
        let endpoints = create_test_endpoints();
        let stats = create_stats(&endpoints);
        let exclude = HashSet::new();

        // Should cycle through all endpoints
        let first = strategy.select(&endpoints, &stats, &exclude).unwrap().url.clone();
        let second = strategy.select(&endpoints, &stats, &exclude).unwrap().url.clone();
        let third = strategy.select(&endpoints, &stats, &exclude).unwrap().url.clone();
        let fourth = strategy.select(&endpoints, &stats, &exclude).unwrap().url.clone();

        // Fourth should wrap around to first
        assert_eq!(first, fourth);
        // All three should be different
        assert_ne!(first, second);
        assert_ne!(second, third);
    }
}
