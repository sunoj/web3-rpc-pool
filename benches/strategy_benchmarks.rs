//! Benchmark tests for endpoint selection strategies.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use web3_rpc_pool::endpoint::{EndpointStats, RpcEndpoint};
use web3_rpc_pool::presets::chain_id;
use web3_rpc_pool::strategies::{
    FailoverStrategy, LatencyBasedStrategy, RoundRobinStrategy, SelectionStrategy,
};

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

fn create_test_stats(endpoints: &[RpcEndpoint], healthy_ratio: f64) -> HashMap<String, EndpointStats> {
    let healthy_count = (endpoints.len() as f64 * healthy_ratio) as usize;
    endpoints
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let mut stats = EndpointStats::new(e);
            if i >= healthy_count {
                stats.is_healthy = false;
            }
            // Add some latency data
            stats.avg_latency_ms = 50.0 + (i as f64 * 10.0);
            (e.url.clone(), stats)
        })
        .collect()
}

fn bench_failover_strategy(c: &mut Criterion) {
    let mut group = c.benchmark_group("failover_strategy");

    for endpoint_count in [5, 10, 20, 50, 100] {
        let endpoints = create_test_endpoints(endpoint_count);
        let stats = create_test_stats(&endpoints, 0.8);
        let tried = HashSet::new();

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("select", endpoint_count),
            &endpoint_count,
            |b, _| {
                let mut strategy = FailoverStrategy;
                b.iter(|| {
                    black_box(strategy.select(&endpoints, &stats, &tried));
                });
            },
        );
    }

    group.finish();
}

fn bench_round_robin_strategy(c: &mut Criterion) {
    let mut group = c.benchmark_group("round_robin_strategy");

    for endpoint_count in [5, 10, 20, 50, 100] {
        let endpoints = create_test_endpoints(endpoint_count);
        let stats = create_test_stats(&endpoints, 0.8);
        let tried = HashSet::new();

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("select", endpoint_count),
            &endpoint_count,
            |b, _| {
                let mut strategy = RoundRobinStrategy::new();
                b.iter(|| {
                    black_box(strategy.select(&endpoints, &stats, &tried));
                });
            },
        );
    }

    group.finish();
}

fn bench_latency_based_strategy(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency_based_strategy");

    for endpoint_count in [5, 10, 20, 50, 100] {
        let endpoints = create_test_endpoints(endpoint_count);
        let stats = create_test_stats(&endpoints, 0.8);
        let tried = HashSet::new();

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("select", endpoint_count),
            &endpoint_count,
            |b, _| {
                let mut strategy = LatencyBasedStrategy;
                b.iter(|| {
                    black_box(strategy.select(&endpoints, &stats, &tried));
                });
            },
        );
    }

    group.finish();
}

fn bench_strategy_with_exclusions(c: &mut Criterion) {
    let mut group = c.benchmark_group("strategy_with_exclusions");
    let endpoints = create_test_endpoints(20);
    let stats = create_test_stats(&endpoints, 1.0);

    for exclusion_count in [0, 5, 10, 15, 19] {
        let tried: HashSet<String> = endpoints
            .iter()
            .take(exclusion_count)
            .map(|e| e.url.clone())
            .collect();

        group.bench_with_input(
            BenchmarkId::new("failover", exclusion_count),
            &exclusion_count,
            |b, _| {
                let mut strategy = FailoverStrategy;
                b.iter(|| {
                    black_box(strategy.select(&endpoints, &stats, &tried));
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("latency_based", exclusion_count),
            &exclusion_count,
            |b, _| {
                let mut strategy = LatencyBasedStrategy;
                b.iter(|| {
                    black_box(strategy.select(&endpoints, &stats, &tried));
                });
            },
        );
    }

    group.finish();
}

fn bench_stats_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("endpoint_stats");
    let endpoint = RpcEndpoint::new("https://rpc.example.com");

    group.bench_function("record_success", |b| {
        let mut stats = EndpointStats::new(&endpoint);
        b.iter(|| {
            stats.record_success(black_box(50));
        });
    });

    group.bench_function("record_failure", |b| {
        let mut stats = EndpointStats::new(&endpoint);
        b.iter(|| {
            stats.record_failure(black_box("error".to_string()), 3);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_failover_strategy,
    bench_round_robin_strategy,
    bench_latency_based_strategy,
    bench_strategy_with_exclusions,
    bench_stats_update,
);
criterion_main!(benches);
