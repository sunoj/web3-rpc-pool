# Performance Baseline v0.1.0

**Date:** 2026-01-30
**Git Hash:** Initial baseline
**Rust Version:** stable

## Summary

All performance targets met.

## Strategy Selection Performance

| Strategy        | Avg Latency | Throughput      | Status |
|-----------------|-------------|-----------------|--------|
| Failover        | 71 ns       | 13.9M ops/sec   | PASS   |
| Round Robin     | 1,093 ns    | 914k ops/sec    | PASS   |
| Latency Based   | 1,824 ns    | 548k ops/sec    | PASS   |

**Target:** < 10,000 ns (10 us)

## Pool Creation Performance

| Endpoints | Avg Latency | Throughput    | Status |
|-----------|-------------|---------------|--------|
| 5         | 3.2 us      | 307k ops/sec  | PASS   |
| 10        | 4.7 us      | 211k ops/sec  | PASS   |
| 20        | 9.3 us      | 107k ops/sec  | PASS   |
| 50        | 22.3 us     | 45k ops/sec   | PASS   |

**Target:** < 1,000 us (1 ms)

## Stats Update Performance

| Operation       | Avg Latency | Throughput      | Status |
|-----------------|-------------|-----------------|--------|
| Record Success  | 40 ns       | 24.8M ops/sec   | PASS   |
| Record Failure  | 118 ns      | 8.4M ops/sec    | PASS   |

**Target:** < 1,000 ns (1 us) for success, < 2,000 ns (2 us) for failure

## Metrics Collection Performance

| Endpoints | Avg Latency | Throughput    | Status |
|-----------|-------------|---------------|--------|
| 5         | 7.6 us      | 132k ops/sec  | PASS   |
| 10        | 9.8 us      | 102k ops/sec  | PASS   |
| 20        | 13.5 us     | 74k ops/sec   | PASS   |
| 50        | 26.5 us     | 38k ops/sec   | PASS   |

**Target:** < 100 us

## Concurrent Access Performance

| Metric            | Value           | Status |
|-------------------|-----------------|--------|
| Concurrent Tasks  | 100             | -      |
| Ops per Task      | 1,000           | -      |
| Total Ops         | 100,000         | -      |
| Total Time        | 824 ms          | -      |
| Throughput        | 121k ops/sec    | PASS   |

**Target:** > 100,000 ops/sec

## Memory Efficiency

| Metric          | Value       |
|-----------------|-------------|
| Pools Created   | 100         |
| Endpoints/Pool  | 20          |
| Creation Time   | 52 ms total |
| Avg Creation    | 520 us      |
| Access Time     | 3.9 ms      |
| Avg Access      | 39 us       |

## Performance Targets Summary

| Metric                           | Target       | Actual    | Status |
|----------------------------------|--------------|-----------|--------|
| Strategy selection               | < 10 us      | 0.07 us   | PASS   |
| Pool creation (20 endpoints)     | < 1 ms       | 9.3 us    | PASS   |
| Stats update (success)           | < 1 us       | 40 ns     | PASS   |
| Stats update (failure)           | < 2 us       | 118 ns    | PASS   |
| Metrics collection (50 endpoints)| < 100 us     | 26.5 us   | PASS   |
| Concurrent throughput            | > 100k/sec   | 121k/sec  | PASS   |

---

*This baseline was established for release v0.1.0 and should be used for regression testing.*
