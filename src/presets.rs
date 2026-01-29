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
        chain_id::AVALANCHE => avalanche_endpoints(),
        chain_id::BASE => base_endpoints(),
        chain_id::BSC => bsc_endpoints(),
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
            .with_priority(55)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://rpc.ankr.com/arbitrum")
            .with_name("Ankr")
            .with_priority(60)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://arbitrum-one-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://arbitrum-one-rpc.publicnode.com")
            .with_priority(65)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://arbitrum.drpc.org")
            .with_name("dRPC")
            .with_priority(70)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://1rpc.io/arb")
            .with_name("1RPC")
            .with_priority(75)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://arbitrum-one.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(80)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://arbitrum.api.onfinality.io/public")
            .with_name("OnFinality")
            .with_priority(85)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://arb-mainnet.g.alchemy.com/v2/demo")
            .with_name("Alchemy Demo")
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
            .with_priority(55)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://rpc.ankr.com/base")
            .with_name("Ankr")
            .with_priority(60)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://base-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://base-rpc.publicnode.com")
            .with_priority(65)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://base.drpc.org")
            .with_name("dRPC")
            .with_priority(70)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://1rpc.io/base")
            .with_name("1RPC")
            .with_priority(75)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://base-mainnet.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(80)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://base.api.onfinality.io/public")
            .with_name("OnFinality")
            .with_priority(85)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://base.meowrpc.com")
            .with_name("MeowRPC")
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
            .with_priority(55)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://ethereum-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://ethereum-rpc.publicnode.com")
            .with_priority(60)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://ethereum.drpc.org")
            .with_name("dRPC")
            .with_priority(65)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://1rpc.io/eth")
            .with_name("1RPC")
            .with_priority(70)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://cloudflare-eth.com")
            .with_name("Cloudflare")
            .with_priority(75)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://eth-mainnet.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(80)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://rpc.payload.de")
            .with_name("Payload")
            .with_priority(85)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://eth.merkle.io")
            .with_name("Merkle")
            .with_priority(90)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://rpc.flashbots.net")
            .with_name("Flashbots")
            .with_priority(95)
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
            .with_priority(55)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://rpc.ankr.com/optimism")
            .with_name("Ankr")
            .with_priority(60)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://optimism-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://optimism-rpc.publicnode.com")
            .with_priority(65)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://optimism.drpc.org")
            .with_name("dRPC")
            .with_priority(70)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://1rpc.io/op")
            .with_name("1RPC")
            .with_priority(75)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://optimism-mainnet.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(80)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://optimism.api.onfinality.io/public")
            .with_name("OnFinality")
            .with_priority(85)
            .with_chain_id(chain_id::OPTIMISM),
    ]
}

/// Default endpoints for BSC (Binance Smart Chain).
pub fn bsc_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://bsc-dataseed.bnbchain.org")
            .with_name("BNB Chain Official")
            .with_priority(50)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-dataseed1.defibit.io")
            .with_name("Defibit")
            .with_priority(55)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-dataseed1.ninicoin.io")
            .with_name("Ninicoin")
            .with_priority(60)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://binance.llamarpc.com")
            .with_name("Llama RPC")
            .with_priority(65)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://rpc.ankr.com/bsc")
            .with_name("Ankr")
            .with_priority(70)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://bsc-rpc.publicnode.com")
            .with_priority(75)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc.drpc.org")
            .with_name("dRPC")
            .with_priority(80)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://1rpc.io/bnb")
            .with_name("1RPC")
            .with_priority(85)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-mainnet.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(90)
            .with_chain_id(chain_id::BSC),
    ]
}

