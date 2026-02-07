//! RPC endpoint definitions and statistics tracking.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Instant;

/// Capability metadata for an RPC endpoint.
///
/// Tracks what features an endpoint supports, enabling quality-based
/// endpoint selection and grading.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EndpointCapabilities {
    /// Whether the endpoint supports `eth_getLogs`. `None` = untested.
    pub supports_eth_get_logs: Option<bool>,

    /// Maximum batch request size. `None` = unknown, `Some(0)` = unlimited.
    pub max_batch_size: Option<u32>,

    /// Maximum block range for `eth_getLogs`. `None` = unknown, `Some(0)` = unlimited.
    pub max_block_range: Option<u64>,

    /// Whether the endpoint supports `debug_traceTransaction`. `None` = untested.
    pub supports_debug_trace: Option<bool>,

    /// Whether the endpoint supports WebSocket connections (derived from ws_url).
    #[serde(default)]
    pub supports_websocket: bool,

    /// Known rate limit in requests per second. `None` = unknown.
    pub rate_limit_rps: Option<u32>,
}

impl EndpointCapabilities {
    /// Compute a quality grade based on known capabilities.
    pub fn grade(&self) -> EndpointGrade {
        // If we have no data at all, grade as D (unknown)
        let has_any_data = self.supports_eth_get_logs.is_some()
            || self.max_batch_size.is_some()
            || self.max_block_range.is_some();

        if !has_any_data {
            return EndpointGrade::D;
        }

        let supports_logs = self.supports_eth_get_logs.unwrap_or(false);

        if !supports_logs {
            return EndpointGrade::D;
        }

        // Has eth_getLogs support — now check batch and range
        let batch = self.max_batch_size.unwrap_or(0); // 0 = unlimited
        let range = self.max_block_range.unwrap_or(0); // 0 = unlimited

        let batch_ok_a = batch == 0 || batch >= 100;
        let range_ok_a = range == 0 || range >= 10_000;

        if batch_ok_a && range_ok_a {
            return EndpointGrade::A;
        }

        let batch_ok_b = batch == 0 || batch >= 10;
        let range_ok_b = range == 0 || range >= 1_000;

        if batch_ok_b && range_ok_b {
            return EndpointGrade::B;
        }

        EndpointGrade::C
    }

    /// Return a priority adjustment value based on grade.
    ///
    /// Lower values mean higher priority. Returns 0 when capabilities are unknown.
    /// - Grade A: -20 (highest priority)
    /// - Grade B: -10
    /// - Grade C: 0
    /// - Grade D: +10
    /// - Grade F: +50 (lowest priority)
    pub fn priority_adjustment(&self) -> i32 {
        match self.grade() {
            EndpointGrade::A => -20,
            EndpointGrade::B => -10,
            EndpointGrade::C => 0,
            EndpointGrade::D => {
                // Only penalize if we actually tested and found lacking;
                // if untested, no adjustment
                let has_any_data = self.supports_eth_get_logs.is_some()
                    || self.max_batch_size.is_some()
                    || self.max_block_range.is_some();
                if has_any_data { 10 } else { 0 }
            }
            EndpointGrade::F => 50,
        }
    }
}

/// Quality grade for an RPC endpoint (F < D < C < B < A).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EndpointGrade {
    /// Unreachable endpoint.
    F = 0,
    /// No eth_getLogs or all capabilities unknown.
    D = 1,
    /// eth_getLogs with limited batch/range.
    C = 2,
    /// eth_getLogs + batch >= 10 + block range >= 1,000.
    B = 3,
    /// eth_getLogs + batch >= 100 + block range >= 10,000.
    A = 4,
}

impl fmt::Display for EndpointGrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EndpointGrade::A => write!(f, "A"),
            EndpointGrade::B => write!(f, "B"),
            EndpointGrade::C => write!(f, "C"),
            EndpointGrade::D => write!(f, "D"),
            EndpointGrade::F => write!(f, "F"),
        }
    }
}

/// Configuration for a single RPC endpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RpcEndpoint {
    /// HTTP/HTTPS RPC URL.
    pub url: String,

    /// WebSocket URL (optional, for subscriptions).
    #[serde(default)]
    pub ws_url: Option<String>,

    /// Human-readable name for logging and metrics.
    #[serde(default = "default_name")]
    pub name: String,

    /// Priority for failover strategy (lower = higher priority).
    #[serde(default = "default_priority")]
    pub priority: u32,

    /// Chain ID this endpoint serves.
    #[serde(default)]
    pub chain_id: u64,

    /// Capability metadata (supports backward-compatible deserialization).
    #[serde(default)]
    pub capabilities: EndpointCapabilities,
}

