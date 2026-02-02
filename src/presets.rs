//! Preset RPC endpoint configurations for popular chains.
//!
//! All endpoints have been verified to be working as of 2026-02.
//! Endpoints are tested with eth_blockNumber RPC call.

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

/// Default endpoints for Ethereum Mainnet (22 verified endpoints).
pub fn ethereum_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://ethereum-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://ethereum-rpc.publicnode.com")
            .with_priority(50)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://1rpc.io/eth")
            .with_name("1RPC")
            .with_priority(51)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://eth.drpc.org")
            .with_name("dRPC")
            .with_priority(52)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://eth-mainnet.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(53)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://eth.merkle.io")
            .with_name("Merkle")
            .with_priority(54)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://rpc.flashbots.net")
            .with_name("Flashbots")
            .with_priority(55)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://eth.api.onfinality.io/public")
            .with_name("OnFinality")
            .with_priority(56)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://eth.meowrpc.com")
            .with_name("MeowRPC")
            .with_priority(57)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://eth-pokt.nodies.app")
            .with_name("Nodies")
            .with_priority(58)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://rpc.mevblocker.io")
            .with_name("MEV Blocker")
            .with_priority(59)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://ethereum.rpc.subquery.network/public")
            .with_name("SubQuery")
            .with_priority(60)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://0xrpc.io/eth")
            .with_name("0xRPC")
            .with_priority(61)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://eth.blockrazor.xyz")
            .with_name("BlockRazor")
            .with_priority(62)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://endpoints.omniatech.io/v1/eth/mainnet/public")
            .with_name("OmniaTech")
            .with_priority(63)
            .with_chain_id(chain_id::ETHEREUM),
        // Additional verified endpoints
        RpcEndpoint::new("https://eth.rpc.blxrbdn.com")
            .with_name("BloXroute")
            .with_priority(64)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://rpc.eth.gateway.fm")
            .with_name("Gateway.fm")
            .with_priority(65)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://core.gashawk.io/rpc")
            .with_name("GasHawk")
            .with_priority(66)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://rpc.tornadoeth.cash/eth")
            .with_name("TornadoETH")
            .with_priority(67)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://mainnet.gateway.tenderly.co")
            .with_name("Tenderly")
            .with_priority(68)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://rpc.mevblocker.io/fast")
            .with_name("MEV Blocker Fast")
            .with_priority(69)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://rpc.mevblocker.io/noreverts")
            .with_name("MEV Blocker NoReverts")
            .with_priority(70)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://rpc.mevblocker.io/fullprivacy")
            .with_name("MEV Blocker FullPrivacy")
            .with_priority(71)
            .with_chain_id(chain_id::ETHEREUM),
    ]
}

/// Default endpoints for Arbitrum One (10 verified endpoints).
pub fn arbitrum_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://arb1.arbitrum.io/rpc")
            .with_name("Arbitrum Official")
            .with_ws_url("wss://arb1.arbitrum.io/rpc")
            .with_priority(50)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://arbitrum-one-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://arbitrum-one-rpc.publicnode.com")
            .with_priority(51)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://1rpc.io/arb")
            .with_name("1RPC")
            .with_priority(52)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://arbitrum.drpc.org")
            .with_name("dRPC")
            .with_priority(53)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://arbitrum-one.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(54)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://arbitrum.api.onfinality.io/public")
            .with_name("OnFinality")
            .with_priority(55)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://arbitrum.meowrpc.com")
            .with_name("MeowRPC")
            .with_priority(56)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://arb-pokt.nodies.app")
            .with_name("Nodies")
            .with_priority(57)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://arbitrum.gateway.tenderly.co")
            .with_name("Tenderly")
            .with_priority(58)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://arbitrum.rpc.subquery.network/public")
            .with_name("SubQuery")
            .with_priority(59)
            .with_chain_id(chain_id::ARBITRUM_ONE),
    ]
}

