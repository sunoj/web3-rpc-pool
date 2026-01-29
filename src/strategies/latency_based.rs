//! Latency-based strategy - selects the fastest endpoint.

use super::SelectionStrategy;
use crate::endpoint::{EndpointStats, RpcEndpoint};
use std::collections::{HashMap, HashSet};

/// Latency-based selection strategy.
///
/// Always selects the healthy endpoint with the lowest average latency.
/// Uses exponential moving average (EMA) to smooth out latency measurements.
///
/// Best for: Latency-sensitive applications where response time is critical.
#[derive(Debug, Default, Clone)]
pub struct LatencyBasedStrategy;

impl SelectionStrategy for LatencyBasedStrategy {
    fn select<'a>(
        &mut self,
        endpoints: &'a [RpcEndpoint],
        stats: &HashMap<String, EndpointStats>,
        exclude: &HashSet<String>,
    ) -> Option<&'a RpcEndpoint> {
        // Collect healthy, non-excluded endpoints with their latencies
        let mut healthy: Vec<_> = endpoints
            .iter()
            .filter(|e| !exclude.contains(&e.url))
            .filter(|e| stats.get(&e.url).map(|s| s.is_healthy).unwrap_or(true))
            .collect();

        if healthy.is_empty() {
            // Fallback: any non-excluded endpoint
            return endpoints.iter().find(|e| !exclude.contains(&e.url));
        }

        // Sort by average latency (ascending)
        healthy.sort_by(|a, b| {
            let lat_a = stats.get(&a.url).map(|s| s.avg_latency_ms).unwrap_or(f64::MAX);
            let lat_b = stats.get(&b.url).map(|s| s.avg_latency_ms).unwrap_or(f64::MAX);
            lat_a.partial_cmp(&lat_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        healthy.first().copied()
    }

    fn name(&self) -> &'static str {
        "latency-based"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_endpoints() -> Vec<RpcEndpoint> {
        vec![
            RpcEndpoint::new("https://slow.rpc").with_name("Slow"),
            RpcEndpoint::new("https://fast.rpc").with_name("Fast"),
            RpcEndpoint::new("https://medium.rpc").with_name("Medium"),
        ]
    }

    #[test]
    fn test_selects_lowest_latency() {
        let mut strategy = LatencyBasedStrategy;
        let endpoints = create_test_endpoints();

        let mut stats: HashMap<String, EndpointStats> = endpoints
            .iter()
            .map(|e| (e.url.clone(), EndpointStats::new(e)))
            .collect();

        // Set different latencies
        stats.get_mut("https://slow.rpc").unwrap().avg_latency_ms = 500.0;
        stats.get_mut("https://fast.rpc").unwrap().avg_latency_ms = 50.0;
        stats.get_mut("https://medium.rpc").unwrap().avg_latency_ms = 200.0;

        let exclude = HashSet::new();
        let selected = strategy.select(&endpoints, &stats, &exclude);
        assert_eq!(selected.unwrap().url, "https://fast.rpc");
    }

    #[test]
    fn test_prefers_no_data_over_high_latency() {
        let mut strategy = LatencyBasedStrategy;
        let endpoints = create_test_endpoints();

        let mut stats: HashMap<String, EndpointStats> = endpoints
            .iter()
            .map(|e| (e.url.clone(), EndpointStats::new(e)))
            .collect();

        // Only set high latency for one
        stats.get_mut("https://slow.rpc").unwrap().avg_latency_ms = 500.0;
        // Others have 0.0 (no data yet)

        let exclude = HashSet::new();
        let selected = strategy.select(&endpoints, &stats, &exclude);
        // Should not select the slow one
        assert_ne!(selected.unwrap().url, "https://slow.rpc");
    }
}
