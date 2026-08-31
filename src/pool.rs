//! Core RPC pool implementation.

use crate::endpoint::{EndpointStats, RpcEndpoint};
use crate::error::RpcPoolError;
use crate::freshness::{best_known_healthy_block, freshness_distribution};
use crate::metrics::{EndpointMetrics, RpcPoolMetrics};
use crate::strategies::SelectionStrategy;

use alloy::providers::{Provider, RootProvider};
use alloy::transports::http::Http;
use parking_lot::RwLock;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, instrument, trace, warn};

/// Maximum length for error messages to prevent unbounded memory growth.
const MAX_ERROR_MESSAGE_LENGTH: usize = 512;

/// Default request timeout in seconds.
const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 30;

/// Default health check timeout in seconds.
const DEFAULT_HEALTH_CHECK_TIMEOUT_SECS: u64 = 10;

/// Summary of endpoint health status.
#[derive(Debug, Clone, Copy)]
pub struct HealthSummary {
    /// Number of healthy endpoints.
    pub healthy: usize,
    /// Number of unhealthy endpoints.
    pub unhealthy: usize,
    /// Total number of endpoints.
    pub total: usize,
}

impl HealthSummary {
    /// Returns true if all endpoints are unhealthy.
    pub fn all_unhealthy(&self) -> bool {
        self.healthy == 0 && self.total > 0
    }

    /// Returns the percentage of healthy endpoints.
    pub fn health_percentage(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.healthy as f64 / self.total as f64) * 100.0
    }
}

/// Configuration for the RPC pool.
#[derive(Clone)]
pub struct RpcPoolConfig {
    /// List of RPC endpoints (will be sorted by priority).
    pub endpoints: Vec<RpcEndpoint>,

    /// Strategy for selecting endpoints.
    pub strategy: Arc<RwLock<Box<dyn SelectionStrategy>>>,

    /// Interval between health checks.
    pub health_check_interval: Duration,

    /// Number of consecutive errors before marking an endpoint unhealthy.
    pub max_consecutive_errors: u32,

    /// Delay before retrying an unhealthy endpoint.
    pub retry_delay: Duration,

    /// Timeout for individual RPC requests.
    pub request_timeout: Duration,

    /// Timeout for health check probes.
    pub health_check_timeout: Duration,

    /// Maximum allowed lag behind the best-known endpoint head.
    pub max_block_lag: Option<u64>,
}

impl Default for RpcPoolConfig {
    fn default() -> Self {
        Self {
            endpoints: vec![],
            strategy: Arc::new(RwLock::new(Box::new(crate::strategies::FailoverStrategy))),
            health_check_interval: Duration::from_secs(60),
            max_consecutive_errors: 3,
            retry_delay: Duration::from_secs(5),
            request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
            health_check_timeout: Duration::from_secs(DEFAULT_HEALTH_CHECK_TIMEOUT_SECS),
            max_block_lag: None,
        }
    }
}

impl RpcPoolConfig {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: set endpoints.
    pub fn with_endpoints(mut self, endpoints: Vec<RpcEndpoint>) -> Self {
        self.endpoints = endpoints;
        self
    }

    /// Builder: set strategy.
    pub fn with_strategy(mut self, strategy: Box<dyn SelectionStrategy>) -> Self {
        self.strategy = Arc::new(RwLock::new(strategy));
        self
    }

    /// Builder: set health check interval.
    pub fn with_health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = interval;
        self
    }

    /// Builder: set max consecutive errors.
    pub fn with_max_consecutive_errors(mut self, max: u32) -> Self {
        self.max_consecutive_errors = max;
        self
    }

    /// Builder: set retry delay.
    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// Builder: set request timeout.
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Builder: set health check timeout.
    pub fn with_health_check_timeout(mut self, timeout: Duration) -> Self {
        self.health_check_timeout = timeout;
        self
    }

    /// Builder: set the maximum allowed block lag for endpoint selection.
    pub fn with_max_block_lag(mut self, max_block_lag: u64) -> Self {
        self.max_block_lag = Some(max_block_lag);
        self
    }
}

/// High-availability RPC connection pool with automatic failover.
pub struct RpcPool {
    /// Configured endpoints (sorted by priority).
    endpoints: Vec<RpcEndpoint>,

    /// Statistics for each endpoint.
    stats: RwLock<HashMap<String, EndpointStats>>,

    /// Selection strategy.
    strategy: Arc<RwLock<Box<dyn SelectionStrategy>>>,

    /// Configuration.
    max_consecutive_errors: u32,
    retry_delay: Duration,
    health_check_interval: Duration,
    request_timeout: Duration,
    health_check_timeout: Duration,
    max_block_lag: Option<u64>,

    /// Aggregated metrics.
    total_requests: AtomicU64,
    failovers: AtomicU64,

    /// Cancellation flag and notification for graceful shutdown.
    cancelled: AtomicBool,
    cancel_notify: tokio::sync::Notify,

    /// Handle to the health check task (if running).
    health_check_handle: RwLock<Option<AbortHandleWrapper>>,

    /// Shared HTTP client for health check probes (avoids creating a new
    /// reqwest::Client + TLS CA store per probe — prevents memory leak).
    probe_client: alloy::transports::http::reqwest::Client,
}

