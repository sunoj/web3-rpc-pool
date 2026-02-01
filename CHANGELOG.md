# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