/// Default endpoints for Base (11 verified endpoints).
pub fn base_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://mainnet.base.org")
            .with_name("Base Official")
            .with_ws_url("wss://mainnet.base.org")
            .with_priority(50)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://base-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://base-rpc.publicnode.com")
            .with_priority(51)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://1rpc.io/base")
            .with_name("1RPC")
            .with_priority(52)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://base.drpc.org")
            .with_name("dRPC")
            .with_priority(53)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://base-mainnet.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(54)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://base.meowrpc.com")
            .with_name("MeowRPC")
            .with_priority(55)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://base.gateway.tenderly.co")
            .with_name("Tenderly")
            .with_priority(56)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://base-pokt.nodies.app")
            .with_name("Nodies")
            .with_priority(57)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://developer-access-mainnet.base.org")
            .with_name("Base Developer")
            .with_priority(58)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://base.rpc.subquery.network/public")
            .with_name("SubQuery")
            .with_priority(59)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://endpoints.omniatech.io/v1/base/mainnet/public")
            .with_name("OmniaTech")
            .with_priority(60)
            .with_chain_id(chain_id::BASE),
    ]
}

/// Default endpoints for Optimism (8 verified endpoints).
pub fn optimism_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://mainnet.optimism.io")
            .with_name("Optimism Official")
            .with_priority(50)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://optimism-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://optimism-rpc.publicnode.com")
            .with_priority(51)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://1rpc.io/op")
            .with_name("1RPC")
            .with_priority(52)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://optimism.drpc.org")
            .with_name("dRPC")
            .with_priority(53)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://optimism.api.onfinality.io/public")
            .with_name("OnFinality")
            .with_priority(54)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://op-pokt.nodies.app")
            .with_name("Nodies")
            .with_priority(55)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://optimism.gateway.tenderly.co")
            .with_name("Tenderly")
            .with_priority(56)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://endpoints.omniatech.io/v1/op/mainnet/public")
            .with_name("OmniaTech")
            .with_priority(57)
            .with_chain_id(chain_id::OPTIMISM),
    ]
}

/// Default endpoints for BSC (22 verified endpoints).
pub fn bsc_endpoints() -> Vec<RpcEndpoint> {
    vec![
        // Official BNB Chain endpoints
        RpcEndpoint::new("https://bsc-dataseed.bnbchain.org")
            .with_name("BNB Chain Official")
            .with_priority(50)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-dataseed1.bnbchain.org")
            .with_name("BNB Chain 1")
            .with_priority(51)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-dataseed2.bnbchain.org")
            .with_name("BNB Chain 2")
            .with_priority(52)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-dataseed3.bnbchain.org")
            .with_name("BNB Chain 3")
            .with_priority(53)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-dataseed4.bnbchain.org")
            .with_name("BNB Chain 4")
            .with_priority(54)
            .with_chain_id(chain_id::BSC),
        // Third-party verified endpoints
        RpcEndpoint::new("https://bsc-dataseed1.defibit.io")
            .with_name("Defibit 1")
            .with_priority(55)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-dataseed2.defibit.io")
            .with_name("Defibit 2")
            .with_priority(56)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-dataseed3.defibit.io")
            .with_name("Defibit 3")
            .with_priority(57)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-dataseed4.defibit.io")
            .with_name("Defibit 4")
            .with_priority(58)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-dataseed1.ninicoin.io")
            .with_name("Ninicoin 1")
            .with_priority(59)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-dataseed2.ninicoin.io")
            .with_name("Ninicoin 2")
            .with_priority(60)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-dataseed3.ninicoin.io")
            .with_name("Ninicoin 3")
            .with_priority(61)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-dataseed4.ninicoin.io")
            .with_name("Ninicoin 4")
            .with_priority(62)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://bsc-rpc.publicnode.com")
            .with_priority(63)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc.publicnode.com")
            .with_name("PublicNode Alt")
            .with_priority(64)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://1rpc.io/bnb")
            .with_name("1RPC")
            .with_priority(65)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc.drpc.org")
            .with_name("dRPC")
            .with_priority(66)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-mainnet.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(67)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc.meowrpc.com")
            .with_name("MeowRPC")
            .with_priority(68)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-pokt.nodies.app")
            .with_name("Nodies")
            .with_priority(69)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://endpoints.omniatech.io/v1/bsc/mainnet/public")
            .with_name("OmniaTech")
            .with_priority(70)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bnb.rpc.subquery.network/public")
            .with_name("SubQuery")
            .with_priority(71)
            .with_chain_id(chain_id::BSC),
    ]
}