impl RpcPool {
    /// Create a new RPC pool with the given configuration.
    pub fn new(mut config: RpcPoolConfig) -> Result<Self, RpcPoolError> {
        if config.endpoints.is_empty() {
            error!("Attempted to create RPC pool with no endpoints configured");
            return Err(RpcPoolError::NoEndpointsConfigured);
        }

        // Deduplicate endpoints by URL, keeping the first (lowest priority value = highest priority)
        {
            let mut seen = HashSet::new();
            let before = config.endpoints.len();
            config.endpoints.retain(|e| seen.insert(e.url.clone()));
            let removed = before - config.endpoints.len();
            if removed > 0 {
                warn!(removed, "Duplicate endpoints removed from RPC pool");
            }
        }

        // Sort endpoints by priority (lower = higher priority)
        config.endpoints.sort_by_key(|e| e.priority);

        // Initialize stats for each endpoint
        let mut stats = HashMap::new();
        for endpoint in &config.endpoints {
            stats.insert(endpoint.url.clone(), EndpointStats::new(endpoint));
            trace!(
                endpoint_name = %endpoint.name,
                endpoint_url = %endpoint.url,
                priority = endpoint.priority,
                chain_id = endpoint.chain_id,
                "Registered endpoint"
            );
        }

        let strategy_name = config.strategy.read().name();
        info!(
            endpoints = config.endpoints.len(),
            strategy = strategy_name,
            request_timeout_ms = config.request_timeout.as_millis() as u64,
            health_check_timeout_ms = config.health_check_timeout.as_millis() as u64,
            health_check_interval_secs = config.health_check_interval.as_secs(),
            max_consecutive_errors = config.max_consecutive_errors,
            max_block_lag = ?config.max_block_lag,
            retry_delay_secs = config.retry_delay.as_secs(),
            "RPC pool initialized"
        );

        // Log endpoint summary at debug level
        debug!(
            endpoint_names = ?config.endpoints.iter().map(|e| &e.name).collect::<Vec<_>>(),
            "Configured endpoints (sorted by priority)"
        );

        Ok(Self {
            endpoints: config.endpoints,
            stats: RwLock::new(stats),
            strategy: config.strategy,
            max_consecutive_errors: config.max_consecutive_errors,
            retry_delay: config.retry_delay,
            health_check_interval: config.health_check_interval,
            request_timeout: config.request_timeout,
            health_check_timeout: config.health_check_timeout,
            max_block_lag: config.max_block_lag,
            total_requests: AtomicU64::new(0),
            failovers: AtomicU64::new(0),
            cancelled: AtomicBool::new(false),
            cancel_notify: tokio::sync::Notify::new(),
            health_check_handle: RwLock::new(None),
            probe_client: {
                crate::tls::ensure_provider();
                alloy::transports::http::reqwest::Client::builder()
                    .pool_max_idle_per_host(1)
                    .pool_idle_timeout(std::time::Duration::from_secs(10))
                    .build()
                    .expect("failed to build health-check HTTP client")
            },
        })
    }

    /// Get the configured request timeout.
    pub fn request_timeout(&self) -> Duration {
        self.request_timeout
    }

    /// Get the configured maximum block lag for endpoint selection.
    pub fn max_block_lag(&self) -> Option<u64> {
        self.max_block_lag
    }

    /// Check if the pool has been shut down.
    pub fn is_shutdown(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    /// Wait until cancellation is signalled.
    async fn cancelled(&self) {
        // Fast path: already cancelled
        if self.is_shutdown() {
            return;
        }
        // Wait for notification
        loop {
            let notified = self.cancel_notify.notified();
            if self.is_shutdown() {
                return;
            }
            notified.await;
            if self.is_shutdown() {
                return;
            }
        }
    }

    /// Get the URL of the currently selected endpoint.
    pub fn get_current_url(&self) -> Option<String> {
        let exclude = HashSet::new();
        self.select_endpoint(&exclude).map(|endpoint| endpoint.url)
    }

    /// Get all configured RPC URLs.
    pub fn get_all_urls(&self) -> Vec<String> {
        self.endpoints.iter().map(|e| e.url.clone()).collect()
    }

    /// Collect a snapshot of stats.
    #[inline]
    fn collect_stats_snapshot(&self) -> HashMap<String, EndpointStats> {
        self.stats.read().clone()
    }

    fn select_endpoint(&self, exclude: &HashSet<String>) -> Option<RpcEndpoint> {
        let stats_map = self.collect_stats_snapshot();
        let candidates = self.selection_candidates(&stats_map);
        let mut strategy = self.strategy.write();
        let selected = strategy
            .select(candidates.as_ref(), &stats_map, exclude)
            .cloned();
        let selected_is_healthy = selected
            .as_ref()
            .and_then(|endpoint| stats_map.get(&endpoint.url))
            .map(|stats| stats.is_healthy)
            .unwrap_or(true);
        if (selected.is_some() && selected_is_healthy)
            || candidates.len() == self.endpoints.len()
        {
            return selected;
        }

        warn!(
            fresh_candidates = candidates.len(),
            total_endpoints = self.endpoints.len(),
            "Fresh RPC endpoints exhausted; falling back to legacy endpoint selection"
        );
        strategy.select(&self.endpoints, &stats_map, exclude).cloned()
    }

    fn selection_candidates<'a>(
        &'a self,
        stats: &HashMap<String, EndpointStats>,
    ) -> Cow<'a, [RpcEndpoint]> {
        let Some(max_block_lag) = self.max_block_lag else {
            return Cow::Borrowed(&self.endpoints);
        };

        // An unhealthy endpoint's last observation may be stale or malformed;
        // it must not define the reference that filters healthy endpoints.
        let Some(best_known_block) = best_known_healthy_block(stats) else {
            warn!(
                max_block_lag,
                endpoint_count = self.endpoints.len(),
                "No RPC endpoint head is known; using legacy endpoint selection"
            );
            return Cow::Borrowed(&self.endpoints);
        };

        let mut candidates = Vec::with_capacity(self.endpoints.len());
        let mut lagging = 0usize;
        let mut unknown = 0usize;
        for endpoint in &self.endpoints {
            match stats
                .get(&endpoint.url)
                .and_then(|s| s.latest_block_number)
            {
                Some(block) if best_known_block.saturating_sub(block) <= max_block_lag => {
                    candidates.push(endpoint.clone());
                }
                Some(_) => lagging += 1,
                None => unknown += 1,
            }
        }

        if candidates.is_empty() {
            warn!(
                max_block_lag,
                best_known_block,
                lagging,
                unknown,
                "No RPC endpoint passed head freshness; using legacy endpoint selection"
            );
            return Cow::Borrowed(&self.endpoints);
        }

