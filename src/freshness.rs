//! Computes health-aware endpoint head freshness.
//! Exports the internal freshness reference and diagnostic distribution.

use crate::endpoint::EndpointStats;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FreshnessDistribution {
    pub(crate) best_known_block: Option<u64>,
    pub(crate) fresh: usize,
    pub(crate) lagging: usize,
    pub(crate) unknown: usize,
    pub(crate) unhealthy: usize,
}

pub(crate) fn best_known_healthy_block(stats: &HashMap<String, EndpointStats>) -> Option<u64> {
    stats
        .values()
        .filter(|stats| stats.is_healthy)
        .filter_map(|stats| stats.latest_block_number)
        .max()
}

pub(crate) fn freshness_distribution(
    stats: &HashMap<String, EndpointStats>,
    max_block_lag: u64,
) -> FreshnessDistribution {
    let best_known_block = best_known_healthy_block(stats);
    let mut distribution = FreshnessDistribution {
        best_known_block,
        fresh: 0,
        lagging: 0,
        unknown: 0,
        unhealthy: 0,
    };
    for endpoint in stats.values() {
        if !endpoint.is_healthy {
            distribution.unhealthy += 1;
            continue;
        }
        match (best_known_block, endpoint.latest_block_number) {
            (Some(best), Some(block)) if best.saturating_sub(block) <= max_block_lag => {
                distribution.fresh += 1;
            }
            (Some(_), Some(_)) => distribution.lagging += 1,
            _ => distribution.unknown += 1,
        }
    }
    distribution
}

#[cfg(test)]
mod tests {
    use super::{freshness_distribution, FreshnessDistribution};
    use crate::endpoint::{EndpointStats, RpcEndpoint};
    use std::collections::HashMap;

    #[test]
    fn distribution_ignores_unhealthy_high_head() {
        let endpoints = [
            RpcEndpoint::new("https://unhealthy.rpc"),
            RpcEndpoint::new("https://current.rpc"),
            RpcEndpoint::new("https://lagging.rpc"),
        ];
        let mut stats = endpoints
            .iter()
            .map(|endpoint| (endpoint.url.clone(), EndpointStats::new(endpoint)))
            .collect::<HashMap<_, _>>();
        let unhealthy = stats.get_mut("https://unhealthy.rpc").unwrap();
        unhealthy.is_healthy = false;
        unhealthy.latest_block_number = Some(1_000);
        stats
            .get_mut("https://current.rpc")
            .unwrap()
            .latest_block_number = Some(100);
        stats
            .get_mut("https://lagging.rpc")
            .unwrap()
            .latest_block_number = Some(90);

        assert_eq!(
            freshness_distribution(&stats, 2),
            FreshnessDistribution {
                best_known_block: Some(100),
                fresh: 1,
                lagging: 1,
                unknown: 0,
                unhealthy: 1,
            }
        );
    }
}
