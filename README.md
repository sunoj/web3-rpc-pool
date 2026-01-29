# web3-rpc-pool

High-availability multi-endpoint RPC pool with automatic failover and load balancing for Web3 applications.

## Features

- **Multiple selection strategies**
  - Failover: Use highest priority healthy endpoint
  - Round-Robin: Distribute load evenly
  - Latency-Based: Always use the fastest endpoint

- **Automatic failover**: Seamlessly switches to healthy endpoints on failure

- **Health monitoring**: Periodic health checks with automatic recovery

- **Latency tracking**: Exponential moving average (EMA) for performance monitoring

- **Built-in presets**: Default endpoints for popular chains (Arbitrum, Base, Ethereum, etc.)

## Installation

```toml
[dependencies]
web3-rpc-pool = "0.1"
```

## Quick Start

```rust
use web3_rpc_pool::{RpcPool, RpcPoolConfig, strategies::FailoverStrategy, presets};
use std::time::Duration;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create pool with default Arbitrum endpoints
    let pool = Arc::new(RpcPool::new(RpcPoolConfig {
        endpoints: presets::arbitrum_endpoints(),
        strategy: Box::new(FailoverStrategy),
        health_check_interval: Duration::from_secs(60),
        max_consecutive_errors: 3,
        retry_delay: Duration::from_secs(5),
    })?);

    // Start background health checker
    let _health_task = pool.start_health_check();

    // Execute with automatic failover
    let block = pool.execute(|provider| async move {
        provider.get_block_number().await
    }).await?;

    println!("Current block: {}", block);

    // Get metrics
    let metrics = pool.metrics();
    println!("Total requests: {}", metrics.total_requests);
    println!("Failovers: {}", metrics.failovers);

    Ok(())
}
```

## Selection Strategies

### Failover (Default)

Always uses the highest priority healthy endpoint. Best for production systems with a clear primary endpoint and backups.

```rust
use web3_rpc_pool::strategies::FailoverStrategy;

let config = RpcPoolConfig {
    strategy: Box::new(FailoverStrategy),
    ..Default::default()
};
```

### Round-Robin

Cycles through healthy endpoints to distribute load evenly.

```rust
use web3_rpc_pool::strategies::RoundRobinStrategy;

let config = RpcPoolConfig {
    strategy: Box::new(RoundRobinStrategy::new()),
    ..Default::default()
};
```

### Latency-Based

Always selects the healthy endpoint with the lowest average latency.

```rust
use web3_rpc_pool::strategies::LatencyBasedStrategy;

let config = RpcPoolConfig {
    strategy: Box::new(LatencyBasedStrategy),
    ..Default::default()
};
```

## Custom Endpoints

```rust
use web3_rpc_pool::RpcEndpoint;

let endpoints = vec![
    RpcEndpoint::new("https://my-primary-rpc.com")
        .with_name("Primary")
        .with_priority(10)
        .with_chain_id(42161),
    RpcEndpoint::new("https://my-backup-rpc.com")
        .with_name("Backup")
        .with_priority(50)
        .with_chain_id(42161),
];
```

## Metrics

```rust
let metrics = pool.metrics();

// Aggregate stats
println!("Total requests: {}", metrics.total_requests);
println!("Failovers: {}", metrics.failovers);
println!("Current endpoint: {}", metrics.current_endpoint);

// Per-endpoint stats
for endpoint in &metrics.endpoints {
    println!("{}: {} req, {:.1}% success, {:.0}ms avg latency",
        endpoint.name,
        endpoint.total_requests,
        endpoint.success_rate,
        endpoint.avg_latency_ms
    );
}
```

## Supported Chains

Built-in presets for:
- Arbitrum One (42161)
- Base (8453)
- Ethereum Mainnet (1)
- Optimism (10)
- Polygon (137)

```rust
use web3_rpc_pool::presets;

let arbitrum = presets::arbitrum_endpoints();
let base = presets::base_endpoints();
let ethereum = presets::ethereum_endpoints();
```

## License

MIT
