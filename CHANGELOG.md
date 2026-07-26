# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2026-07-26

### Changed

- **`reqwest` 0.12 → 0.13, and the ring `CryptoProvider` is now installed by this library.**
  This is a **breaking** change in behaviour, hence the minor bump, even though no public type
  changed.

  reqwest 0.13 offers only two rustls options and **removed `rustls-tls-webpki-roots`**:

  | feature | provider | roots |
  |---|---|---|
  | `rustls` | `aws-lc-rs` → `aws-lc-sys` (C/asm, **hostile to cross-compilation**) | platform verifier |
  | `rustls-no-provider` | none — the process must install one or `Client::build()` **panics** | platform verifier |

  We take `rustls-no-provider` to stay cross-compile clean (macOS → Linux), and install `ring`
  ourselves in the new `tls` module. `ensure_provider()` is `Once`-guarded, idempotent and
  thread-safe, and is called before every client this crate builds: `TieredPool::new` (the shared
  keep-alive RPC client) and `RpcPool::new` (the health-probe client).

  **Doing this in the library rather than the caller is the point.** `RpcPool::new` builds a
  `reqwest::Client` internally, so requiring callers to install a provider would make this bump
  panic at pool construction for every existing consumer.

### Breaking

- **Root certificates now come from the OS trust store** (`rustls-platform-verifier`) instead of a
  bundled `webpki-roots` set, because reqwest 0.13 deleted that feature. **Hosts must have a CA
  bundle installed** (e.g. `ca-certificates`). Verify this before deploying.
- **This crate no longer enables a provider-selecting reqwest feature.** A consumer that was
  relying on our old `rustls-tls` (which implied `__rustls-ring`) to supply a provider for *its
  own* clients must now either enable one itself or call
  `web3_rpc_pool::tls::ensure_provider()`. Consumers that declare their own reqwest with a
  provider feature are unaffected — `liquidation-engine` was checked and is fine, because it
  declares `reqwest 0.12` with `rustls-tls`.

### Fixed

- **A comment in `Cargo.toml` documented an invariant that did not hold.** It claimed the ring
  provider was pinned "via our own reqwest dependency", but in any tree where `alloy` pulls
  reqwest 0.13, the alloy HTTP transport links *that*, not our 0.12 pin. Unifying on one reqwest
  version makes the claim checkable and true.
- **`tests/live_endpoint_tests.rs` builds a `reqwest::Client` directly**, not through `RpcPool`, so
  the library's internal self-install did not cover it. It regressed on the first attempt at this
  change and was caught by running the unmodified tree as a control. The general limit is worth
  knowing: **`ensure_provider()` covers clients this library builds, not clients a consumer
  builds.**

### Verification

- 115 tests pass, 0 fail — including the live network endpoint test making real calls.
- `aws-lc` is **absent** under `--no-default-features --features http`. It still appears under
  `--all-features` via the `ws` feature, because `alloy-transport-ws` hard-codes `aws_lc_rs`; that
  is a pre-existing caveat already documented in `Cargo.toml`.
- Verified to build from a clean clone of the release branch (an earlier commit had omitted
  `src/tls.rs` entirely — `git commit -a` does not stage new files).

## [0.5.8] - 2026-07-20

### Fixed

- **Selector-only consumers could keep picking a dead endpoint forever**: `check_health()` previously probed only endpoints already marked unhealthy, so a primary that died while healthy was never re-tested and `get_current_url()` kept returning it. Every healthy endpoint now gets one `eth_blockNumber` probe per health-check cycle; repeated failures trip `max_consecutive_errors` and failover proceeds. Cost is one probe per healthy endpoint per interval.

### Added

- **Robinhood Chain (4663) preset** — public mainnet RPC, free/read-only tier. Also added to `all_chain_ids()` and `chain_name()`, which it was missing.
- **`NO_PUBLIC_WS_CHAINS`** — an explicit list of chains that publish no WebSocket endpoint (currently Robinhood). The WS-coverage invariant now skips them by name rather than being weakened for everyone, and a companion test asserts those chains really have no `ws_url`. A consumer that assumed `newHeads` existed on 4663 ran with a block clock frozen at 0 for eleven days.

## [0.5.2] - 2026-03-05

### Fixed

