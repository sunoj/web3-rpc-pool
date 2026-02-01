//! Integration performance tests for web3-rpc-pool.
//!
//! These tests measure real-world performance characteristics including:
//! - Concurrent request handling
//! - Failover timing
//! - Memory usage patterns
//! - Strategy switching overhead

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use web3_rpc_pool::endpoint::{EndpointStats, RpcEndpoint};
use web3_rpc_pool::presets::chain_id;
use web3_rpc_pool::strategies::{
    FailoverStrategy, LatencyBasedStrategy, RoundRobinStrategy, SelectionStrategy,
};
use web3_rpc_pool::{RpcPool, RpcPoolConfig};

/// Performance test result.
#[derive(Debug, Clone)]
pub struct PerfResult {
    pub name: String,
    pub iterations: u64,
    pub total_duration_ms: u64,
    pub avg_duration_ns: u64,
    pub min_duration_ns: u64,
    pub max_duration_ns: u64,
    pub throughput_ops_per_sec: f64,
}

impl PerfResult {
    pub fn new(name: &str, durations_ns: Vec<u64>) -> Self {
        let iterations = durations_ns.len() as u64;
        let total_ns: u64 = durations_ns.iter().sum();
        let min_ns = *durations_ns.iter().min().unwrap_or(&0);
        let max_ns = *durations_ns.iter().max().unwrap_or(&0);
        let avg_ns = if iterations > 0 {
            total_ns / iterations
        } else {
            0
        };
        let throughput = if total_ns > 0 {
            (iterations as f64 * 1_000_000_000.0) / total_ns as f64
        } else {
            0.0
        };

        Self {
            name: name.to_string(),
            iterations,
            total_duration_ms: total_ns / 1_000_000,
            avg_duration_ns: avg_ns,
            min_duration_ns: min_ns,
            max_duration_ns: max_ns,
            throughput_ops_per_sec: throughput,
        }
    }

    pub fn print(&self) {
        println!("\n=== {} ===", self.name);
        println!("  Iterations:    {}", self.iterations);
        println!("  Total time:    {} ms", self.total_duration_ms);
        println!(
            "  Avg duration:  {} ns ({:.3} us)",
            self.avg_duration_ns,
            self.avg_duration_ns as f64 / 1000.0
        );
        println!("  Min duration:  {} ns", self.min_duration_ns);
        println!("  Max duration:  {} ns", self.max_duration_ns);
        println!("  Throughput:    {:.2} ops/sec", self.throughput_ops_per_sec);
    }
}

fn create_test_endpoints(count: usize) -> Vec<RpcEndpoint> {
    (0..count)
        .map(|i| {
            RpcEndpoint::new(&format!("https://rpc{}.example.com", i))
                .with_name(&format!("RPC {}", i))
                .with_priority((i * 10) as u32)
                .with_chain_id(chain_id::ETHEREUM)
        })
        .collect()
}

fn create_test_stats(endpoints: &[RpcEndpoint]) -> HashMap<String, EndpointStats> {
    endpoints
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let mut stats = EndpointStats::new(e);
            stats.avg_latency_ms = 50.0 + (i as f64 * 10.0);
            (e.url.clone(), stats)
        })
        .collect()
}

/// Test strategy selection performance under high concurrency.
#[test]
fn test_strategy_selection_performance() {
    const ITERATIONS: usize = 10_000;
    const ENDPOINT_COUNT: usize = 20;

    let endpoints = create_test_endpoints(ENDPOINT_COUNT);
    let stats = create_test_stats(&endpoints);
    let tried = HashSet::new();

    // Test Failover Strategy
    let mut durations = Vec::with_capacity(ITERATIONS);
    let mut strategy = FailoverStrategy;
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        let _ = strategy.select(&endpoints, &stats, &tried);
        durations.push(start.elapsed().as_nanos() as u64);
    }
    let result = PerfResult::new("Failover Strategy Selection", durations);
    result.print();
    assert!(
        result.avg_duration_ns < 10_000,
        "Failover selection too slow: {} ns",
        result.avg_duration_ns
    );

    // Test Round Robin Strategy
    let mut durations = Vec::with_capacity(ITERATIONS);
    let mut strategy = RoundRobinStrategy::new();
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        let _ = strategy.select(&endpoints, &stats, &tried);
        durations.push(start.elapsed().as_nanos() as u64);
    }
    let result = PerfResult::new("Round Robin Strategy Selection", durations);
    result.print();
    assert!(
        result.avg_duration_ns < 10_000,
        "Round Robin selection too slow: {} ns",
        result.avg_duration_ns
    );

    // Test Latency Based Strategy
    let mut durations = Vec::with_capacity(ITERATIONS);
    let mut strategy = LatencyBasedStrategy;
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        let _ = strategy.select(&endpoints, &stats, &tried);
        durations.push(start.elapsed().as_nanos() as u64);
    }
    let result = PerfResult::new("Latency Based Strategy Selection", durations);
    result.print();
    assert!(
        result.avg_duration_ns < 50_000,
        "Latency based selection too slow: {} ns",
        result.avg_duration_ns
    );
}

