//! Preset RPC endpoint configurations for popular chains.

use crate::endpoint::RpcEndpoint;

/// Chain IDs for common networks.
pub mod chain_id {
    pub const ETHEREUM: u64 = 1;
    pub const GOERLI: u64 = 5;
    pub const SEPOLIA: u64 = 11155111;
    pub const ARBITRUM_ONE: u64 = 42161;
    pub const ARBITRUM_SEPOLIA: u64 = 421614;
    pub const BASE: u64 = 8453;
    pub const BASE_SEPOLIA: u64 = 84532;
    pub const OPTIMISM: u64 = 10;
    pub const POLYGON: u64 = 137;
    pub const BSC: u64 = 56;
    pub const AVALANCHE: u64 = 43114;
}

/// Get default endpoints for a chain by chain ID.
pub fn default_endpoints(chain_id: u64) -> Vec<RpcEndpoint> {
    match chain_id {
        chain_id::ARBITRUM_ONE => arbitrum_endpoints(),
        chain_id::BASE => base_endpoints(),
        chain_id::ETHEREUM => ethereum_endpoints(),
        chain_id::OPTIMISM => optimism_endpoints(),
        chain_id::POLYGON => polygon_endpoints(),
        _ => vec![],
    }
}

/// Default endpoints for Arbitrum One.
pub fn arbitrum_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://arb1.arbitrum.io/rpc")
            .with_name("Arbitrum Official")
            .with_ws_url("wss://arb1.arbitrum.io/rpc")
            .with_priority(50)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://arbitrum.llamarpc.com")
            .with_name("Llama RPC")
            .with_priority(60)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://rpc.ankr.com/arbitrum")
            .with_name("Ankr")
            .with_priority(70)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://arbitrum.drpc.org")
            .with_name("dRPC")
            .with_priority(80)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://1rpc.io/arb")
            .with_name("1RPC")
            .with_priority(90)
            .with_chain_id(chain_id::ARBITRUM_ONE),
    ]
}

/// Default endpoints for Base.
pub fn base_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://mainnet.base.org")
            .with_name("Base Official")
            .with_ws_url("wss://mainnet.base.org")
            .with_priority(50)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://base.llamarpc.com")
            .with_name("Llama RPC")
            .with_priority(60)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://rpc.ankr.com/base")
            .with_name("Ankr")
            .with_priority(70)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://base.drpc.org")
            .with_name("dRPC")
            .with_priority(80)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://1rpc.io/base")
            .with_name("1RPC")
            .with_priority(90)
            .with_chain_id(chain_id::BASE),
    ]
}

/// Default endpoints for Ethereum Mainnet.
pub fn ethereum_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://eth.llamarpc.com")
            .with_name("Llama RPC")
            .with_priority(50)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://rpc.ankr.com/eth")
            .with_name("Ankr")
            .with_priority(60)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://ethereum.drpc.org")
            .with_name("dRPC")
            .with_priority(70)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://1rpc.io/eth")
            .with_name("1RPC")
            .with_priority(80)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://cloudflare-eth.com")
            .with_name("Cloudflare")
            .with_priority(90)
            .with_chain_id(chain_id::ETHEREUM),
    ]
}

/// Default endpoints for Optimism.
pub fn optimism_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://mainnet.optimism.io")
            .with_name("Optimism Official")
            .with_priority(50)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://optimism.llamarpc.com")
            .with_name("Llama RPC")
            .with_priority(60)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://rpc.ankr.com/optimism")
            .with_name("Ankr")
            .with_priority(70)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://1rpc.io/op")
            .with_name("1RPC")
            .with_priority(80)
            .with_chain_id(chain_id::OPTIMISM),
    ]
}

/// Default endpoints for Polygon.
pub fn polygon_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://polygon-rpc.com")
            .with_name("Polygon Official")
            .with_priority(50)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://polygon.llamarpc.com")
            .with_name("Llama RPC")
            .with_priority(60)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://rpc.ankr.com/polygon")
            .with_name("Ankr")
            .with_priority(70)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://1rpc.io/matic")
            .with_name("1RPC")
            .with_priority(80)
            .with_chain_id(chain_id::POLYGON),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arbitrum_endpoints() {
        let endpoints = arbitrum_endpoints();
        assert!(!endpoints.is_empty());
        assert!(endpoints.iter().all(|e| e.chain_id == chain_id::ARBITRUM_ONE));
        // Should be sorted by priority
        for i in 1..endpoints.len() {
            assert!(endpoints[i - 1].priority <= endpoints[i].priority);
        }
    }

    #[test]
    fn test_default_endpoints() {
        assert!(!default_endpoints(chain_id::ARBITRUM_ONE).is_empty());
        assert!(!default_endpoints(chain_id::BASE).is_empty());
        assert!(!default_endpoints(chain_id::ETHEREUM).is_empty());
        assert!(default_endpoints(99999).is_empty()); // Unknown chain
    }
}