fn default_name() -> String {
    "unnamed".to_string()
}

fn default_priority() -> u32 {
    100
}

impl RpcEndpoint {
    /// Create a new endpoint with minimal configuration.
    pub fn new(url: impl Into<String>) -> Self {
        let url = url.into();
        Self {
            name: url.clone(),
            url,
            ws_url: None,
            priority: 100,
            chain_id: 0,
            capabilities: EndpointCapabilities::default(),
        }
    }

    /// Builder: set endpoint name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Builder: set WebSocket URL.
    pub fn with_ws_url(mut self, ws_url: impl Into<String>) -> Self {
        self.ws_url = Some(ws_url.into());
        self.capabilities.supports_websocket = true;
        self
    }

    /// Builder: set priority (lower = higher priority).
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Builder: set chain ID.
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = chain_id;
        self
    }

    /// Builder: set endpoint capabilities.
    pub fn with_capabilities(mut self, capabilities: EndpointCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }
}

/// Runtime statistics for an RPC endpoint.
#[derive(Debug, Clone)]
pub struct EndpointStats {
    /// Endpoint URL (key).
    pub url: String,

    /// Endpoint name.
    pub name: String,

    /// Total number of requests sent.
    pub total_requests: u64,

    /// Number of successful requests.
    pub successful_requests: u64,

    /// Number of failed requests.
    pub failed_requests: u64,

    /// Exponential moving average of latency in milliseconds.
    pub avg_latency_ms: f64,

    /// Latency of the most recent request.
    pub last_latency_ms: u64,

    /// Most recent error message (if any).
    pub last_error: Option<String>,

    /// Timestamp of the most recent error.
    pub last_error_time: Option<Instant>,

    /// Whether the endpoint is currently considered healthy.
    pub is_healthy: bool,

    /// Number of consecutive errors (resets on success).
    pub consecutive_errors: u32,

    /// Number of consecutive recovery failures (for exponential backoff).
    pub recovery_attempts: u32,
}

/// Maximum recovery backoff duration (5 minutes).
const MAX_RECOVERY_BACKOFF_SECS: u64 = 300;

impl EndpointStats {
    /// Create new stats for an endpoint.
    pub fn new(endpoint: &RpcEndpoint) -> Self {
        Self {
            url: endpoint.url.clone(),
            name: endpoint.name.clone(),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_latency_ms: 0.0,
            last_latency_ms: 0,
            last_error: None,
            last_error_time: None,
            is_healthy: true,
            consecutive_errors: 0,
            recovery_attempts: 0,
        }
    }

    /// Update latency using exponential moving average.
    ///
    /// Uses 90% historical weight + 10% current weight for smoothing.
    pub fn update_latency(&mut self, latency_ms: u64) {
        self.last_latency_ms = latency_ms;
        if self.avg_latency_ms == 0.0 {
            self.avg_latency_ms = latency_ms as f64;
        } else {
            // EMA: 90% history + 10% current
            self.avg_latency_ms = self.avg_latency_ms * 0.9 + latency_ms as f64 * 0.1;
        }
    }

    /// Record a successful request.
    pub fn record_success(&mut self, latency_ms: u64) {
        self.total_requests += 1;
        self.successful_requests += 1;
        self.update_latency(latency_ms);
        self.consecutive_errors = 0;
        self.is_healthy = true;
    }

    /// Record a failed request.
    ///
    /// Returns `true` if the endpoint should be marked unhealthy.
    pub fn record_failure(&mut self, error: String, max_consecutive: u32) -> bool {
        self.total_requests += 1;
        self.failed_requests += 1;
        self.consecutive_errors += 1;
        self.last_error = Some(error);
        self.last_error_time = Some(Instant::now());

        if self.consecutive_errors >= max_consecutive {
            self.is_healthy = false;
            true
        } else {
            false
        }
    }