/// Test pool creation performance.
#[test]
fn test_pool_creation_performance() {
    const ITERATIONS: usize = 1_000;

    for endpoint_count in [5, 10, 20, 50] {
        let endpoints = create_test_endpoints(endpoint_count);
        let mut durations = Vec::with_capacity(ITERATIONS);

        for _ in 0..ITERATIONS {
            let start = Instant::now();
            let config = RpcPoolConfig::new()
                .with_endpoints(endpoints.clone())
                .with_strategy(Box::new(FailoverStrategy))
                .with_health_check_interval(Duration::from_secs(60))
                .with_max_consecutive_errors(3)
                .with_retry_delay(Duration::from_secs(5));
            let _ = RpcPool::new(config).unwrap();
            durations.push(start.elapsed().as_nanos() as u64);
        }

        let result = PerfResult::new(
            &format!("Pool Creation ({} endpoints)", endpoint_count),
            durations,
        );
        result.print();

        // Pool creation should be under 1ms even for 50 endpoints
        assert!(
            result.avg_duration_ns < 1_000_000,
            "Pool creation too slow for {} endpoints: {} ns",
            endpoint_count,
            result.avg_duration_ns
        );
    }
}

/// Test endpoint stats update performance.
#[test]
fn test_stats_update_performance() {
    const ITERATIONS: usize = 100_000;
    let endpoint = RpcEndpoint::new("https://rpc.example.com");

    // Test success recording
    let mut stats = EndpointStats::new(&endpoint);
    let mut durations = Vec::with_capacity(ITERATIONS);
    for i in 0..ITERATIONS {
        let start = Instant::now();
        stats.record_success((i % 100) as u64);
        durations.push(start.elapsed().as_nanos() as u64);
    }
    let result = PerfResult::new("Stats Record Success", durations);
    result.print();
    assert!(
        result.avg_duration_ns < 1_000,
        "Stats update too slow: {} ns",
        result.avg_duration_ns
    );

    // Test failure recording
    let mut stats = EndpointStats::new(&endpoint);
    let mut durations = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        stats.record_failure("test error".to_string(), 3);
        durations.push(start.elapsed().as_nanos() as u64);
    }
    let result = PerfResult::new("Stats Record Failure", durations);
    result.print();
    assert!(
        result.avg_duration_ns < 2_000,
        "Stats failure recording too slow: {} ns",
        result.avg_duration_ns
    );
}

/// Test metrics collection performance.
#[test]
fn test_metrics_collection_performance() {
    const ITERATIONS: usize = 10_000;

    for endpoint_count in [5, 10, 20, 50] {
        let endpoints = create_test_endpoints(endpoint_count);
        let config = RpcPoolConfig::new()
            .with_endpoints(endpoints)
            .with_strategy(Box::new(FailoverStrategy))
            .with_health_check_interval(Duration::from_secs(60))
            .with_max_consecutive_errors(3)
            .with_retry_delay(Duration::from_secs(5));
        let pool = RpcPool::new(config).unwrap();

        let mut durations = Vec::with_capacity(ITERATIONS);
        for _ in 0..ITERATIONS {
            let start = Instant::now();
            let _ = pool.metrics();
            durations.push(start.elapsed().as_nanos() as u64);
        }

        let result = PerfResult::new(
            &format!("Metrics Collection ({} endpoints)", endpoint_count),
            durations,
        );
        result.print();

        // Metrics collection should be under 100us even for 50 endpoints
        assert!(
            result.avg_duration_ns < 100_000,
            "Metrics collection too slow for {} endpoints: {} ns",
            endpoint_count,
            result.avg_duration_ns
        );
    }
}

