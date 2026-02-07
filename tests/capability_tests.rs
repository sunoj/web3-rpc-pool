//! Integration tests for the capability and grading system.

use web3_rpc_pool::presets::{self, chain_id};
use web3_rpc_pool::{EndpointCapabilities, EndpointGrade, RpcEndpoint};

#[test]
fn test_all_presets_have_accessible_capabilities() {
    for &cid in &presets::all_chain_ids() {
        let endpoints = presets::default_endpoints(cid);
        for ep in &endpoints {
            // Capabilities should be accessible (default is fine)
            let _caps = &ep.capabilities;
            let _grade = ep.capabilities.grade();
        }
    }
}

#[test]
fn test_all_endpoints_have_correct_chain_id() {
    for &cid in &presets::all_chain_ids() {
        let endpoints = presets::default_endpoints(cid);
        for ep in &endpoints {
            assert_eq!(
                ep.chain_id, cid,
                "Endpoint '{}' has wrong chain_id: expected {}, got {}",
                ep.name, cid, ep.chain_id
            );
        }
    }
}

#[test]
fn test_grading_system_ordering() {
    assert!(EndpointGrade::F < EndpointGrade::D);
    assert!(EndpointGrade::D < EndpointGrade::C);
    assert!(EndpointGrade::C < EndpointGrade::B);
    assert!(EndpointGrade::B < EndpointGrade::A);
}

#[test]
fn test_grade_a_criteria() {
    let caps = EndpointCapabilities {
        supports_eth_get_logs: Some(true),
        max_batch_size: Some(100),
        max_block_range: Some(10_000),
        ..Default::default()
    };
    assert_eq!(caps.grade(), EndpointGrade::A);

    // Unlimited should also be A
    let caps_unlimited = EndpointCapabilities {
        supports_eth_get_logs: Some(true),
        max_batch_size: Some(0),
        max_block_range: Some(0),
        ..Default::default()
    };
    assert_eq!(caps_unlimited.grade(), EndpointGrade::A);
}

#[test]
fn test_grade_b_criteria() {
    let caps = EndpointCapabilities {
        supports_eth_get_logs: Some(true),
        max_batch_size: Some(50),
        max_block_range: Some(5_000),
        ..Default::default()
    };
    assert_eq!(caps.grade(), EndpointGrade::B);
}

#[test]
fn test_grade_c_criteria() {
    let caps = EndpointCapabilities {
        supports_eth_get_logs: Some(true),
        max_batch_size: Some(5),
        max_block_range: Some(100),
        ..Default::default()
    };
    assert_eq!(caps.grade(), EndpointGrade::C);
}

#[test]
fn test_grade_d_no_logs() {
    let caps = EndpointCapabilities {
        supports_eth_get_logs: Some(false),
        max_batch_size: Some(1000),
        max_block_range: Some(100_000),
        ..Default::default()
    };
    assert_eq!(caps.grade(), EndpointGrade::D);
}

#[test]
fn test_grade_d_unknown() {
    let caps = EndpointCapabilities::default();
    assert_eq!(caps.grade(), EndpointGrade::D);
}

#[test]
fn test_serialization_roundtrip() {
    let caps = EndpointCapabilities {
        supports_eth_get_logs: Some(true),
        max_batch_size: Some(100),
        max_block_range: Some(10_000),
        supports_debug_trace: Some(false),
        supports_websocket: true,
        rate_limit_rps: Some(25),
    };
    let endpoint = RpcEndpoint::new("https://rpc.example.com")
        .with_name("Test")
        .with_chain_id(1)
        .with_capabilities(caps);

    let json = serde_json::to_string(&endpoint).unwrap();
    let deserialized: RpcEndpoint = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.url, "https://rpc.example.com");
    assert_eq!(deserialized.name, "Test");
    assert_eq!(deserialized.chain_id, 1);
    assert_eq!(deserialized.capabilities.grade(), EndpointGrade::A);
    assert!(deserialized.capabilities.supports_websocket);
    assert_eq!(deserialized.capabilities.rate_limit_rps, Some(25));
    assert_eq!(deserialized.capabilities.supports_debug_trace, Some(false));
}

