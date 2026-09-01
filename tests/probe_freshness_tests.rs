//! Verifies freshness selection across real background health-probe cycles.
//! Exports: integration regressions for transient probe failures.
//! Deps: RpcPool public API, tokio, wiremock.

use std::{sync::Arc, time::Duration};
use web3_rpc_pool::{FailoverStrategy, RpcEndpoint, RpcPool, RpcPoolConfig};
use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

async fn mount_head(server: &MockServer, block_hex: &str) {
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": block_hex,
        })))
        .mount(server)
        .await;
}

async fn wait_for_head(pool: &RpcPool, url: &str, expected: u64) {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let observed = pool.metrics().endpoints.into_iter().find(|item| item.url == url);
            if observed.is_some_and(|item| item.latest_block_number == Some(expected)) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("health probe should publish the endpoint head");
}

async fn wait_for_errors(pool: &RpcPool, url: &str, expected: u32) {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let observed = pool.metrics().endpoints.into_iter().find(|item| item.url == url);
            if observed.is_some_and(|item| item.consecutive_errors == expected) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("health probe should publish the endpoint error count");
}

#[tokio::test]
async fn transient_primary_probe_failure_keeps_fresh_primary_selected() {
    let primary = MockServer::start().await;
    let fallback = MockServer::start().await;
    mount_head(&primary, "0x104").await;
    mount_head(&fallback, "0x64").await;
    let pool = Arc::new(
        RpcPool::new(
            RpcPoolConfig::new()
                .with_endpoints(vec![
                    RpcEndpoint::new(primary.uri()).with_priority(1),
                    RpcEndpoint::new(fallback.uri()).with_priority(2),
                ])
                .with_strategy(Box::new(FailoverStrategy))
                .with_health_check_interval(Duration::from_secs(60))
                .with_health_check_timeout(Duration::from_secs(1))
                .with_max_consecutive_errors(3)
                .with_max_block_lag(50),
        )
        .expect("pool"),
    );

    let initial_probe = pool.start_health_check();
    wait_for_head(&pool, &primary.uri(), 260).await;
    wait_for_head(&pool, &fallback.uri(), 100).await;
    initial_probe.abort();
    primary.reset().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&primary)
        .await;

    let failing_probe = pool.start_health_check();
    wait_for_errors(&pool, &primary.uri(), 1).await;
    failing_probe.abort();

    assert_eq!(pool.get_current_url().as_deref(), Some(primary.uri().as_str()));
}
