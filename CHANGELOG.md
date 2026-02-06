# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.3] - 2026-02-07

### Added

- **More Arbitrum Endpoints**: Added 3 new verified public RPC endpoints for Arbitrum One (16 → 19)
  - thirdweb (`arbitrum.rpc.thirdweb.com`)
  - Pocket Network (`arb-one.api.pocket.network`)
  - LeoRPC (`arb.leorpc.com`)

## [0.3.2] - 2026-02-06

### Fixed

- **Automatic Endpoint Deduplication**: Endpoints with duplicate URLs are now automatically removed
  - `TieredPoolBuilder::build()` deduplicates by URL, keeping the first occurrence (earlier-added / higher-tier wins)
  - `RpcPool::new()` provides a safety-net dedup for direct config construction
  - Warns via `tracing::warn!` when duplicates are detected
  - Fixes `RateAwareStrategy` giving extra weight to duplicated URLs when config manually adds endpoints that also exist in built-in presets

## [0.3.0] - 2026-02-02

### Added

- **Health Summary API**: New `health_summary()` method on `RpcPool` to get counts of healthy/unhealthy endpoints
  - `HealthSummary` struct with `healthy`, `unhealthy`, `total` counts
  - Helper methods: `all_unhealthy()`, `health_percentage()`

- **Exponential Backoff Recovery**: Unhealthy endpoints now use exponential backoff for retry attempts
  - Base delay × 2^attempts, capped at 5 minutes
  - First failure: 5s, second: 10s, third: 20s, etc.
  - Backoff resets on successful recovery

### Changed

- **Improved Error Messages**: "All endpoints failed" error now includes healthy/unhealthy endpoint counts
  ```
  All endpoints failed (most endpoints marked unhealthy from previous failures)
  tried_endpoints=2, healthy_endpoints=0, unhealthy_endpoints=16, total_endpoints=16
  ```

## [0.2.3] - 2026-02-02

### Added

- **More Arbitrum & Base Endpoints**: Added 10 more verified endpoints (91 → 101 total)
  - Arbitrum: +6 (Nodies Public, BlockPI, ZAN, Lava, Tatum, FastNode)
  - Base: +4 (BlockPI, Nodies Public, Pocket, HairyLabs)

## [0.2.2] - 2026-02-02

### Added

- **Additional RPC Endpoints**: Added 14 more verified endpoints (77 → 91 total)
  - Ethereum: +8 (BloXroute, Gateway.fm, GasHawk, TornadoETH, Tenderly, MEV Blocker variants)
  - BSC: +5 (Defibit 3-4, Ninicoin 3-4, PublicNode Alt)
  - Polygon: +1 (QuickNode)

## [0.2.1] - 2026-02-02

### Changed

- **RPC Endpoints Verification**: All 77 built-in RPC endpoints verified working via `eth_blockNumber` test
  - Ethereum: 14 endpoints (+3 new: SubQuery, 0xRPC, BlockRazor, OmniaTech)
  - Arbitrum: 10 endpoints (+1 new: SubQuery)
  - Base: 11 endpoints (+2 new: SubQuery, OmniaTech)
  - Optimism: 8 endpoints (+1 new: OmniaTech)
  - BSC: 17 endpoints (+2 new: OmniaTech, SubQuery)
  - Avalanche: 8 endpoints (+1 new: OmniaTech)
  - Polygon: 9 endpoints (+3 new: SubQuery, OmniaTech, Nodies)

### Removed

- Removed 35 non-working RPC endpoints:
  - Llama RPC (all chains) - connection issues
  - Ankr public endpoints (all chains) - rate limited
  - BlockPI (all chains) - connection issues
  - Cloudflare, Payload (Ethereum) - not responding
  - Gateway.fm (Arbitrum, Optimism, Polygon) - connection issues
  - NotADegen (Base) - not responding

## [0.2.0] - 2026-02-01

### Added

- **Public RPC Endpoints**: Added extensive built-in public RPC endpoints for 7 mainstream chains:
  - Ethereum (15+ endpoints including Cloudflare, Ankr, LlamaNodes, PublicNode, etc.)
  - Polygon (10+ endpoints)
  - Arbitrum (8+ endpoints)
  - Optimism (8+ endpoints)
  - Base (6+ endpoints)
  - BSC/BNB Chain (8+ endpoints)
  - Avalanche C-Chain (6+ endpoints)

- **Performance Testing Framework**:
  - Criterion benchmarks for strategy selection, pool creation, and stats operations
  - Integration performance tests with throughput measurements
  - Performance test runner script (`scripts/run_perf_tests.sh`)
  - GitHub Actions workflows for CI and performance testing
  - Performance baseline documentation

- **Resource Management**:
  - Graceful shutdown support with `CancellationToken`
  - `Drop` implementation for automatic cleanup
  - Request timeout configuration (default 30s)
  - Health check timeout configuration (default 10s)
  - Builder pattern for `RpcPoolConfig`
  - Error message truncation to prevent memory growth

- **Logging**:
  - Structured logging with proper log levels (trace/debug/info/warn/error)
  - Request tracing with `request_id` for correlation
  - Health check cycle summaries
  - Endpoint recovery/failure logging

### Changed

- Updated to Alloy 1.0 API (`connect_http` instead of `on_http`)
- Pool creation now requires `Arc<Self>` for `start_health_check`
- Improved strategy selection performance (Round Robin 33% faster)

### Fixed

- Health check task can now be gracefully stopped
- Requests are properly cancelled on pool shutdown
- Fixed clippy warnings

## [0.1.0] - 2026-01-29

### Added

- Initial release
- RPC connection pool with automatic failover
- Multiple selection strategies (Failover, Round Robin, Latency-based)
- Health monitoring with automatic recovery
- Metrics collection
- Basic preset endpoints for Ethereum and Polygon