#[test]
fn test_backward_compat_deserialization() {
    // Old JSON format without capabilities field
    let json = r#"{
        "url": "https://rpc.example.com",
        "name": "OldEndpoint",
        "priority": 50,
        "chain_id": 1
    }"#;

    let endpoint: RpcEndpoint = serde_json::from_str(json).unwrap();
    assert_eq!(endpoint.url, "https://rpc.example.com");
    assert_eq!(endpoint.name, "OldEndpoint");
    assert_eq!(endpoint.priority, 50);
    assert_eq!(endpoint.chain_id, 1);
    // Capabilities should be default
    assert_eq!(endpoint.capabilities.grade(), EndpointGrade::D);
    assert!(!endpoint.capabilities.supports_websocket);
    assert!(endpoint.capabilities.supports_eth_get_logs.is_none());
}

#[test]
fn test_total_endpoint_count_at_least_150() {
    let total: usize = presets::all_chain_ids()
        .iter()
        .map(|&id| presets::default_endpoints(id).len())
        .sum();

    assert!(
        total >= 150,
        "Should have at least 150 total endpoints across all chains, got {}",
        total
    );
}

#[test]
fn test_all_chain_ids_complete() {
    let ids = presets::all_chain_ids();

    // All main chains should be present
    assert!(ids.contains(&chain_id::ETHEREUM));
    assert!(ids.contains(&chain_id::ARBITRUM_ONE));
    assert!(ids.contains(&chain_id::AVALANCHE));
    assert!(ids.contains(&chain_id::BASE));
    assert!(ids.contains(&chain_id::BSC));
    assert!(ids.contains(&chain_id::FANTOM));
    assert!(ids.contains(&chain_id::HYPERLIQUID_EVM));
    assert!(ids.contains(&chain_id::LINEA));
    assert!(ids.contains(&chain_id::OPTIMISM));
    assert!(ids.contains(&chain_id::POLYGON));
    assert!(ids.contains(&chain_id::ZKSYNC_ERA));
}

#[test]
fn test_chain_names_not_unknown() {
    for &cid in &presets::all_chain_ids() {
        let name = presets::chain_name(cid);
        assert_ne!(
            name, "Unknown",
            "Chain ID {} should have a known name",
            cid
        );
    }
}

#[test]
fn test_priority_adjustment_values() {
    // Grade A: -20
    let a = EndpointCapabilities {
        supports_eth_get_logs: Some(true),
        max_batch_size: Some(100),
        max_block_range: Some(10_000),
        ..Default::default()
    };
    assert_eq!(a.priority_adjustment(), -20);

    // Grade B: -10
    let b = EndpointCapabilities {
        supports_eth_get_logs: Some(true),
        max_batch_size: Some(50),
        max_block_range: Some(5_000),
        ..Default::default()
    };
    assert_eq!(b.priority_adjustment(), -10);

    // Grade C: 0
    let c = EndpointCapabilities {
        supports_eth_get_logs: Some(true),
        max_batch_size: Some(5),
        max_block_range: Some(100),
        ..Default::default()
    };
    assert_eq!(c.priority_adjustment(), 0);

    // Grade D (tested): +10
    let d = EndpointCapabilities {
        supports_eth_get_logs: Some(false),
        ..Default::default()
    };
    assert_eq!(d.priority_adjustment(), 10);

    // Grade D (unknown): 0
    let unknown = EndpointCapabilities::default();
    assert_eq!(unknown.priority_adjustment(), 0);
}

#[test]
fn test_ws_url_sets_supports_websocket() {
    let endpoint = RpcEndpoint::new("https://rpc.example.com")
        .with_ws_url("wss://rpc.example.com");
    assert!(endpoint.capabilities.supports_websocket);
}

#[test]
fn test_no_ws_url_does_not_set_websocket() {
    let endpoint = RpcEndpoint::new("https://rpc.example.com");
    assert!(!endpoint.capabilities.supports_websocket);
}

#[test]
fn test_endpoints_sorted_by_priority() {
    for &cid in &presets::all_chain_ids() {
        let endpoints = presets::default_endpoints(cid);
        for i in 1..endpoints.len() {
            assert!(
                endpoints[i - 1].priority <= endpoints[i].priority,
                "Chain {} ({}): endpoints not sorted by priority at index {} ({} > {})",
                presets::chain_name(cid),
                cid,
                i,
                endpoints[i - 1].priority,
                endpoints[i].priority
            );
        }
    }
}

#[test]
fn test_no_duplicate_urls_per_chain() {
    for &cid in &presets::all_chain_ids() {
        let endpoints = presets::default_endpoints(cid);
        let mut seen = std::collections::HashSet::new();
        for ep in &endpoints {
            assert!(
                seen.insert(&ep.url),
                "Chain {} ({}): duplicate URL found: {}",
                presets::chain_name(cid),
                cid,
                ep.url
            );
        }
    }
}
