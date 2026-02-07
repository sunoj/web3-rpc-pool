//! Live endpoint reachability tests.
//!
//! These tests make actual network requests to verify endpoint availability.
//! Run with: `cargo test --features live-tests -- --test live_endpoint_tests`
//!
//! These tests are behind the `live-tests` feature flag to avoid running
//! in CI or during normal development.

#![cfg(feature = "live-tests")]

use web3_rpc_pool::presets;

async fn check_endpoint_reachable(url: &str) -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    });

    match client.post(url).json(&body).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

/// Test that at least one endpoint per chain is reachable.
#[tokio::test]
async fn test_at_least_one_endpoint_reachable_per_chain() {
    for &chain_id in &presets::all_chain_ids() {
        let endpoints = presets::default_endpoints(chain_id);
        let chain_name = presets::chain_name(chain_id);

        assert!(
            !endpoints.is_empty(),
            "Chain {} ({}) has no endpoints",
            chain_name,
            chain_id
        );

        let mut any_reachable = false;
        let mut tried = 0;

        // Try up to 3 endpoints per chain to avoid timeout
        for ep in endpoints.iter().take(3) {
            tried += 1;
            if check_endpoint_reachable(&ep.url).await {
                any_reachable = true;
                println!(
                    "  [OK] {} ({}): {} is reachable",
                    chain_name, chain_id, ep.name
                );
                break;
            } else {
                println!(
                    "  [FAIL] {} ({}): {} is not reachable",
                    chain_name, chain_id, ep.name
                );
            }
        }

        assert!(
            any_reachable,
            "Chain {} ({}) has no reachable endpoints (tried {})",
            chain_name, chain_id, tried
        );
    }
}
