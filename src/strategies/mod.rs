//! Endpoint selection strategies.
//!
//! This module provides different algorithms for selecting which RPC endpoint
//! to use for a request.
//!
//! ## Available Strategies
//!
//! - [`FailoverStrategy`]: Uses primary endpoint, switches on failure (best for premium tier)
//! - [`RoundRobinStrategy`]: Cycles through endpoints evenly (good for load distribution)
//! - [`LatencyBasedStrategy`]: Selects fastest endpoint (best for latency-sensitive ops)
//! - [`RateAwareStrategy`]: Tracks usage per endpoint, selects least recently used (best for free tier)

mod failover;
mod latency_based;
mod rate_aware;
mod round_robin;

pub use failover::FailoverStrategy;
pub use latency_based::LatencyBasedStrategy;
pub use rate_aware::RateAwareStrategy;
pub use round_robin::RoundRobinStrategy;

use crate::endpoint::{EndpointStats, RpcEndpoint};
use std::collections::{HashMap, HashSet};

/// Trait for endpoint selection strategies.
///
/// Implement this trait to create custom selection algorithms.
pub trait SelectionStrategy: Send + Sync {
    /// Select the next endpoint to use.
    ///
    /// # Arguments
    ///
    /// * `endpoints` - All configured endpoints (sorted by priority).
    /// * `stats` - Current statistics for each endpoint.
    /// * `exclude` - URLs to exclude (already tried in current request).
    ///
    /// # Returns
    ///
    /// The selected endpoint, or `None` if no suitable endpoint is available.
    fn select<'a>(
        &mut self,
        endpoints: &'a [RpcEndpoint],
        stats: &HashMap<String, EndpointStats>,
        exclude: &HashSet<String>,
    ) -> Option<&'a RpcEndpoint>;

    /// Name of this strategy for logging.
    fn name(&self) -> &'static str;
}
