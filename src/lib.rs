//! # web3-rpc-pool
//!
//! High-availability multi-endpoint RPC pool with automatic failover and load balancing.
//!
//! ## Features
//!
//! - **Multiple selection strategies**: Failover, Round-Robin, Latency-based
//! - **Automatic failover**: Seamlessly switches to healthy endpoints on failure
//! - **Health monitoring**: Periodic health checks with automatic recovery
//! - **Latency tracking**: Exponential moving average for performance monitoring
//! - **Built-in presets**: Default endpoints for popular chains
//!
//! ## Example
//!
//! ```rust,no_run
//! use web3_rpc_pool::{RpcPool, RpcPoolConfig, strategies::FailoverStrategy, presets};
//! use std::sync::Arc;
//! use std::time::Duration;
//! use alloy::providers::{Provider, ProviderBuilder};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let pool = Arc::new(RpcPool::new(RpcPoolConfig {
//!         endpoints: presets::arbitrum_endpoints(),
//!         strategy: Box::new(FailoverStrategy),
//!         health_check_interval: Duration::from_secs(60),
//!         max_consecutive_errors: 3,
//!         retry_delay: Duration::from_secs(5),
//!     })?);
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
//!     Ok(())
//! }
//! ```

pub mod endpoint;
pub mod error;
pub mod metrics;
pub mod pool;
pub mod presets;
pub mod strategies;

pub use endpoint::{RpcEndpoint, EndpointStats};
pub use error::RpcPoolError;
pub use metrics::RpcPoolMetrics;
pub use pool::{RpcPool, RpcPoolConfig};
pub use strategies::SelectionStrategy;