/// Default endpoints for Avalanche C-Chain.
pub fn avalanche_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://api.avax.network/ext/bc/C/rpc")
            .with_name("Avalanche Official")
            .with_priority(50)
            .with_chain_id(chain_id::AVALANCHE),
        RpcEndpoint::new("https://avalanche.llamarpc.com")
            .with_name("Llama RPC")
            .with_priority(55)
            .with_chain_id(chain_id::AVALANCHE),
        RpcEndpoint::new("https://rpc.ankr.com/avalanche")
            .with_name("Ankr")
            .with_priority(60)
            .with_chain_id(chain_id::AVALANCHE),
        RpcEndpoint::new("https://avalanche-c-chain-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://avalanche-c-chain-rpc.publicnode.com")
            .with_priority(65)
            .with_chain_id(chain_id::AVALANCHE),
        RpcEndpoint::new("https://avalanche.drpc.org")
            .with_name("dRPC")
            .with_priority(70)
            .with_chain_id(chain_id::AVALANCHE),
        RpcEndpoint::new("https://1rpc.io/avax/c")
            .with_name("1RPC")
            .with_priority(75)
            .with_chain_id(chain_id::AVALANCHE),
        RpcEndpoint::new("https://avalanche-mainnet.public.blastapi.io/ext/bc/C/rpc")
            .with_name("BlastAPI")
            .with_priority(80)
            .with_chain_id(chain_id::AVALANCHE),
        RpcEndpoint::new("https://avalanche.api.onfinality.io/public/ext/bc/C/rpc")
            .with_name("OnFinality")
            .with_priority(85)
            .with_chain_id(chain_id::AVALANCHE),
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
            .with_priority(55)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://rpc.ankr.com/polygon")
            .with_name("Ankr")
            .with_priority(60)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://polygon-bor-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://polygon-bor-rpc.publicnode.com")
            .with_priority(65)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://polygon.drpc.org")
            .with_name("dRPC")
            .with_priority(70)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://1rpc.io/matic")
            .with_name("1RPC")
            .with_priority(75)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://polygon-mainnet.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(80)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://rpc-mainnet.maticvigil.com")
            .with_name("MaticVigil")
            .with_priority(85)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://polygon.api.onfinality.io/public")
            .with_name("OnFinality")
            .with_priority(90)
            .with_chain_id(chain_id::POLYGON),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_valid_endpoints(endpoints: &[RpcEndpoint], expected_chain_id: u64) {
        assert!(!endpoints.is_empty(), "Endpoints should not be empty");
        assert!(
            endpoints.iter().all(|e| e.chain_id == expected_chain_id),
            "All endpoints should have correct chain ID"
        );
        // Should be sorted by priority
        for i in 1..endpoints.len() {
            assert!(
                endpoints[i - 1].priority <= endpoints[i].priority,
                "Endpoints should be sorted by priority"
            );
        }
    }

    #[test]
    fn test_ethereum_endpoints() {
        let endpoints = ethereum_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::ETHEREUM);
        assert!(endpoints.len() >= 5, "Should have multiple Ethereum endpoints");
    }

    #[test]
    fn test_arbitrum_endpoints() {
        let endpoints = arbitrum_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::ARBITRUM_ONE);
    }

    #[test]
    fn test_optimism_endpoints() {
        let endpoints = optimism_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::OPTIMISM);
    }

    #[test]
    fn test_base_endpoints() {
        let endpoints = base_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::BASE);
    }

    #[test]
    fn test_polygon_endpoints() {
        let endpoints = polygon_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::POLYGON);
    }

    #[test]
    fn test_bsc_endpoints() {
        let endpoints = bsc_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::BSC);
    }

    #[test]
    fn test_avalanche_endpoints() {
        let endpoints = avalanche_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::AVALANCHE);
    }

    #[test]
    fn test_default_endpoints() {
        assert!(!default_endpoints(chain_id::ARBITRUM_ONE).is_empty());
        assert!(!default_endpoints(chain_id::AVALANCHE).is_empty());
        assert!(!default_endpoints(chain_id::BASE).is_empty());
        assert!(!default_endpoints(chain_id::BSC).is_empty());
        assert!(!default_endpoints(chain_id::ETHEREUM).is_empty());
        assert!(!default_endpoints(chain_id::OPTIMISM).is_empty());
        assert!(!default_endpoints(chain_id::POLYGON).is_empty());
        assert!(default_endpoints(99999).is_empty()); // Unknown chain
    }

    #[test]
    fn test_endpoints_have_valid_urls() {
        let all_endpoints: Vec<Vec<RpcEndpoint>> = vec![
            ethereum_endpoints(),
            arbitrum_endpoints(),
            optimism_endpoints(),
            base_endpoints(),
            polygon_endpoints(),
            bsc_endpoints(),
            avalanche_endpoints(),
        ];

        for endpoints in all_endpoints {
            for endpoint in endpoints {
                assert!(
                    endpoint.url.starts_with("https://"),
                    "URL should use HTTPS: {}",
                    endpoint.url
                );
                assert!(
                    !endpoint.name.is_empty(),
                    "Endpoint should have a name"
                );
            }
        }
    }
}