        Cow::Owned(candidates)
    }

    /// Execute a function with automatic failover across endpoints.
    ///
    /// The provided function receives the endpoint URL and should create
    /// and use its own provider instance.
    #[instrument(skip(self, f), level = "trace", fields(request_id = %self.total_requests.load(Ordering::Relaxed) + 1))]
    pub async fn execute_with_url<F, Fut, T, E>(&self, f: F) -> Result<T, RpcPoolError>
    where
        F: Fn(String) -> Fut + Clone,
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        if self.is_shutdown() {
            debug!("Request rejected: pool is shut down");
            return Err(RpcPoolError::PoolShutdown);
        }

        let request_id = self.total_requests.fetch_add(1, Ordering::Relaxed) + 1;
        trace!(request_id, "Starting request execution");

        let mut tried = HashSet::new();
        let mut last_error = None;
        let mut attempt = 0u32;

        for _ in 0..self.endpoints.len() {
            attempt += 1;

            // Check for shutdown
            if self.is_shutdown() {
                debug!(
                    request_id,
                    attempt, "Request cancelled: pool shutdown in progress"
                );
                return Err(RpcPoolError::PoolShutdown);
            }

            // Select endpoint
            let endpoint = self.select_endpoint(&tried);

            let endpoint = match endpoint {
                Some(e) => e,
                None => {
                    debug!(
                        request_id,
                        attempt,
                        tried_count = tried.len(),
                        "No more endpoints available to try"
                    );
                    break;
                }
            };

            tried.insert(endpoint.url.clone());

            trace!(
                request_id,
                attempt,
                endpoint_name = %endpoint.name,
                endpoint_url = %endpoint.url,
                "Selected endpoint for request"
            );

            // Execute request with timeout
            let start = Instant::now();
            let request_future = f(endpoint.url.clone());

            let result = tokio::select! {
                biased;

                _ = self.cancelled() => {
                    return Err(RpcPoolError::PoolShutdown);
                }

                result = tokio::time::timeout(self.request_timeout, request_future) => {
                    result
                }
            };

            match result {
                Ok(Ok(value)) => {
                    let latency = start.elapsed().as_millis() as u64;
                    if let Some(stats) = self.stats.write().get_mut(&endpoint.url) {
                        stats.record_success(latency);
                    }
                    trace!(
                        request_id,
                        endpoint_name = %endpoint.name,
                        latency_ms = latency,
                        "Request completed successfully"
                    );
                    return Ok(value);
                }
                Ok(Err(e)) => {
                    let raw_error = e.to_string();
                    let error_msg = truncate_error_message(&raw_error);

                    // The node answered; the CALL failed. Failing over would replay
                    // the same deterministic error on every remaining endpoint while
                    // marking each one unhealthy, which is how a single reverting
                    // call takes down a whole tier.
                    if is_call_failure(&raw_error) {
                        trace!(
                            request_id,
                            endpoint_name = %endpoint.name,
                            error = %error_msg,
                            "Call failed on a healthy endpoint; not retrying"
                        );
                        return Err(RpcPoolError::CallFailed(error_msg));
                    }

                    if let Some(stats) = self.stats.write().get_mut(&endpoint.url) {
                        let marked_unhealthy =
                            stats.record_failure(error_msg.clone(), self.max_consecutive_errors);
                        if marked_unhealthy {
                            warn!(
                                endpoint = %endpoint.name,
                                consecutive_errors = stats.consecutive_errors,
                                "Endpoint marked unhealthy"
                            );
                        }
                    }

                    self.failovers.fetch_add(1, Ordering::Relaxed);
                    last_error = Some(error_msg);

                    debug!(
                        endpoint = %endpoint.name,
                        error = %e,
                        "Request failed, trying next endpoint"
                    );
                }
                Err(_timeout) => {
                    let error_msg = format!(
                        "Request timeout after {}ms",
                        self.request_timeout.as_millis()
                    );
                    if let Some(stats) = self.stats.write().get_mut(&endpoint.url) {
                        let marked_unhealthy =
                            stats.record_failure(error_msg.clone(), self.max_consecutive_errors);
                        if marked_unhealthy {
                            warn!(
                                endpoint = %endpoint.name,
                                "Endpoint marked unhealthy due to timeout"
                            );
                        }
                    }

                    self.failovers.fetch_add(1, Ordering::Relaxed);
                    last_error = Some(error_msg);

                    debug!(
                        endpoint = %endpoint.name,
                        timeout_ms = self.request_timeout.as_millis() as u64,
                        "Request timed out, trying next endpoint"
                    );
                }
            }
        }

        let error_msg = last_error.unwrap_or_else(|| "Unknown error".to_string());
        let health = self.health_summary();
        error!(
            request_id,
            tried_endpoints = tried.len(),
            healthy_endpoints = health.healthy,
            unhealthy_endpoints = health.unhealthy,
            total_endpoints = health.total,
            last_error = %error_msg,
            "All endpoints failed (most endpoints marked unhealthy from previous failures)"
        );
        Err(RpcPoolError::AllEndpointsFailed(error_msg))
    }

    /// Execute a request with automatic failover using a pre-built provider.
    ///
    /// Creates a new provider for each attempt (recommended for most use cases).
    pub async fn execute<T, E, F, Fut>(&self, f: F) -> Result<T, RpcPoolError>
    where
        F: Fn(url::Url) -> Fut + Clone,
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error,
    {
        self.execute_with_url(|url_str| {
            let f = f.clone();
            async move {
                let url: url::Url = url_str.parse().map_err(|e: url::ParseError| {
                    std::io::Error::other(format!("Invalid URL: {}", e))
                })?;
                f(url)
                    .await
                    .map_err(|e| std::io::Error::other(e.to_string()))
            }
        })
        .await
    }

    /// Start background health check task.
    ///
    /// Returns a handle that can be used to abort the task.
    /// The task will automatically stop when `shutdown()` is called.
    pub fn start_health_check(self: &Arc<Self>) -> tokio::task::JoinHandle<()> {
        let pool = Arc::clone(self);
        let interval = self.health_check_interval;

        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);

            loop {
                tokio::select! {
                    biased;

                    _ = pool.cancelled() => {
                        info!("Health check task shutting down");
                        break;
                    }

                    _ = ticker.tick() => {
                        pool.check_health().await;
                    }
                }
            }
        });

        // Store handle for cleanup
        *self.health_check_handle.write() = Some(handle.abort_handle().into());

        handle
    }

    /// Perform health check on all endpoints.
    ///
    /// Probes **every** endpoint each cycle, in two roles:
    /// - **Detection** (currently-healthy endpoints): a live probe runs every
    ///   cycle. Repeated probe failures call `record_failure`, marking the
    ///   endpoint unhealthy once `max_consecutive_errors` is reached so that
    ///   `get_current_url()` fails over. This is essential for callers that use
    ///   the pool only as a URL selector (their own provider sends the real
    ///   traffic, so request failures never flow back into the pool).
    /// - **Recovery** (currently-unhealthy endpoints): probed subject to the
    ///   existing exponential backoff; a success marks the endpoint recovered.
    async fn check_health(&self) {
        trace!("Starting health check cycle");
        let mut checked_count = 0u32;
        let mut recovered_count = 0u32;
        let mut downed_count = 0u32;

        for endpoint in &self.endpoints {
            // Check for shutdown
            if self.is_shutdown() {
                debug!("Health check interrupted by shutdown");
                return;
            }

            // Decide whether to probe, and remember pre-probe health so the
            // result routes to detection (healthy) or recovery (unhealthy).
            // Healthy endpoints are probed every cycle; unhealthy ones honor
            // the recovery backoff window.
            let was_healthy = {
                let stats = self.stats.read();
                match stats.get(&endpoint.url) {
                    Some(s) if s.is_healthy => Some(true),
                    Some(s) if s.can_retry(self.retry_delay) => Some(false),
                    Some(_) => None, // unhealthy, still within backoff — skip
                    None => Some(true),
                }
            };
            let Some(was_healthy) = was_healthy else {
                continue;
            };

            checked_count += 1;

            // Probe with a simple block-number request (with timeout).
            let url: Result<url::Url, _> = endpoint.url.parse();
            if let Ok(url) = url {
                let transport = Http::with_client(self.probe_client.clone(), url);
                let provider = RootProvider::<alloy::network::Ethereum>::new(
                    alloy::rpc::client::RpcClient::new(transport, true),
                );

                let probe_start = Instant::now();
                let probe_result = tokio::select! {
                    biased;

                    _ = self.cancelled() => {
                        return;
                    }

                    result = tokio::time::timeout(
                        self.health_check_timeout,
                        provider.get_block_number()
                    ) => {
                        result
                    }
                };

                // Normalize so detection/recovery share one match.
                let outcome: Result<u64, String> = match probe_result {
                    Ok(Ok(block_number)) => Ok(block_number),
                    Ok(Err(e)) => Err(e.to_string()),
                    Err(_) => Err(format!(
                        "health probe timed out after {}ms",
                        self.health_check_timeout.as_millis()
                    )),
                };

                match outcome {
                    Ok(block_number) => {
                        if let Some(stats) = self.stats.write().get_mut(&endpoint.url) {
                            stats.latest_block_number = Some(block_number);
                            let latency_ms = probe_start.elapsed().as_millis() as u64;
                            if was_healthy {
                                // Liveness confirmed — record latency, reset any
                                // partial failure streak.
                                stats.record_success(latency_ms);
                            } else {
                                stats.mark_recovered();
                                recovered_count += 1;
                                info!(endpoint = %endpoint.name, "Endpoint recovered");
                            }
                        }
                    }
                    Err(error) => {
                        if let Some(stats) = self.stats.write().get_mut(&endpoint.url) {
                            stats.latest_block_number = None;
                            if was_healthy {
                                // Detection: count the failure; mark unhealthy at
                                // the threshold so get_current_url() fails over.
                                let now_down = stats
                                    .record_failure(error.clone(), self.max_consecutive_errors);
                                if now_down {
                                    downed_count += 1;
                                    warn!(
                                        endpoint = %endpoint.name,
                                        %error,
                                        consecutive_errors = stats.consecutive_errors,
                                        "Endpoint detected down — marked unhealthy, will fail over"
                                    );
                                } else {
                                    warn!(
                                        endpoint = %endpoint.name,
                                        %error,
                                        consecutive_errors = stats.consecutive_errors,
                                        max = self.max_consecutive_errors,
                                        "Endpoint probe failed"
                                    );
                                }
                            } else {
                                // Recovery backoff (existing behavior).
                                stats.last_error_time = Some(Instant::now());
                                stats.increment_recovery_attempts();
                                let next_retry = stats.current_retry_delay(self.retry_delay);
                                trace!(
                                    endpoint_name = %endpoint.name,
                                    %error,
                                    recovery_attempts = stats.recovery_attempts,
                                    next_retry_secs = next_retry.as_secs(),
                                    "Endpoint health check failed, increasing backoff"
                                );
                            }
                        }
                    }
                }
            } else if let Some(stats) = self.stats.write().get_mut(&endpoint.url) {
                stats.latest_block_number = None;
            }
        }

        self.log_freshness_distribution();

        if checked_count > 0 {
            debug!(
                checked = checked_count,
                recovered = recovered_count,
                downed = downed_count,
                "Health check cycle completed"
            );
        }
    }

    fn log_freshness_distribution(&self) {
        let Some(max_block_lag) = self.max_block_lag else {
            return;
        };
        let stats = self.stats.read();
        let distribution = freshness_distribution(&stats, max_block_lag);
        info!(
            max_block_lag,
            best_known_block = ?distribution.best_known_block,
            fresh = distribution.fresh,
            lagging = distribution.lagging,
            unknown = distribution.unknown,
            unhealthy = distribution.unhealthy,
            endpoint_count = stats.len(),
            "RPC endpoint head freshness distribution"
        );
    }

    /// Gracefully shutdown the pool.
    ///
    /// This cancels the health check task and prevents new requests.
    /// Existing in-flight requests will complete or be cancelled.
    pub async fn shutdown(&self) {
        info!("Initiating RPC pool shutdown");

        // Signal cancellation
        self.cancelled.store(true, Ordering::Release);
        self.cancel_notify.notify_waiters();

        // Wait for health check task to finish
        let handle = self.health_check_handle.write().take();
        if let Some(handle) = handle {
            // Give it a moment to finish gracefully
            let _ = tokio::time::timeout(Duration::from_secs(5), async {
                loop {
                    if handle.is_finished() {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            })
            .await;
        }

        info!("RPC pool shutdown complete");
    }

    /// Get a summary of endpoint health status.
    ///
    /// Returns counts of healthy, unhealthy, and total endpoints.
    pub fn health_summary(&self) -> HealthSummary {
        let mut healthy = 0;
        let mut unhealthy = 0;

        for stats in self.stats.read().values() {
            if stats.is_healthy {
                healthy += 1;
            } else {
                unhealthy += 1;
            }
        }

        HealthSummary {
            healthy,
            unhealthy,
            total: self.endpoints.len(),
        }
    }

    /// Manually mark an endpoint as unhealthy.
    pub fn mark_unhealthy(&self, url: &str) {
        if let Some(stats) = self.stats.write().get_mut(url) {
            stats.is_healthy = false;
            stats.last_error_time = Some(Instant::now());
            debug!(
                endpoint_name = %stats.name,
                endpoint_url = %url,
                "Endpoint manually marked unhealthy"
            );
        } else {
            warn!(endpoint_url = %url, "Attempted to mark unknown endpoint as unhealthy");
        }
    }

    /// Manually mark an endpoint as healthy.
    pub fn mark_healthy(&self, url: &str) {
        if let Some(stats) = self.stats.write().get_mut(url) {
            stats.mark_recovered();
            debug!(
                endpoint_name = %stats.name,
                endpoint_url = %url,
                "Endpoint manually marked healthy"
            );
        } else {
            warn!(endpoint_url = %url, "Attempted to mark unknown endpoint as healthy");
        }
    }

    /// Get current metrics.
    pub fn metrics(&self) -> RpcPoolMetrics {
        let endpoints: Vec<EndpointMetrics> = self
            .stats
            .read()
            .values()
            .map(EndpointMetrics::from)
            .collect();

        let current_endpoint = self
            .select_endpoint(&HashSet::new())
            .map(|endpoint| endpoint.name)
            .unwrap_or_else(|| "none".to_string());

        RpcPoolMetrics {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            failovers: self.failovers.load(Ordering::Relaxed),
            current_endpoint,
            endpoints,
        }
    }
}

impl Drop for RpcPool {
    fn drop(&mut self) {
        // Signal shutdown to any running tasks
        self.cancelled.store(true, Ordering::Release);
        self.cancel_notify.notify_waiters();

        // Abort health check task if still running
        if let Some(handle) = self.health_check_handle.get_mut().take() {
            if !handle.is_finished() {
                handle.abort();
            }
        }

        debug!("RpcPool dropped, resources cleaned up");
    }
}

/// Wrapper to store abort handle with is_finished check.
struct AbortHandleWrapper {
    handle: tokio::task::AbortHandle,
}

impl AbortHandleWrapper {
    fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    fn abort(&self) {
        self.handle.abort();
    }
}

impl From<tokio::task::AbortHandle> for AbortHandleWrapper {
    fn from(handle: tokio::task::AbortHandle) -> Self {
        Self { handle }
    }
}

/// Does this error describe a failed CALL rather than a failed ENDPOINT?
///
/// An EVM revert is an outcome the node computed and reported: it received the
/// request, ran it, and answered. Counting that against endpoint health marks
/// healthy endpoints unhealthy, and on a small tier it is fatal — the Base standard
/// tier has two endpoints, so ~6 reverting requests take the tier down and
/// subsequent legitimate reads fail with "All endpoints failed".
///
/// Two markers are required, and the conjunction is the point:
///
/// - **a JSON-RPC error *response*** (`error code 3`, EIP-1474's execution error) —
///   evidence that a node answered, rather than text we found in a proxy's HTML,
///   an HTTP body echoed by a transport error, or an error the caller's own closure
///   constructed;
/// - **the revert phrase itself**, matched case-insensitively.
///
/// The asymmetry drives the narrowness. A false negative costs nothing new: the
/// error keeps today's failover and health accounting. A false positive is a real
/// availability regression — a genuinely broken endpoint would escape both health
/// accounting *and* failover, so the request fails without ever trying a working
/// endpoint. When in doubt, treat it as an endpoint failure.
///
/// Consequently this does NOT match reverts a vendor reports under a different code
/// (some use `-32000`), nor `invalid opcode` / out-of-gas / `VM Exception` phrasings.
/// Those keep the old behaviour, which is safe, just not optimal.
///
/// One caveat on "deterministic": a revert is deterministic *at a given block*.
/// Endpoints sitting at different `latest` heads can genuinely disagree, so not
/// retrying may surface a revert that a further-ahead endpoint would not have
/// produced. That is accepted — the alternative poisons the pool on every revert.
///
/// # Known limitation
///
/// The marker is searched for anywhere in the message, not anchored at its start,
/// and that is deliberate. Consumers wrap: this crate's own `execute` re-wraps into
/// `io::Error`, and callers wrap again into their own error types, so an anchored
/// match would reject the very production errors this exists to classify.
///
/// The cost is that a message which merely *quotes* the marker is indistinguishable
/// from one that wraps it — `Display` is all we have for a generic `E`, and the two
/// are byte-identical. In practice the quoting case is close to unconstructible: the
/// marker is Alloy's Rust-side rendering, not anything a node or proxy emits, so
/// producing it means Rust code formatted a genuine revert error — which is a
/// genuine revert. Closing the gap properly needs a typed error or provenance from
/// the transport, not a better pattern; that is an API change, deliberately not made
/// here.
#[inline]
pub(crate) fn is_call_failure(msg: &str) -> bool {
    // Alloy's rendering of a JSON-RPC error response, carrying the code as an exact
    // token. Matching the whole marker rather than "error code 3" is what makes the
    // code a token instead of a prefix — otherwise `error code 30` also qualifies.
    const REVERT_MARKER: &str = "server returned an error response: error code 3: ";

    let Some(offset) = msg.find(REVERT_MARKER) else {
        return false;
    };
    // The phrase must be what the node reported for THIS error, i.e. immediately
    // after the marker — not merely present somewhere later in a response body that
    // the error happens to quote.
    msg[offset + REVERT_MARKER.len()..]
        .to_ascii_lowercase()
        .starts_with("execution reverted")
}

/// Truncate error message to prevent unbounded memory growth.
#[inline]
fn truncate_error_message(msg: &str) -> String {
    if msg.len() <= MAX_ERROR_MESSAGE_LENGTH {
        msg.to_string()
    } else {
        format!("{}...(truncated)", &msg[..MAX_ERROR_MESSAGE_LENGTH])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategies::FailoverStrategy;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn create_test_config() -> RpcPoolConfig {
        RpcPoolConfig::new()
            .with_endpoints(vec![
                RpcEndpoint::new("https://rpc1.example.com"),
                RpcEndpoint::new("https://rpc2.example.com"),
            ])
            .with_strategy(Box::new(FailoverStrategy))
    }

    #[test]
    fn test_pool_creation() {
        let config = create_test_config();
        let pool = RpcPool::new(config);
        assert!(pool.is_ok());
    }

    #[test]
    fn test_empty_endpoints() {
        let config = RpcPoolConfig::new().with_strategy(Box::new(FailoverStrategy));

        let pool = RpcPool::new(config);
        assert!(matches!(pool, Err(RpcPoolError::NoEndpointsConfigured)));
    }

    #[test]
    fn test_get_urls() {
        let config = RpcPoolConfig::new()
            .with_endpoints(vec![
                RpcEndpoint::new("https://rpc1.example.com").with_priority(10),
                RpcEndpoint::new("https://rpc2.example.com").with_priority(50),
            ])
            .with_strategy(Box::new(FailoverStrategy));

        let pool = RpcPool::new(config).unwrap();

        // Should return highest priority (lowest number) endpoint
        assert_eq!(
            pool.get_current_url(),
            Some("https://rpc1.example.com".to_string())
        );

        // Should return all URLs
        let all = pool.get_all_urls();
        assert_eq!(all.len(), 2);
    }

    /// The real Base standard-tier payload: a JSON-RPC error response reporting an
    /// EVM revert.
    const REVERT_RESPONSE: &str = "server returned an error response: error code 3: \
         execution reverted: Unexpected error, data: \"0x08c379a0\"";

    #[test]
    fn classifies_json_rpc_reverts_as_call_failures() {
        assert!(is_call_failure(REVERT_RESPONSE));
        // Vendors differ on capitalisation of the phrase.
        assert!(is_call_failure(
            "server returned an error response: error code 3: Execution reverted"
        ));
    }

    #[test]
    fn endpoint_failures_keep_their_failover() {
        for endpoint_error in [
            "error sending request for url (https://rpc.example.com/base)",
            "server returned an error response: error code -32001: rate limited",
            "server returned an error response: error code -32000: header not found",
            "server returned an error response: error code -32601: method not found",
            "Request timeout after 2000ms",
        ] {
            assert!(
                !is_call_failure(endpoint_error),
                "must stay an endpoint failure: {endpoint_error}"
            );
        }
    }

    /// The dangerous direction. A false positive suppresses BOTH health accounting
    /// and failover, so a genuinely broken endpoint would keep being selected and the
    /// request would fail without ever trying a working one. Every case here carries
    /// revert-looking text — several carry BOTH markers — and none may qualify.
    #[test]
    fn revert_looking_text_that_is_not_this_error_is_not_a_call_failure() {
        for impostor in [
            // Transport failure whose Display quotes an upstream body.
            "error sending request for url (https://proxy.example.com): \
             body: {\"error\":\"execution reverted\"}",
            // Both markers present, but the revert phrase is not what this code
            // reported — it trails a body the transport error quoted.
            "error sending request for url (https://proxy.example.com): body: \
             error code 3, execution reverted",
            // Code boundary: 30 and 300 are not 3.
            "server returned an error response: error code 30: execution reverted",
            "server returned an error response: error code 300: execution reverted",
            // Rate limit that merely mentions the phrase.
            "server returned an error response: error code -32001: upstream said \
             execution reverted",
            // Error built by the caller's own closure.
            "failed to decode quote: execution reverted",
        ] {
            assert!(
                !is_call_failure(impostor),
                "must not suppress endpoint health: {impostor}"
            );
        }
    }

    /// A consumer that wraps the alloy error in its own type keeps the rendering
    /// intact, and that IS a genuine revert — classification must survive nesting.
    #[test]
    fn wrapped_but_genuine_revert_is_still_a_call_failure() {
        assert!(is_call_failure(&format!("RPC error: {REVERT_RESPONSE}")));
    }

    /// A reverting call must leave endpoint health completely untouched. Before this
    /// distinction existed, ~6 reverting requests marked every endpoint in a
    /// two-endpoint tier unhealthy and blinded the bot.
    #[tokio::test]
    async fn reverting_call_does_not_mark_endpoints_unhealthy() {
        let pool = RpcPool::new(create_test_config()).unwrap();
        let attempts = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        for _ in 0..10 {
            let counter = attempts.clone();
            let result: Result<(), RpcPoolError> = pool
                .execute_with_url(move |_url| {
                    let counter = counter.clone();
                    async move {
                        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        Err::<(), _>(std::io::Error::other(REVERT_RESPONSE))
                    }
                })
                .await;
            assert!(matches!(result, Err(RpcPoolError::CallFailed(_))));
        }

        assert_eq!(
            pool.health_summary().healthy,
            2,
            "a reverting call must not consume endpoint health"
        );
        assert_eq!(
            attempts.load(std::sync::atomic::Ordering::Relaxed),
            10,
            "each call must run once, not once per endpoint"
        );
        assert_eq!(
            pool.metrics().failovers,
            0,
            "a reverting call is not a failover"
        );
    }

    /// The contrast case: a genuine transport failure still marks endpoints unhealthy
    /// and still fails over across all of them.
    #[tokio::test]
    async fn transport_failure_still_degrades_endpoint_health() {
        let pool = RpcPool::new(create_test_config()).unwrap();

        for _ in 0..10 {
            let result: Result<(), RpcPoolError> = pool
                .execute_with_url(|_url| async {
                    Err::<(), _>(std::io::Error::other("error sending request for url"))
                })
                .await;
            assert!(matches!(result, Err(RpcPoolError::AllEndpointsFailed(_))));
        }

        assert_eq!(
            pool.health_summary().healthy,
            0,
            "a transport failure must still degrade endpoint health"
        );
    }

    #[test]
    fn test_truncate_error_message() {
        let short_msg = "Short error";
        assert_eq!(truncate_error_message(short_msg), short_msg);

        let long_msg = "x".repeat(1000);
        let truncated = truncate_error_message(&long_msg);
        assert!(truncated.len() < long_msg.len());
        assert!(truncated.ends_with("...(truncated)"));
    }

    #[test]
    fn test_config_builder() {
        let config = RpcPoolConfig::new()
            .with_request_timeout(Duration::from_secs(10))
            .with_health_check_timeout(Duration::from_secs(5))
            .with_health_check_interval(Duration::from_secs(30))
            .with_max_consecutive_errors(5)
            .with_retry_delay(Duration::from_secs(10));

        assert_eq!(config.request_timeout, Duration::from_secs(10));
        assert_eq!(config.health_check_timeout, Duration::from_secs(5));
        assert_eq!(config.health_check_interval, Duration::from_secs(30));
        assert_eq!(config.max_consecutive_errors, 5);
        assert_eq!(config.retry_delay, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn health_probe_retains_block_number() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": "0x75"
            })))
            .mount(&server)
            .await;
        let pool = RpcPool::new(
            RpcPoolConfig::new()
                .with_endpoints(vec![RpcEndpoint::new(server.uri())])
                .with_health_check_timeout(Duration::from_secs(1)),
        )
        .unwrap();

        pool.check_health().await;

        assert_eq!(
            pool.stats
                .read()
                .get(&server.uri())
                .unwrap()
                .latest_block_number,
            Some(117)
        );
    }

    #[tokio::test]
    async fn failed_health_probe_clears_previous_block_number() {
        let server = MockServer::start().await;
        let success = ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x75"
        }));
        Mock::given(method("POST"))
            .respond_with(success)
            .mount(&server)
            .await;
        let pool = RpcPool::new(
            RpcPoolConfig::new()
                .with_endpoints(vec![RpcEndpoint::new(server.uri())])
                .with_max_consecutive_errors(3)
                .with_health_check_timeout(Duration::from_secs(1)),
        )
        .unwrap();
        pool.check_health().await;
        assert_eq!(pool.stats.read().get(&server.uri()).unwrap().latest_block_number, Some(117));

        server.reset().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;
        pool.check_health().await;

        let stats = pool.stats.read();
        let endpoint = stats.get(&server.uri()).unwrap();
        assert_eq!(endpoint.latest_block_number, None);
        assert!(endpoint.is_healthy);
        assert_eq!(endpoint.consecutive_errors, 1);
    }

    #[test]
    fn configured_freshness_skips_lagging_endpoint() {
        let config = RpcPoolConfig::new()
            .with_endpoints(vec![
                RpcEndpoint::new("https://lagging.rpc").with_priority(1),
                RpcEndpoint::new("https://current.rpc").with_priority(2),
            ])
            .with_strategy(Box::new(FailoverStrategy))
            .with_max_block_lag(2);
        let pool = RpcPool::new(config).unwrap();

        let mut stats = pool.stats.write();
        stats
            .get_mut("https://lagging.rpc")
            .unwrap()
            .latest_block_number = Some(100);
        stats
            .get_mut("https://current.rpc")
            .unwrap()
            .latest_block_number = Some(117);
        drop(stats);

        assert_eq!(pool.get_current_url().as_deref(), Some("https://current.rpc"));
        let stats = pool.stats.read();
        assert!(stats.get("https://lagging.rpc").unwrap().is_healthy);
        assert_eq!(stats.get("https://lagging.rpc").unwrap().consecutive_errors, 0);
    }

    #[test]
    fn configured_freshness_skips_unknown_endpoint() {
        let config = RpcPoolConfig::new()
            .with_endpoints(vec![
                RpcEndpoint::new("https://unknown.rpc").with_priority(1),
                RpcEndpoint::new("https://probed.rpc").with_priority(2),
            ])
            .with_strategy(Box::new(FailoverStrategy))
            .with_max_block_lag(0);
        let pool = RpcPool::new(config).unwrap();

        pool.stats
            .write()
            .get_mut("https://probed.rpc")
            .unwrap()
            .latest_block_number = Some(117);

        assert_eq!(pool.get_current_url().as_deref(), Some("https://probed.rpc"));
    }

    #[tokio::test]
    async fn fresh_failure_falls_back_to_lagging_endpoint() {
        let config = RpcPoolConfig::new()
            .with_endpoints(vec![
                RpcEndpoint::new("https://fresh.rpc").with_priority(1),
                RpcEndpoint::new("https://lagging.rpc").with_priority(2),
            ])
            .with_strategy(Box::new(FailoverStrategy))
            .with_max_block_lag(2);
        let pool = RpcPool::new(config).unwrap();
        let mut stats = pool.stats.write();
        stats
            .get_mut("https://fresh.rpc")
            .unwrap()
            .latest_block_number = Some(117);
        stats
            .get_mut("https://lagging.rpc")
            .unwrap()
            .latest_block_number = Some(100);
        drop(stats);

        let result = pool
            .execute_with_url(|url| async move {
                if url == "https://fresh.rpc" {
                    Err::<String, _>(std::io::Error::other("fresh endpoint failed"))
                } else {
                    Ok::<String, std::io::Error>(url)
                }
            })
            .await;

        assert!(result.is_ok(), "request should succeed through the lagging endpoint");
        assert_eq!(result.ok().as_deref(), Some("https://lagging.rpc"));
    }

    #[tokio::test]
    async fn exhausted_fresh_health_prefers_healthy_lagging_fallback() {
        let config = RpcPoolConfig::new()
            .with_endpoints(vec![
                RpcEndpoint::new("https://fresh.rpc").with_priority(1),
                RpcEndpoint::new("https://fresh-unhealthy.rpc").with_priority(2),
                RpcEndpoint::new("https://lagging.rpc").with_priority(3),
            ])
            .with_strategy(Box::new(FailoverStrategy))
            .with_max_block_lag(2);
        let pool = RpcPool::new(config).unwrap();
        let mut stats = pool.stats.write();
        stats.get_mut("https://fresh.rpc").unwrap().latest_block_number = Some(117);
        let unhealthy = stats.get_mut("https://fresh-unhealthy.rpc").unwrap();
        unhealthy.latest_block_number = Some(117);
        unhealthy.is_healthy = false;
        stats.get_mut("https://lagging.rpc").unwrap().latest_block_number = Some(100);
        drop(stats);

        let result = pool
            .execute_with_url(|url| async move {
                if url == "https://fresh.rpc" {
                    Err::<String, _>(std::io::Error::other("fresh endpoint failed"))
                } else {
                    Ok::<String, std::io::Error>(url)
                }
            })
            .await;

        assert!(result.is_ok(), "request should use the healthy lagging fallback");
        assert_eq!(result.ok().as_deref(), Some("https://lagging.rpc"));
    }

    #[test]
    fn unhealthy_high_head_does_not_poison_freshness_reference() {
        let config = RpcPoolConfig::new()
            .with_endpoints(vec![
                RpcEndpoint::new("https://poisoned.rpc").with_priority(1),
                RpcEndpoint::new("https://current.rpc").with_priority(2),
                RpcEndpoint::new("https://lagging.rpc").with_priority(3),
            ])
            .with_strategy(Box::new(FailoverStrategy))
            .with_max_block_lag(0);
        let pool = RpcPool::new(config).unwrap();
        let mut stats = pool.stats.write();
        let poisoned = stats.get_mut("https://poisoned.rpc").unwrap();
        poisoned.is_healthy = false;
        poisoned.latest_block_number = Some(1_000);
        stats
            .get_mut("https://current.rpc")
            .unwrap()
            .latest_block_number = Some(100);
        stats
            .get_mut("https://lagging.rpc")
            .unwrap()
            .latest_block_number = Some(99);
        drop(stats);

        assert_eq!(pool.get_current_url().as_deref(), Some("https://current.rpc"));
    }

    #[test]
    fn unset_freshness_margin_preserves_legacy_selection() {
        let config = RpcPoolConfig::new()
            .with_endpoints(vec![
                RpcEndpoint::new("https://fast.rpc").with_priority(1),
                RpcEndpoint::new("https://current.rpc").with_priority(2),
            ])
            .with_strategy(Box::new(FailoverStrategy));
        let pool = RpcPool::new(config).unwrap();

        let mut stats = pool.stats.write();
        stats
            .get_mut("https://fast.rpc")
            .unwrap()
            .latest_block_number = Some(100);
        stats
            .get_mut("https://current.rpc")
            .unwrap()
            .latest_block_number = Some(117);
        drop(stats);

        assert_eq!(pool.get_current_url().as_deref(), Some("https://fast.rpc"));
    }

    #[test]
    fn freshness_falls_back_when_no_endpoint_has_a_head() {
        let config = RpcPoolConfig::new()
            .with_endpoints(vec![
                RpcEndpoint::new("https://unknown.rpc").with_priority(1),
                RpcEndpoint::new("https://also-unknown.rpc").with_priority(2),
            ])
            .with_strategy(Box::new(FailoverStrategy))
            .with_max_block_lag(0);
        let pool = RpcPool::new(config).unwrap();

        assert_eq!(pool.get_current_url().as_deref(), Some("https://unknown.rpc"));
    }

    #[test]
    fn test_pool_drop_sets_shutdown() {
        let config = create_test_config();
        let pool = RpcPool::new(config).unwrap();

        assert!(!pool.is_shutdown());
    }

    #[tokio::test]
    async fn test_shutdown() {
        let config = create_test_config();
        let pool = Arc::new(RpcPool::new(config).unwrap());

        // Start health check
        let _handle = pool.start_health_check();

        assert!(!pool.is_shutdown());

        // Shutdown
        pool.shutdown().await;

        assert!(pool.is_shutdown());
    }

    #[test]
    fn test_health_summary() {
        let config = create_test_config();
        let pool = RpcPool::new(config).unwrap();

        // Initially all endpoints should be healthy
        let summary = pool.health_summary();
        assert_eq!(summary.healthy, 2);
        assert_eq!(summary.unhealthy, 0);
        assert_eq!(summary.total, 2);
        assert!(!summary.all_unhealthy());
        assert_eq!(summary.health_percentage(), 100.0);

        // Mark one as unhealthy
        pool.mark_unhealthy("https://rpc1.example.com");
        let summary = pool.health_summary();
        assert_eq!(summary.healthy, 1);
        assert_eq!(summary.unhealthy, 1);
        assert!(!summary.all_unhealthy());
        assert_eq!(summary.health_percentage(), 50.0);

        // Mark another as unhealthy
        pool.mark_unhealthy("https://rpc2.example.com");
        let summary = pool.health_summary();
        assert_eq!(summary.healthy, 0);
        assert_eq!(summary.unhealthy, 2);
        assert!(summary.all_unhealthy());
        assert_eq!(summary.health_percentage(), 0.0);
    }

    /// Regression for the failover-detection gap: a *currently healthy*
    /// endpoint that goes down must be detected by the background probe and
    /// marked unhealthy, even though no request traffic flows through the pool
    /// (callers that use the pool only as a URL selector via
    /// `get_current_url()`). Before the fix, `check_health` probed only
    /// already-unhealthy endpoints, so a dead primary stayed "healthy" forever
    /// and never failed over.
    #[tokio::test]
    async fn health_check_detects_dead_healthy_endpoint() {
        // Closed local ports → connection refused immediately (no network).
        let config = RpcPoolConfig::new()
            .with_endpoints(vec![
                RpcEndpoint::new("http://127.0.0.1:1")
                    .with_name("dead-primary")
                    .with_priority(1),
                RpcEndpoint::new("http://127.0.0.1:2")
                    .with_name("dead-backup")
                    .with_priority(2),
            ])
            .with_strategy(Box::new(FailoverStrategy))
            .with_max_consecutive_errors(2)
            .with_health_check_timeout(Duration::from_millis(500));
        let pool = RpcPool::new(config).unwrap();

        // Initially healthy → primary selected, nothing unhealthy.
        assert_eq!(pool.health_summary().unhealthy, 0);
        assert_eq!(
            pool.get_current_url().as_deref(),
            Some("http://127.0.0.1:1")
        );

        // One failing probe is below the threshold — must stay healthy.
        pool.check_health().await;
        assert_eq!(
            pool.health_summary().unhealthy,
            0,
            "a single probe failure must not flip a healthy endpoint"
        );

        // The second consecutive failing probe reaches max_consecutive_errors
        // → endpoints are actively marked unhealthy.
        pool.check_health().await;
        assert_eq!(
            pool.health_summary().unhealthy,
            2,
            "consecutive probe failures must mark healthy endpoints down"
        );
    }
}
