//! # web3-rpc-pool
//!
//! High-availability multi-endpoint RPC pool with automatic failover and load balancing.
//!
//! ## Features
//!
//! - **Multiple selection strategies**: Failover, Round-Robin, Latency-based, Rate-aware
//! - **Automatic failover**: Seamlessly switches to healthy endpoints on failure
//! - **Tiered routing**: Route requests by priority (Critical/Normal/Low) to different endpoint tiers
//! - **Health monitoring**: Periodic health checks with exponential backoff recovery
//! - **Latency tracking**: Exponential moving average for performance monitoring
//! - **Built-in presets**: 100+ verified endpoints for 7 popular chains
//!
//! ## Quick Start with TieredPool
//!
//! ```rust,no_run
//! use web3_rpc_pool::{TieredPoolBuilder, RequestPriority, presets::chain_id};
//! use alloy::providers::{Provider, ProviderBuilder};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create tiered pool with auto-loaded free endpoints
//!     let pool = TieredPoolBuilder::new()
//!         .add_premium("https://arb-mainnet.g.alchemy.com/v2/YOUR_KEY", "Alchemy")
//!         .with_default_free_endpoints(chain_id::ARBITRUM_ONE)  // Auto-load 10+ free endpoints
//!         .build()?;
//!
//!     // Debug: check tier configuration
//!     pool.log_tier_info();  // Logs: Premium=1, Free=10
//!
//!     // Execute with automatic tier fallback
//!     let block = pool.execute(RequestPriority::Normal, |url: url::Url| async move {
//!         let provider = ProviderBuilder::new().connect_http(url);
//!         provider.get_block_number().await
//!     }).await?;
//!
//!     println!("Current block: {}", block);
//!     Ok(())
//! }
//! ```
//!
//! ## Basic RpcPool Example
//!
//! ```rust,no_run
//! use web3_rpc_pool::{RpcPool, RpcPoolConfig, strategies::FailoverStrategy, presets};
//! use std::sync::Arc;
//! use std::time::Duration;
//! use alloy::providers::{Provider, ProviderBuilder};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create pool with builder pattern
//!     let pool = Arc::new(RpcPool::new(
//!         RpcPoolConfig::new()
//!             .with_endpoints(presets::arbitrum_endpoints())
//!             .with_strategy(Box::new(FailoverStrategy))
//!             .with_request_timeout(Duration::from_secs(30))
//!     )?);
//!
//!     // Start background health checker
//!     let _health_task = pool.start_health_check();
//!
//!     // Execute with automatic failover
//!     let block = pool.execute(|url: url::Url| async move {
//!         let provider = ProviderBuilder::new().connect_http(url);
//!         provider.get_block_number().await
//!     }).await?;
//!
//!     println!("Current block: {}", block);
//!
//!     // Check health status
//!     let health = pool.health_summary();
//!     println!("Healthy: {}/{}", health.healthy, health.total);
//!
//!     // Graceful shutdown
//!     pool.shutdown().await;
//!     Ok(())
//! }
//! ```

pub mod endpoint;
pub mod error;
pub mod metrics;
pub mod pool;
pub mod presets;
pub mod strategies;
pub mod tiered;

pub use endpoint::{RpcEndpoint, EndpointStats};
pub use error::RpcPoolError;
pub use metrics::RpcPoolMetrics;
pub use pool::{HealthSummary, RpcPool, RpcPoolConfig};
pub use strategies::{
    FailoverStrategy, LatencyBasedStrategy, RateAwareStrategy, RoundRobinStrategy,
    SelectionStrategy,
};
pub use tiered::{
    EndpointTier, RequestPriority, TieredEndpoint, TieredPool, TieredPoolBuilder, TieredPoolConfig,
};