/// Default endpoints for Avalanche C-Chain (8 verified endpoints).
pub fn avalanche_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://api.avax.network/ext/bc/C/rpc")
            .with_name("Avalanche Official")
            .with_priority(50)
            .with_chain_id(chain_id::AVALANCHE),
        RpcEndpoint::new("https://avalanche-c-chain-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://avalanche-c-chain-rpc.publicnode.com")
            .with_priority(51)
            .with_chain_id(chain_id::AVALANCHE),
        RpcEndpoint::new("https://1rpc.io/avax/c")
            .with_name("1RPC")
            .with_priority(52)
            .with_chain_id(chain_id::AVALANCHE),
        RpcEndpoint::new("https://avalanche.drpc.org")
            .with_name("dRPC")
            .with_priority(53)
            .with_chain_id(chain_id::AVALANCHE),
        RpcEndpoint::new("https://avalanche.api.onfinality.io/public/ext/bc/C/rpc")
            .with_name("OnFinality")
            .with_priority(54)
            .with_chain_id(chain_id::AVALANCHE),
        RpcEndpoint::new("https://avax-pokt.nodies.app/ext/bc/C/rpc")
            .with_name("Nodies")
            .with_priority(55)
            .with_chain_id(chain_id::AVALANCHE),
        RpcEndpoint::new("https://avalanche.gateway.tenderly.co")
            .with_name("Tenderly")
            .with_priority(56)
            .with_chain_id(chain_id::AVALANCHE),
        RpcEndpoint::new("https://endpoints.omniatech.io/v1/avax/mainnet/public")
            .with_name("OmniaTech")
            .with_priority(57)
            .with_chain_id(chain_id::AVALANCHE),
    ]
}

/// Default endpoints for Polygon (10 verified endpoints).
pub fn polygon_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://polygon-rpc.com")
            .with_name("Polygon Official")
            .with_priority(50)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://polygon-bor-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://polygon-bor-rpc.publicnode.com")
            .with_priority(51)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://1rpc.io/matic")
            .with_name("1RPC")
            .with_priority(52)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://polygon.drpc.org")
            .with_name("dRPC")
            .with_priority(53)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://polygon.api.onfinality.io/public")
            .with_name("OnFinality")
            .with_priority(54)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://polygon.gateway.tenderly.co")
            .with_name("Tenderly")
            .with_priority(55)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://polygon.rpc.subquery.network/public")
            .with_name("SubQuery")
            .with_priority(56)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://endpoints.omniatech.io/v1/matic/mainnet/public")
            .with_name("OmniaTech")
            .with_priority(57)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://polygon-pokt.nodies.app")
            .with_name("Nodies")
            .with_priority(58)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://rpc-mainnet.matic.quiknode.pro")
            .with_name("QuickNode")
            .with_priority(59)
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
        assert!(endpoints.len() >= 20, "Should have at least 20 endpoints");
    }

    #[test]
    fn test_arbitrum_endpoints() {
        let endpoints = arbitrum_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::ARBITRUM_ONE);
        assert!(endpoints.len() >= 8, "Should have at least 8 endpoints");
    }

    #[test]
    fn test_optimism_endpoints() {
        let endpoints = optimism_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::OPTIMISM);
        assert!(endpoints.len() >= 6, "Should have at least 6 endpoints");
    }

    #[test]
    fn test_base_endpoints() {
        let endpoints = base_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::BASE);
        assert!(endpoints.len() >= 8, "Should have at least 8 endpoints");
    }

    #[test]
    fn test_polygon_endpoints() {
        let endpoints = polygon_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::POLYGON);
        assert!(endpoints.len() >= 8, "Should have at least 8 endpoints");
    }

    #[test]
    fn test_bsc_endpoints() {
        let endpoints = bsc_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::BSC);
        assert!(endpoints.len() >= 15, "Should have at least 15 endpoints");
    }

    #[test]
    fn test_avalanche_endpoints() {
        let endpoints = avalanche_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::AVALANCHE);
        assert!(endpoints.len() >= 6, "Should have at least 6 endpoints");
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
    fn test_total_endpoint_count() {
        let total = ethereum_endpoints().len()
            + arbitrum_endpoints().len()
            + optimism_endpoints().len()
            + base_endpoints().len()
            + polygon_endpoints().len()
            + bsc_endpoints().len()
            + avalanche_endpoints().len();

        assert!(total >= 70, "Should have at least 70 total endpoints, got {}", total);
    }
}
