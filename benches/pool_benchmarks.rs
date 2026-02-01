//! Benchmark tests for RPC pool operations.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;
use web3_rpc_pool::endpoint::RpcEndpoint;
use web3_rpc_pool::presets::{self, chain_id};
use web3_rpc_pool::strategies::{FailoverStrategy, LatencyBasedStrategy, RoundRobinStrategy};
use web3_rpc_pool::{RpcPool, RpcPoolConfig};

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

fn bench_pool_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_creation");

    for endpoint_count in [5, 10, 20, 50, 100] {
        let endpoints = create_test_endpoints(endpoint_count);

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("with_failover", endpoint_count),
            &endpoints,
            |b, endpoints| {
                b.iter(|| {
                    let config = RpcPoolConfig::new()
                        .with_endpoints(endpoints.clone())
                        .with_strategy(Box::new(FailoverStrategy))
                        .with_health_check_interval(Duration::from_secs(60))
                        .with_max_consecutive_errors(3)
                        .with_retry_delay(Duration::from_secs(5));
                    black_box(RpcPool::new(config).unwrap())
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("with_round_robin", endpoint_count),
            &endpoints,
            |b, endpoints| {
                b.iter(|| {
                    let config = RpcPoolConfig::new()
                        .with_endpoints(endpoints.clone())
                        .with_strategy(Box::new(RoundRobinStrategy::new()))
                        .with_health_check_interval(Duration::from_secs(60))
                        .with_max_consecutive_errors(3)
                        .with_retry_delay(Duration::from_secs(5));
                    black_box(RpcPool::new(config).unwrap())
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("with_latency_based", endpoint_count),
            &endpoints,
            |b, endpoints| {
                b.iter(|| {
                    let config = RpcPoolConfig::new()
                        .with_endpoints(endpoints.clone())
                        .with_strategy(Box::new(LatencyBasedStrategy))
                        .with_health_check_interval(Duration::from_secs(60))
                        .with_max_consecutive_errors(3)
                        .with_retry_delay(Duration::from_secs(5));
                    black_box(RpcPool::new(config).unwrap())
                });
            },
        );
    }

    group.finish();
}

fn bench_preset_endpoints(c: &mut Criterion) {
    let mut group = c.benchmark_group("preset_endpoints");

    group.bench_function("ethereum_endpoints", |b| {
        b.iter(|| black_box(presets::ethereum_endpoints()));
    });

    group.bench_function("polygon_endpoints", |b| {
        b.iter(|| black_box(presets::polygon_endpoints()));
    });

    group.bench_function("arbitrum_endpoints", |b| {
        b.iter(|| black_box(presets::arbitrum_endpoints()));
    });

    group.bench_function("bsc_endpoints", |b| {
        b.iter(|| black_box(presets::bsc_endpoints()));
    });

    group.bench_function("avalanche_endpoints", |b| {
        b.iter(|| black_box(presets::avalanche_endpoints()));
    });

    group.finish();
}

fn bench_pool_get_url(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_get_url");

    for endpoint_count in [5, 10, 20, 50] {
        let endpoints = create_test_endpoints(endpoint_count);
        let config = RpcPoolConfig::new()
            .with_endpoints(endpoints)
            .with_strategy(Box::new(FailoverStrategy))
            .with_health_check_interval(Duration::from_secs(60))
            .with_max_consecutive_errors(3)
            .with_retry_delay(Duration::from_secs(5));
        let pool = RpcPool::new(config).unwrap();

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("get_current_url", endpoint_count),
            &pool,
            |b, pool| {
                b.iter(|| black_box(pool.get_current_url()));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("get_all_urls", endpoint_count),
            &pool,
            |b, pool| {
                b.iter(|| black_box(pool.get_all_urls()));
            },
        );
    }

    group.finish();
}

fn bench_pool_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_metrics");

    for endpoint_count in [5, 10, 20, 50] {
        let endpoints = create_test_endpoints(endpoint_count);
        let config = RpcPoolConfig::new()
            .with_endpoints(endpoints)
            .with_strategy(Box::new(FailoverStrategy))
            .with_health_check_interval(Duration::from_secs(60))
            .with_max_consecutive_errors(3)
            .with_retry_delay(Duration::from_secs(5));
        let pool = RpcPool::new(config).unwrap();

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("collect_metrics", endpoint_count),
            &pool,
            |b, pool| {
                b.iter(|| black_box(pool.metrics()));
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_pool_creation,
    bench_preset_endpoints,
    bench_pool_get_url,
    bench_pool_metrics,
);
criterion_main!(benches);