- **WebSocket subscriptions drop immediately**: All WS subscription streams (`subscribe_new_heads`, `subscribe_pending_transactions`, `subscribe_logs`, and standalone helpers) now keep the WS provider alive for the lifetime of the stream. Previously the provider was dropped when the subscribe function returned, causing the underlying transport to shut down and the stream to yield `None` immediately ("Pubsub service request channel closed").

### Added

- `OwnedStream` wrapper that co-owns the WS provider alongside the subscription stream
- Tests verifying stream lifetime and owner drop semantics

## [0.5.1] - 2026-02-27

### Added

- **Evaluator retry mode**: New `--retry-from` flag to re-evaluate only failed (grade F) endpoints from a previous report, merging results with previously successful endpoints

### Changed

- **Evaluator default concurrency**: Changed from 4 to 1 to avoid triggering IP rate limits on public RPC providers, improving reachability from ~39% to ~80%

## [0.5.0] - 2026-02-18

### Added

- **WebSocket Subscription Support**: New `WsPool` module with automatic failover across WebSocket endpoints
  - `subscribe_new_heads()`, `subscribe_pending_transactions()`, `subscribe_logs()` with stream-based API
  - Automatic reconnection with exponential backoff on connection drops
  - 43 WSS endpoints across all 38 chains (dRPC, PublicNode, BlockPI, official providers)
  - New `ws` feature flag (enabled by default)

- **21 New EVM Chains**: Expanded from 17 to 38 supported chains with 68 new endpoints
  - Gnosis (8), Sonic (6), Moonbeam (5), Celo (4), Metis (4), opBNB (4), Aurora (3), Berachain (3), Fraxtal (3), Fuse (3), Kava (3), Klaytn (3), Taiko (3), Cronos (2), Harmony (2), Immutable zkEVM (2), Lisk (2), Rootstock (2), Sei (2), World Chain (2), ZetaChain (2)

- **27 New Verified Endpoints** for existing chains from GitHub and aggregator sources
  - Ethereum +5, zkSync Era +3, Scroll +3, Mantle +3, BSC +2, Linea +2, Blast +2, and more

### Changed

- **Slimmed Dependencies**: Removed 4 unused crates (futures, async-trait, dashmap, tokio-util), reduced alloy features from `full` to minimal set
  - Total crates: 339 → 272 (-20%), clean build: 45.3s → 39.8s (-12%)
  - Replaced `DashMap` with `RwLock<HashMap>`, `CancellationToken` with `AtomicBool+Notify`

- **Major Endpoint Audit**: Live-tested all endpoints, removed 83 dead, added 23 new verified
  - Removed: Ankr (requires API key), OmniaTech (521 errors), BlastAPI/BlockPI (discontinued/TLS), and others
  - Net result: 213 → 276 endpoints, all verified reachable

## [0.4.0] - 2026-02-07

### Added

- **6 New DeFi L2 Chains**: Added 55 verified public RPC endpoints across 6 new chains (158 → 213 total, 11 → 17 chains)
  - **Scroll** (534352): 12 endpoints — Official, PublicNode, 1RPC, dRPC, BlastAPI, Ankr, OnFinality, Nodies, OmniaTech, thirdweb, Pocket Network, IceCreamSwap
  - **Polygon zkEVM** (1101): 8 endpoints — Official, 1RPC, BlastAPI, dRPC, Nodies, OmniaTech, Pocket Network, Gateway.fm
  - **Blast** (81457): 11 endpoints — Official, PublicNode, dRPC, BlastAPI, OmniaTech, Pocket Network, BlockPI, thirdweb, DIN, Ankr, OnFinality
  - **Mantle** (5000): 12 endpoints — Official, PublicNode, 1RPC, dRPC, BlastAPI, Nodies, OnFinality, ZAN, OmniaTech, Pocket Network, thirdweb, UncleZak
  - **Mode** (34443): 5 endpoints — Official, 1RPC, dRPC, thirdweb, Tenderly
  - **Manta Pacific** (169): 7 endpoints — Official, 1RPC, dRPC, thirdweb, Caldera, Caldera Aperture, Ankr

- **Endpoint Capability Grading**: All new chain endpoints evaluated and graded (A/B/C/D/F) with embedded capability data
  - `supports_eth_get_logs`, `max_batch_size`, `max_block_range` measured per endpoint
  - Capability data baked in as defaults for instant availability

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
