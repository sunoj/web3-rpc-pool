//! Failover strategy - uses highest priority healthy endpoint.

use super::SelectionStrategy;
use crate::endpoint::{EndpointStats, RpcEndpoint};
use std::collections::{HashMap, HashSet};

/// Failover selection strategy.
///
/// Always selects the highest priority (lowest priority number) healthy endpoint.
/// Falls back to any available endpoint if all are unhealthy.
///
/// Best for: Production systems with a clear primary endpoint and backups.
#[derive(Debug, Default, Clone)]
pub struct FailoverStrategy;

impl SelectionStrategy for FailoverStrategy {
    fn select<'a>(
        &mut self,
        endpoints: &'a [RpcEndpoint],
        stats: &HashMap<String, EndpointStats>,
        exclude: &HashSet<String>,
    ) -> Option<&'a RpcEndpoint> {
        // Try to find a healthy, non-excluded endpoint (endpoints are pre-sorted by priority)
        let healthy = endpoints
            .iter()
            .filter(|e| !exclude.contains(&e.url))
            .find(|e| stats.get(&e.url).map(|s| s.is_healthy).unwrap_or(true));

        // Fallback: any non-excluded endpoint
        healthy.or_else(|| endpoints.iter().find(|e| !exclude.contains(&e.url)))
    }

    fn name(&self) -> &'static str {
        "failover"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_endpoints() -> Vec<RpcEndpoint> {
        vec![
            RpcEndpoint::new("https://primary.rpc").with_priority(10),
            RpcEndpoint::new("https://secondary.rpc").with_priority(50),
            RpcEndpoint::new("https://tertiary.rpc").with_priority(100),
        ]
    }

    fn create_stats(endpoints: &[RpcEndpoint]) -> HashMap<String, EndpointStats> {
        endpoints
            .iter()
            .map(|e| (e.url.clone(), EndpointStats::new(e)))
            .collect()
    }

    #[test]
    fn test_selects_highest_priority() {
        let mut strategy = FailoverStrategy;
        let endpoints = create_test_endpoints();
        let stats = create_stats(&endpoints);
        let exclude = HashSet::new();

        let selected = strategy.select(&endpoints, &stats, &exclude);
        assert_eq!(selected.unwrap().url, "https://primary.rpc");
    }

    #[test]
    fn test_skips_unhealthy() {
        let mut strategy = FailoverStrategy;
        let endpoints = create_test_endpoints();
        let mut stats = create_stats(&endpoints);

        // Mark primary as unhealthy
        stats.get_mut("https://primary.rpc").unwrap().is_healthy = false;

        let exclude = HashSet::new();
        let selected = strategy.select(&endpoints, &stats, &exclude);
        assert_eq!(selected.unwrap().url, "https://secondary.rpc");
    }

    #[test]
    fn test_fallback_when_all_unhealthy() {
        let mut strategy = FailoverStrategy;
        let endpoints = create_test_endpoints();
        let mut stats = create_stats(&endpoints);

        // Mark all as unhealthy
        for stat in stats.values_mut() {
            stat.is_healthy = false;
        }

        let exclude = HashSet::new();
        let selected = strategy.select(&endpoints, &stats, &exclude);
        // Should still return something (graceful degradation)
        assert!(selected.is_some());
    }
}