    /// Calculate success rate as a percentage.
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 100.0;
        }
        (self.successful_requests as f64 / self.total_requests as f64) * 100.0
    }

    /// Check if enough time has passed since the last error for a retry.
    ///
    /// Uses exponential backoff: base_delay * 2^recovery_attempts, capped at 5 minutes.
    pub fn can_retry(&self, base_retry_delay: std::time::Duration) -> bool {
        match &self.last_error_time {
            Some(t) => {
                let backoff_multiplier = 2u64.saturating_pow(self.recovery_attempts);
                let backoff_secs = base_retry_delay.as_secs().saturating_mul(backoff_multiplier);
                let capped_secs = backoff_secs.min(MAX_RECOVERY_BACKOFF_SECS);
                let actual_delay = std::time::Duration::from_secs(capped_secs);
                t.elapsed() >= actual_delay
            }
            None => true,
        }
    }

    /// Get the current retry delay with exponential backoff applied.
    pub fn current_retry_delay(&self, base_retry_delay: std::time::Duration) -> std::time::Duration {
        let backoff_multiplier = 2u64.saturating_pow(self.recovery_attempts);
        let backoff_secs = base_retry_delay.as_secs().saturating_mul(backoff_multiplier);
        let capped_secs = backoff_secs.min(MAX_RECOVERY_BACKOFF_SECS);
        std::time::Duration::from_secs(capped_secs)
    }

    /// Increment recovery attempts (called when health check fails).
    pub fn increment_recovery_attempts(&mut self) {
        // Cap at a reasonable max to prevent overflow
        if self.recovery_attempts < 10 {
            self.recovery_attempts += 1;
        }
    }

    /// Mark as recovered (healthy again).
    pub fn mark_recovered(&mut self) {
        self.is_healthy = true;
        self.consecutive_errors = 0;
        self.recovery_attempts = 0; // Reset backoff on successful recovery
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_builder() {
        let endpoint = RpcEndpoint::new("https://rpc.example.com")
            .with_name("Example RPC")
            .with_priority(50)
            .with_chain_id(1);

        assert_eq!(endpoint.url, "https://rpc.example.com");
        assert_eq!(endpoint.name, "Example RPC");
        assert_eq!(endpoint.priority, 50);
        assert_eq!(endpoint.chain_id, 1);
    }

    #[test]
    fn test_ema_latency() {
        let endpoint = RpcEndpoint::new("https://rpc.example.com");
        let mut stats = EndpointStats::new(&endpoint);

        // First request sets the baseline
        stats.update_latency(100);
        assert_eq!(stats.avg_latency_ms, 100.0);

        // Second request uses EMA
        stats.update_latency(200);
        // 100 * 0.9 + 200 * 0.1 = 90 + 20 = 110
        assert!((stats.avg_latency_ms - 110.0).abs() < 0.001);
    }

    #[test]
    fn test_consecutive_errors() {
        let endpoint = RpcEndpoint::new("https://rpc.example.com");
        let mut stats = EndpointStats::new(&endpoint);

        // First two errors don't mark unhealthy
        assert!(!stats.record_failure("error 1".into(), 3));
        assert!(!stats.record_failure("error 2".into(), 3));
        assert!(stats.is_healthy);

        // Third error marks unhealthy
        assert!(stats.record_failure("error 3".into(), 3));
        assert!(!stats.is_healthy);

        // Success resets
        stats.record_success(100);
        assert!(stats.is_healthy);
        assert_eq!(stats.consecutive_errors, 0);
    }

    #[test]
    fn test_endpoint_capabilities_default() {
        let caps = EndpointCapabilities::default();
        assert_eq!(caps.grade(), EndpointGrade::D);
        assert_eq!(caps.priority_adjustment(), 0); // unknown = no adjustment
    }

    #[test]
    fn test_grade_a() {
        let caps = EndpointCapabilities {
            supports_eth_get_logs: Some(true),
            max_batch_size: Some(100),
            max_block_range: Some(10_000),
            ..Default::default()
        };
        assert_eq!(caps.grade(), EndpointGrade::A);
        assert_eq!(caps.priority_adjustment(), -20);
    }

    #[test]
    fn test_grade_a_unlimited() {
        let caps = EndpointCapabilities {
            supports_eth_get_logs: Some(true),
            max_batch_size: Some(0), // unlimited
            max_block_range: Some(0), // unlimited
            ..Default::default()
        };
        assert_eq!(caps.grade(), EndpointGrade::A);
    }

    #[test]
    fn test_grade_b() {
        let caps = EndpointCapabilities {
            supports_eth_get_logs: Some(true),
            max_batch_size: Some(50),
            max_block_range: Some(5_000),
            ..Default::default()
        };
        assert_eq!(caps.grade(), EndpointGrade::B);
        assert_eq!(caps.priority_adjustment(), -10);
    }

    #[test]
    fn test_grade_c() {
        let caps = EndpointCapabilities {
            supports_eth_get_logs: Some(true),
            max_batch_size: Some(5),
            max_block_range: Some(100),
            ..Default::default()
        };
        assert_eq!(caps.grade(), EndpointGrade::C);
        assert_eq!(caps.priority_adjustment(), 0);
    }

    #[test]
    fn test_grade_d_no_logs() {
        let caps = EndpointCapabilities {
            supports_eth_get_logs: Some(false),
            max_batch_size: Some(100),
            max_block_range: Some(10_000),
            ..Default::default()
        };
        assert_eq!(caps.grade(), EndpointGrade::D);
        assert_eq!(caps.priority_adjustment(), 10);
    }

    #[test]
    fn test_grade_ordering() {
        assert!(EndpointGrade::F < EndpointGrade::D);
        assert!(EndpointGrade::D < EndpointGrade::C);
        assert!(EndpointGrade::C < EndpointGrade::B);
        assert!(EndpointGrade::B < EndpointGrade::A);
    }

    #[test]
    fn test_grade_display() {
        assert_eq!(format!("{}", EndpointGrade::A), "A");
        assert_eq!(format!("{}", EndpointGrade::F), "F");
    }

    #[test]
    fn test_with_capabilities_builder() {
        let caps = EndpointCapabilities {
            supports_eth_get_logs: Some(true),
            max_batch_size: Some(100),
            max_block_range: Some(10_000),
            ..Default::default()
        };
        let endpoint = RpcEndpoint::new("https://rpc.example.com")
            .with_capabilities(caps);
        assert_eq!(endpoint.capabilities.grade(), EndpointGrade::A);
    }

    #[test]
    fn test_with_ws_url_sets_websocket_capability() {
        let endpoint = RpcEndpoint::new("https://rpc.example.com")
            .with_ws_url("wss://rpc.example.com");
        assert!(endpoint.capabilities.supports_websocket);
    }

    #[test]
    fn test_backward_compat_deserialization() {
        // Old JSON without capabilities field should deserialize fine
        let json = r#"{"url":"https://rpc.example.com","name":"Test","priority":50,"chain_id":1}"#;
        let endpoint: RpcEndpoint = serde_json::from_str(json).unwrap();
        assert_eq!(endpoint.url, "https://rpc.example.com");
        assert_eq!(endpoint.capabilities.grade(), EndpointGrade::D);
        assert!(!endpoint.capabilities.supports_websocket);
    }

    #[test]
    fn test_capabilities_serialization_roundtrip() {
        let caps = EndpointCapabilities {
            supports_eth_get_logs: Some(true),
            max_batch_size: Some(100),
            max_block_range: Some(10_000),
            supports_debug_trace: Some(false),
            supports_websocket: true,
            rate_limit_rps: Some(25),
        };
        let endpoint = RpcEndpoint::new("https://rpc.example.com")
            .with_name("Test")
            .with_capabilities(caps);

        let json = serde_json::to_string(&endpoint).unwrap();
        let deserialized: RpcEndpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.capabilities.grade(), EndpointGrade::A);
        assert!(deserialized.capabilities.supports_websocket);
        assert_eq!(deserialized.capabilities.rate_limit_rps, Some(25));
    }

    #[test]
    fn test_exponential_backoff() {
        let endpoint = RpcEndpoint::new("https://rpc.example.com");
        let mut stats = EndpointStats::new(&endpoint);
        let base_delay = std::time::Duration::from_secs(5);

        // Initial delay should be base delay (5s)
        assert_eq!(stats.current_retry_delay(base_delay).as_secs(), 5);

        // After first recovery attempt: 5 * 2^1 = 10s
        stats.increment_recovery_attempts();
        assert_eq!(stats.current_retry_delay(base_delay).as_secs(), 10);

        // After second: 5 * 2^2 = 20s
        stats.increment_recovery_attempts();
        assert_eq!(stats.current_retry_delay(base_delay).as_secs(), 20);

        // After third: 5 * 2^3 = 40s
        stats.increment_recovery_attempts();
        assert_eq!(stats.current_retry_delay(base_delay).as_secs(), 40);

        // Should cap at 300s (5 minutes)
        for _ in 0..10 {
            stats.increment_recovery_attempts();
        }
        assert_eq!(stats.current_retry_delay(base_delay).as_secs(), 300);

        // Recovery resets backoff
        stats.mark_recovered();
        assert_eq!(stats.recovery_attempts, 0);
        assert_eq!(stats.current_retry_delay(base_delay).as_secs(), 5);
    }
}