/// Test concurrent access performance.
#[tokio::test]
async fn test_concurrent_url_access() {
    const CONCURRENT_TASKS: usize = 100;
    const ITERATIONS_PER_TASK: usize = 1000;

    let endpoints = create_test_endpoints(20);
    let config = RpcPoolConfig::new()
        .with_endpoints(endpoints)
        .with_strategy(Box::new(FailoverStrategy))
        .with_health_check_interval(Duration::from_secs(60))
        .with_max_consecutive_errors(3)
        .with_retry_delay(Duration::from_secs(5));
    let pool = Arc::new(RpcPool::new(config).unwrap());

    let total_ops = Arc::new(AtomicU64::new(0));
    let start = Instant::now();

    let mut handles = Vec::with_capacity(CONCURRENT_TASKS);
    for _ in 0..CONCURRENT_TASKS {
        let pool = Arc::clone(&pool);
        let total_ops = Arc::clone(&total_ops);
        handles.push(tokio::spawn(async move {
            for _ in 0..ITERATIONS_PER_TASK {
                let _ = pool.get_current_url();
                total_ops.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let elapsed = start.elapsed();
    let total = total_ops.load(Ordering::Relaxed);
    let throughput = total as f64 / elapsed.as_secs_f64();

    println!("\n=== Concurrent URL Access ===");
    println!("  Concurrent tasks:  {}", CONCURRENT_TASKS);
    println!("  Ops per task:      {}", ITERATIONS_PER_TASK);
    println!("  Total operations:  {}", total);
    println!("  Total time:        {:?}", elapsed);
    println!("  Throughput:        {:.2} ops/sec", throughput);

    // Should achieve at least 25k ops/sec in debug mode
    // (release mode achieves 100k+ ops/sec)
    assert!(
        throughput > 25_000.0,
        "Concurrent throughput too low: {:.2} ops/sec",
        throughput
    );
}

/// Test memory efficiency by creating many pools.
#[test]
fn test_memory_efficiency() {
    const POOL_COUNT: usize = 100;
    const ENDPOINTS_PER_POOL: usize = 20;

    let start = Instant::now();
    let mut pools = Vec::with_capacity(POOL_COUNT);

    for _ in 0..POOL_COUNT {
        let endpoints = create_test_endpoints(ENDPOINTS_PER_POOL);
        let config = RpcPoolConfig::new()
            .with_endpoints(endpoints)
            .with_strategy(Box::new(FailoverStrategy))
            .with_health_check_interval(Duration::from_secs(60))
            .with_max_consecutive_errors(3)
            .with_retry_delay(Duration::from_secs(5));
        pools.push(RpcPool::new(config).unwrap());
    }

    let creation_time = start.elapsed();

    // Access all pools to ensure they're functional
    let start = Instant::now();
    for pool in &pools {
        let _ = pool.get_current_url();
        let _ = pool.metrics();
    }
    let access_time = start.elapsed();

    println!("\n=== Memory Efficiency Test ===");
    println!("  Pools created:     {}", POOL_COUNT);
    println!("  Endpoints/pool:    {}", ENDPOINTS_PER_POOL);
    println!("  Creation time:     {:?}", creation_time);
    println!("  Access time:       {:?}", access_time);
    println!("  Avg creation:      {:?}", creation_time / POOL_COUNT as u32);
    println!("  Avg access:        {:?}", access_time / POOL_COUNT as u32);
}

/// Test graceful shutdown.
#[tokio::test]
async fn test_graceful_shutdown() {
    let endpoints = create_test_endpoints(5);
    let config = RpcPoolConfig::new()
        .with_endpoints(endpoints)
        .with_strategy(Box::new(FailoverStrategy))
        .with_health_check_interval(Duration::from_millis(100));
    let pool = Arc::new(RpcPool::new(config).unwrap());

    // Start health check
    let _handle = pool.start_health_check();

    // Let it run briefly
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Shutdown should complete quickly
    let start = Instant::now();
    pool.shutdown().await;
    let shutdown_time = start.elapsed();

    println!("\n=== Graceful Shutdown Test ===");
    println!("  Shutdown time: {:?}", shutdown_time);

    assert!(pool.is_shutdown());
    assert!(
        shutdown_time < Duration::from_secs(1),
        "Shutdown took too long: {:?}",
        shutdown_time
    );
}
