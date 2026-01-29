//! Metrics collection for monitoring RPC pool performance.

use crate::endpoint::EndpointStats;
use serde::{Deserialize, Serialize};

/// Aggregated metrics for the RPC pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcPoolMetrics {
    /// Total number of requests made through the pool.
    pub total_requests: u64,

    /// Number of times failover occurred.
    pub failovers: u64,

    /// Name of the current primary endpoint.
    pub current_endpoint: String,

    /// Statistics for each endpoint.
    pub endpoints: Vec<EndpointMetrics>,
}

/// Serializable endpoint metrics (subset of EndpointStats).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetrics {
    pub url: String,
    pub name: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_latency_ms: f64,
    pub last_latency_ms: u64,
    pub is_healthy: bool,
    pub consecutive_errors: u32,
    pub success_rate: f64,
}

impl From<&EndpointStats> for EndpointMetrics {
    fn from(stats: &EndpointStats) -> Self {
        Self {
            url: stats.url.clone(),
            name: stats.name.clone(),
            total_requests: stats.total_requests,
            successful_requests: stats.successful_requests,
            failed_requests: stats.failed_requests,
            avg_latency_ms: stats.avg_latency_ms,
            last_latency_ms: stats.last_latency_ms,
            is_healthy: stats.is_healthy,
            consecutive_errors: stats.consecutive_errors,
            success_rate: stats.success_rate(),
        }
    }
}

impl RpcPoolMetrics {
    /// Get the total success rate across all endpoints.
    pub fn total_success_rate(&self) -> f64 {
        let total: u64 = self.endpoints.iter().map(|e| e.total_requests).sum();
        let successful: u64 = self.endpoints.iter().map(|e| e.successful_requests).sum();

        if total == 0 {
            return 100.0;
        }
        (successful as f64 / total as f64) * 100.0
    }

    /// Get the number of healthy endpoints.
    pub fn healthy_count(&self) -> usize {
        self.endpoints.iter().filter(|e| e.is_healthy).count()
    }

    /// Get the average latency across all endpoints.
    pub fn avg_latency(&self) -> f64 {
        let healthy: Vec<_> = self
            .endpoints
            .iter()
            .filter(|e| e.is_healthy && e.avg_latency_ms > 0.0)
            .collect();

        if healthy.is_empty() {
            return 0.0;
        }

        let sum: f64 = healthy.iter().map(|e| e.avg_latency_ms).sum();
        sum / healthy.len() as f64
    }
}
