//! Preset RPC endpoint configurations for popular chains.
//!
//! All endpoints have been verified to be working as of 2026-02.
//! Endpoints are tested with eth_blockNumber RPC call.

use crate::endpoint::{EndpointCapabilities, RpcEndpoint};

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
    pub const FANTOM: u64 = 250;
    pub const ZKSYNC_ERA: u64 = 324;
    pub const LINEA: u64 = 59144;
    pub const HYPERLIQUID_EVM: u64 = 999;
    pub const SCROLL: u64 = 534352;
    pub const POLYGON_ZKEVM: u64 = 1101;
    pub const BLAST: u64 = 81457;
    pub const MANTLE: u64 = 5000;
    pub const MODE: u64 = 34443;
    pub const MANTA_PACIFIC: u64 = 169;
}

/// Get default endpoints for a chain by chain ID.
pub fn default_endpoints(chain_id: u64) -> Vec<RpcEndpoint> {
    match chain_id {
        chain_id::ARBITRUM_ONE => arbitrum_endpoints(),
        chain_id::AVALANCHE => avalanche_endpoints(),
        chain_id::BASE => base_endpoints(),
        chain_id::BLAST => blast_endpoints(),
        chain_id::BSC => bsc_endpoints(),
        chain_id::ETHEREUM => ethereum_endpoints(),
        chain_id::FANTOM => fantom_endpoints(),
        chain_id::HYPERLIQUID_EVM => hyperliquid_evm_endpoints(),
        chain_id::LINEA => linea_endpoints(),
        chain_id::MANTA_PACIFIC => manta_pacific_endpoints(),
        chain_id::MANTLE => mantle_endpoints(),
        chain_id::MODE => mode_endpoints(),
        chain_id::OPTIMISM => optimism_endpoints(),
        chain_id::POLYGON => polygon_endpoints(),
        chain_id::POLYGON_ZKEVM => polygon_zkevm_endpoints(),
        chain_id::SCROLL => scroll_endpoints(),
        chain_id::ZKSYNC_ERA => zksync_era_endpoints(),
        _ => vec![],
    }
}

/// Return all supported mainnet chain IDs.
pub fn all_chain_ids() -> Vec<u64> {
    vec![
        chain_id::ETHEREUM,
        chain_id::ARBITRUM_ONE,
        chain_id::AVALANCHE,
        chain_id::BASE,
        chain_id::BLAST,
        chain_id::BSC,
        chain_id::FANTOM,
        chain_id::HYPERLIQUID_EVM,
        chain_id::LINEA,
        chain_id::MANTA_PACIFIC,
        chain_id::MANTLE,
        chain_id::MODE,
        chain_id::OPTIMISM,
        chain_id::POLYGON,
        chain_id::POLYGON_ZKEVM,
        chain_id::SCROLL,
        chain_id::ZKSYNC_ERA,
    ]
}

/// Return a human-readable name for a chain ID.
pub fn chain_name(chain_id: u64) -> &'static str {
    match chain_id {
        self::chain_id::ETHEREUM => "Ethereum",
        self::chain_id::GOERLI => "Goerli",
        self::chain_id::SEPOLIA => "Sepolia",
        self::chain_id::ARBITRUM_ONE => "Arbitrum One",
        self::chain_id::ARBITRUM_SEPOLIA => "Arbitrum Sepolia",
        self::chain_id::AVALANCHE => "Avalanche C-Chain",
        self::chain_id::BASE => "Base",
        self::chain_id::BASE_SEPOLIA => "Base Sepolia",
        self::chain_id::BLAST => "Blast",
        self::chain_id::BSC => "BNB Smart Chain",
        self::chain_id::FANTOM => "Fantom Opera",
        self::chain_id::HYPERLIQUID_EVM => "Hyperliquid EVM",
        self::chain_id::LINEA => "Linea",
        self::chain_id::MANTA_PACIFIC => "Manta Pacific",
        self::chain_id::MANTLE => "Mantle",
        self::chain_id::MODE => "Mode",
        self::chain_id::OPTIMISM => "Optimism",
        self::chain_id::POLYGON => "Polygon",
        self::chain_id::POLYGON_ZKEVM => "Polygon zkEVM",
        self::chain_id::SCROLL => "Scroll",
        self::chain_id::ZKSYNC_ERA => "zkSync Era",
        _ => "Unknown",
    }
}

/// Default endpoints for Ethereum Mainnet (25 verified endpoints).
pub fn ethereum_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://ethereum-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://ethereum-rpc.publicnode.com")
            .with_priority(50)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(100), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://1rpc.io/eth")
            .with_name("1RPC")
            .with_priority(51)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://eth.drpc.org")
            .with_name("dRPC")
            .with_priority(52)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://eth-mainnet.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(53)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://eth.merkle.io")
            .with_name("Merkle")
            .with_priority(54)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://rpc.flashbots.net")
            .with_name("Flashbots")
            .with_priority(55)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://eth.api.onfinality.io/public")
            .with_name("OnFinality")
            .with_priority(56)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://eth.meowrpc.com")
            .with_name("MeowRPC")
            .with_priority(57)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(1), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://eth-pokt.nodies.app")
            .with_name("Nodies")
            .with_priority(58)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(50), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://rpc.mevblocker.io")
            .with_name("MEV Blocker")
            .with_priority(59)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://ethereum.rpc.subquery.network/public")
            .with_name("SubQuery")
            .with_priority(60)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://0xrpc.io/eth")
            .with_name("0xRPC")
            .with_priority(61)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://eth.blockrazor.xyz")
            .with_name("BlockRazor")
            .with_priority(62)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://endpoints.omniatech.io/v1/eth/mainnet/public")
            .with_name("OmniaTech")
            .with_priority(63)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://eth.rpc.blxrbdn.com")
            .with_name("BloXroute")
            .with_priority(64)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://rpc.eth.gateway.fm")
            .with_name("Gateway.fm")
            .with_priority(65)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://core.gashawk.io/rpc")
            .with_name("GasHawk")
            .with_priority(66)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://rpc.tornadoeth.cash/eth")
            .with_name("TornadoETH")
            .with_priority(67)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://mainnet.gateway.tenderly.co")
            .with_name("Tenderly")
            .with_priority(68)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://rpc.mevblocker.io/fast")
            .with_name("MEV Blocker Fast")
            .with_priority(69)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://rpc.mevblocker.io/noreverts")
            .with_name("MEV Blocker NoReverts")
            .with_priority(70)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://rpc.mevblocker.io/fullprivacy")
            .with_name("MEV Blocker FullPrivacy")
            .with_priority(71)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        // Phase 3: additional endpoints
        RpcEndpoint::new("https://rpc.ankr.com/eth")
            .with_name("Ankr")
            .with_priority(72)
            .with_chain_id(chain_id::ETHEREUM),
        RpcEndpoint::new("https://eth.llamarpc.com")
            .with_name("LlamaNodes")
            .with_priority(73)
            .with_chain_id(chain_id::ETHEREUM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://cloudflare-eth.com")
            .with_name("Cloudflare")
            .with_priority(74)
            .with_chain_id(chain_id::ETHEREUM),
    ]
}

/// Default endpoints for Arbitrum One (21 verified endpoints).
pub fn arbitrum_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://arb1.arbitrum.io/rpc")
            .with_name("Arbitrum Official")
            .with_ws_url("wss://arb1.arbitrum.io/rpc")
            .with_priority(50)
            .with_chain_id(chain_id::ARBITRUM_ONE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(100), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://arbitrum-one-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://arbitrum-one-rpc.publicnode.com")
            .with_priority(51)
            .with_chain_id(chain_id::ARBITRUM_ONE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://1rpc.io/arb")
            .with_name("1RPC")
            .with_priority(52)
            .with_chain_id(chain_id::ARBITRUM_ONE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://arbitrum.drpc.org")
            .with_name("dRPC")
            .with_priority(53)
            .with_chain_id(chain_id::ARBITRUM_ONE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://arbitrum-one.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(54)
            .with_chain_id(chain_id::ARBITRUM_ONE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(10000), ..Default::default() }),
        RpcEndpoint::new("https://arbitrum.api.onfinality.io/public")
            .with_name("OnFinality")
            .with_priority(55)
            .with_chain_id(chain_id::ARBITRUM_ONE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://arbitrum.meowrpc.com")
            .with_name("MeowRPC")
            .with_priority(56)
            .with_chain_id(chain_id::ARBITRUM_ONE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(1), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://arb-pokt.nodies.app")
            .with_name("Nodies")
            .with_priority(57)
            .with_chain_id(chain_id::ARBITRUM_ONE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(50), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://arbitrum.gateway.tenderly.co")
            .with_name("Tenderly")
            .with_priority(58)
            .with_chain_id(chain_id::ARBITRUM_ONE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://arbitrum.rpc.subquery.network/public")
            .with_name("SubQuery")
            .with_priority(59)
            .with_chain_id(chain_id::ARBITRUM_ONE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(50), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://arbitrum-one-public.nodies.app")
            .with_name("Nodies Public")
            .with_priority(60)
            .with_chain_id(chain_id::ARBITRUM_ONE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://arbitrum.public.blockpi.network/v1/rpc/public")
            .with_name("BlockPI")
            .with_priority(61)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://api.zan.top/arb-one")
            .with_name("ZAN")
            .with_priority(62)
            .with_chain_id(chain_id::ARBITRUM_ONE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(500), max_block_range: Some(10000), ..Default::default() }),
        RpcEndpoint::new("https://arb1.lava.build")
            .with_name("Lava")
            .with_priority(63)
            .with_chain_id(chain_id::ARBITRUM_ONE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://arb-one-mainnet.gateway.tatum.io")
            .with_name("Tatum")
            .with_priority(64)
            .with_chain_id(chain_id::ARBITRUM_ONE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://public-arb-mainnet.fastnode.io")
            .with_name("FastNode")
            .with_priority(65)
            .with_chain_id(chain_id::ARBITRUM_ONE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://arbitrum.rpc.thirdweb.com")
            .with_name("thirdweb")
            .with_priority(66)
            .with_chain_id(chain_id::ARBITRUM_ONE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://arb-one.api.pocket.network")
            .with_name("Pocket Network")
            .with_priority(67)
            .with_chain_id(chain_id::ARBITRUM_ONE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://arb.leorpc.com/?api_key=FREE")
            .with_name("LeoRPC")
            .with_priority(68)
            .with_chain_id(chain_id::ARBITRUM_ONE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(100), ..Default::default() }),
        // Phase 3: additional endpoints
        RpcEndpoint::new("https://rpc.ankr.com/arbitrum")
            .with_name("Ankr")
            .with_priority(69)
            .with_chain_id(chain_id::ARBITRUM_ONE),
        RpcEndpoint::new("https://0xrpc.io/arb")
            .with_name("0xRPC")
            .with_priority(70)
            .with_chain_id(chain_id::ARBITRUM_ONE),
    ]
}

/// Default endpoints for Base (17 verified endpoints).
pub fn base_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://mainnet.base.org")
            .with_name("Base Official")
            .with_ws_url("wss://mainnet.base.org")
            .with_priority(50)
            .with_chain_id(chain_id::BASE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://base-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://base-rpc.publicnode.com")
            .with_priority(51)
            .with_chain_id(chain_id::BASE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://1rpc.io/base")
            .with_name("1RPC")
            .with_priority(52)
            .with_chain_id(chain_id::BASE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://base.drpc.org")
            .with_name("dRPC")
            .with_priority(53)
            .with_chain_id(chain_id::BASE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://base-mainnet.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(54)
            .with_chain_id(chain_id::BASE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://base.meowrpc.com")
            .with_name("MeowRPC")
            .with_priority(55)
            .with_chain_id(chain_id::BASE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(1), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://base.gateway.tenderly.co")
            .with_name("Tenderly")
            .with_priority(56)
            .with_chain_id(chain_id::BASE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://base-pokt.nodies.app")
            .with_name("Nodies")
            .with_priority(57)
            .with_chain_id(chain_id::BASE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(50), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://developer-access-mainnet.base.org")
            .with_name("Base Developer")
            .with_priority(58)
            .with_chain_id(chain_id::BASE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://base.rpc.subquery.network/public")
            .with_name("SubQuery")
            .with_priority(59)
            .with_chain_id(chain_id::BASE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(10), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://endpoints.omniatech.io/v1/base/mainnet/public")
            .with_name("OmniaTech")
            .with_priority(60)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://base.public.blockpi.network/v1/rpc/public")
            .with_name("BlockPI")
            .with_priority(61)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://base-public.nodies.app")
            .with_name("Nodies Public")
            .with_priority(62)
            .with_chain_id(chain_id::BASE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://base.api.pocket.network")
            .with_name("Pocket")
            .with_priority(63)
            .with_chain_id(chain_id::BASE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://rpcbase.hairylabs.io/rpc")
            .with_name("HairyLabs")
            .with_priority(64)
            .with_chain_id(chain_id::BASE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        // Phase 3: additional endpoints
        RpcEndpoint::new("https://rpc.ankr.com/base")
            .with_name("Ankr")
            .with_priority(65)
            .with_chain_id(chain_id::BASE),
        RpcEndpoint::new("https://base.llamarpc.com")
            .with_name("LlamaNodes")
            .with_priority(66)
            .with_chain_id(chain_id::BASE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(0), ..Default::default() }),
    ]
}

/// Default endpoints for Optimism (16 verified endpoints).
pub fn optimism_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://mainnet.optimism.io")
            .with_name("Optimism Official")
            .with_priority(50)
            .with_chain_id(chain_id::OPTIMISM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://optimism-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://optimism-rpc.publicnode.com")
            .with_priority(51)
            .with_chain_id(chain_id::OPTIMISM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://1rpc.io/op")
            .with_name("1RPC")
            .with_priority(52)
            .with_chain_id(chain_id::OPTIMISM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://optimism.drpc.org")
            .with_name("dRPC")
            .with_priority(53)
            .with_chain_id(chain_id::OPTIMISM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://optimism.api.onfinality.io/public")
            .with_name("OnFinality")
            .with_priority(54)
            .with_chain_id(chain_id::OPTIMISM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(50), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://op-pokt.nodies.app")
            .with_name("Nodies")
            .with_priority(55)
            .with_chain_id(chain_id::OPTIMISM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(50), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://optimism.gateway.tenderly.co")
            .with_name("Tenderly")
            .with_priority(56)
            .with_chain_id(chain_id::OPTIMISM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://endpoints.omniatech.io/v1/op/mainnet/public")
            .with_name("OmniaTech")
            .with_priority(57)
            .with_chain_id(chain_id::OPTIMISM),
        // Phase 3: additional endpoints
        RpcEndpoint::new("https://optimism-mainnet.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(58)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://optimism.meowrpc.com")
            .with_name("MeowRPC")
            .with_priority(59)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://optimism.rpc.subquery.network/public")
            .with_name("SubQuery")
            .with_priority(60)
            .with_chain_id(chain_id::OPTIMISM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://rpc.ankr.com/optimism")
            .with_name("Ankr")
            .with_priority(61)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://optimism.public.blockpi.network/v1/rpc/public")
            .with_name("BlockPI")
            .with_priority(62)
            .with_chain_id(chain_id::OPTIMISM),
        RpcEndpoint::new("https://optimism-public.nodies.app")
            .with_name("Nodies Public")
            .with_priority(63)
            .with_chain_id(chain_id::OPTIMISM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://optimism.rpc.thirdweb.com")
            .with_name("thirdweb")
            .with_priority(64)
            .with_chain_id(chain_id::OPTIMISM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://0xrpc.io/op")
            .with_name("0xRPC")
            .with_priority(65)
            .with_chain_id(chain_id::OPTIMISM),
    ]
}

/// Default endpoints for BSC (25 verified endpoints).
pub fn bsc_endpoints() -> Vec<RpcEndpoint> {
    vec![
        // Official BNB Chain endpoints
        RpcEndpoint::new("https://bsc-dataseed.bnbchain.org")
            .with_name("BNB Chain Official")
            .with_priority(50)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(100), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://bsc-dataseed1.bnbchain.org")
            .with_name("BNB Chain 1")
            .with_priority(51)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(100), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://bsc-dataseed2.bnbchain.org")
            .with_name("BNB Chain 2")
            .with_priority(52)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(100), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://bsc-dataseed3.bnbchain.org")
            .with_name("BNB Chain 3")
            .with_priority(53)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(100), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://bsc-dataseed4.bnbchain.org")
            .with_name("BNB Chain 4")
            .with_priority(54)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(100), max_block_range: Some(0), ..Default::default() }),
        // Third-party verified endpoints
        RpcEndpoint::new("https://bsc-dataseed1.defibit.io")
            .with_name("Defibit 1")
            .with_priority(55)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(100), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://bsc-dataseed2.defibit.io")
            .with_name("Defibit 2")
            .with_priority(56)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(100), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://bsc-dataseed3.defibit.io")
            .with_name("Defibit 3")
            .with_priority(57)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(100), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://bsc-dataseed4.defibit.io")
            .with_name("Defibit 4")
            .with_priority(58)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(100), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://bsc-dataseed1.ninicoin.io")
            .with_name("Ninicoin 1")
            .with_priority(59)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(100), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://bsc-dataseed2.ninicoin.io")
            .with_name("Ninicoin 2")
            .with_priority(60)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(100), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://bsc-dataseed3.ninicoin.io")
            .with_name("Ninicoin 3")
            .with_priority(61)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(100), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://bsc-dataseed4.ninicoin.io")
            .with_name("Ninicoin 4")
            .with_priority(62)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(100), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://bsc-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://bsc-rpc.publicnode.com")
            .with_priority(63)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://bsc.publicnode.com")
            .with_name("PublicNode Alt")
            .with_priority(64)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://1rpc.io/bnb")
            .with_name("1RPC")
            .with_priority(65)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://bsc.drpc.org")
            .with_name("dRPC")
            .with_priority(66)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://bsc-mainnet.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(67)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://bsc.meowrpc.com")
            .with_name("MeowRPC")
            .with_priority(68)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(1), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://bsc-pokt.nodies.app")
            .with_name("Nodies")
            .with_priority(69)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(50), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://endpoints.omniatech.io/v1/bsc/mainnet/public")
            .with_name("OmniaTech")
            .with_priority(70)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bnb.rpc.subquery.network/public")
            .with_name("SubQuery")
            .with_priority(71)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(100), ..Default::default() }),
        // Phase 3: additional endpoints
        RpcEndpoint::new("https://rpc.ankr.com/bsc")
            .with_name("Ankr")
            .with_priority(72)
            .with_chain_id(chain_id::BSC),
        RpcEndpoint::new("https://bsc-mainnet.nodereal.io/v1/64a9df0874fb4a93b9d0a3849de012d3")
            .with_name("NodeReal")
            .with_priority(73)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(100), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://binance.llamarpc.com")
            .with_name("LlamaNodes")
            .with_priority(74)
            .with_chain_id(chain_id::BSC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(0), ..Default::default() }),
    ]
}

/// Default endpoints for Avalanche C-Chain (10 verified endpoints).
pub fn avalanche_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://api.avax.network/ext/bc/C/rpc")
            .with_name("Avalanche Official")
            .with_priority(50)
            .with_chain_id(chain_id::AVALANCHE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://avalanche-c-chain-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://avalanche-c-chain-rpc.publicnode.com")
            .with_priority(51)
            .with_chain_id(chain_id::AVALANCHE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://1rpc.io/avax/c")
            .with_name("1RPC")
            .with_priority(52)
            .with_chain_id(chain_id::AVALANCHE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://avalanche.drpc.org")
            .with_name("dRPC")
            .with_priority(53)
            .with_chain_id(chain_id::AVALANCHE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://avalanche.api.onfinality.io/public/ext/bc/C/rpc")
            .with_name("OnFinality")
            .with_priority(54)
            .with_chain_id(chain_id::AVALANCHE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://avax-pokt.nodies.app/ext/bc/C/rpc")
            .with_name("Nodies")
            .with_priority(55)
            .with_chain_id(chain_id::AVALANCHE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(50), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://avalanche.gateway.tenderly.co")
            .with_name("Tenderly")
            .with_priority(56)
            .with_chain_id(chain_id::AVALANCHE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://endpoints.omniatech.io/v1/avax/mainnet/public")
            .with_name("OmniaTech")
            .with_priority(57)
            .with_chain_id(chain_id::AVALANCHE),
        // Phase 3: additional endpoints
        RpcEndpoint::new("https://rpc.ankr.com/avalanche")
            .with_name("Ankr")
            .with_priority(58)
            .with_chain_id(chain_id::AVALANCHE),
        RpcEndpoint::new("https://0xrpc.io/avax")
            .with_name("0xRPC")
            .with_priority(59)
            .with_chain_id(chain_id::AVALANCHE),
    ]
}

/// Default endpoints for Polygon (12 verified endpoints).
pub fn polygon_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://polygon-rpc.com")
            .with_name("Polygon Official")
            .with_priority(50)
            .with_chain_id(chain_id::POLYGON)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(1), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://polygon-bor-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://polygon-bor-rpc.publicnode.com")
            .with_priority(51)
            .with_chain_id(chain_id::POLYGON)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(100), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://1rpc.io/matic")
            .with_name("1RPC")
            .with_priority(52)
            .with_chain_id(chain_id::POLYGON)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://polygon.drpc.org")
            .with_name("dRPC")
            .with_priority(53)
            .with_chain_id(chain_id::POLYGON)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(100), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://polygon.api.onfinality.io/public")
            .with_name("OnFinality")
            .with_priority(54)
            .with_chain_id(chain_id::POLYGON)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(1), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://polygon.gateway.tenderly.co")
            .with_name("Tenderly")
            .with_priority(55)
            .with_chain_id(chain_id::POLYGON)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://polygon.rpc.subquery.network/public")
            .with_name("SubQuery")
            .with_priority(56)
            .with_chain_id(chain_id::POLYGON)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(50), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://endpoints.omniatech.io/v1/matic/mainnet/public")
            .with_name("OmniaTech")
            .with_priority(57)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://polygon-pokt.nodies.app")
            .with_name("Nodies")
            .with_priority(58)
            .with_chain_id(chain_id::POLYGON)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(50), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://rpc-mainnet.matic.quiknode.pro")
            .with_name("QuickNode")
            .with_priority(59)
            .with_chain_id(chain_id::POLYGON)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(0), ..Default::default() }),
        // Phase 3: additional endpoints
        RpcEndpoint::new("https://rpc.ankr.com/polygon")
            .with_name("Ankr")
            .with_priority(60)
            .with_chain_id(chain_id::POLYGON),
        RpcEndpoint::new("https://polygon.llamarpc.com")
            .with_name("LlamaNodes")
            .with_priority(61)
            .with_chain_id(chain_id::POLYGON),
    ]
}

/// Default endpoints for Fantom Opera (12 verified endpoints).
pub fn fantom_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://rpc.ftm.tools")
            .with_name("FTM Tools")
            .with_priority(50)
            .with_chain_id(chain_id::FANTOM),
        RpcEndpoint::new("https://rpc.fantom.network")
            .with_name("Fantom Official")
            .with_priority(51)
            .with_chain_id(chain_id::FANTOM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://rpc2.fantom.network")
            .with_name("Fantom Official 2")
            .with_priority(52)
            .with_chain_id(chain_id::FANTOM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://rpc3.fantom.network")
            .with_name("Fantom Official 3")
            .with_priority(53)
            .with_chain_id(chain_id::FANTOM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://fantom-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://fantom-rpc.publicnode.com")
            .with_priority(54)
            .with_chain_id(chain_id::FANTOM),
        RpcEndpoint::new("https://1rpc.io/ftm")
            .with_name("1RPC")
            .with_priority(55)
            .with_chain_id(chain_id::FANTOM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://fantom.drpc.org")
            .with_name("dRPC")
            .with_priority(56)
            .with_chain_id(chain_id::FANTOM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://fantom-mainnet.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(57)
            .with_chain_id(chain_id::FANTOM),
        RpcEndpoint::new("https://rpc.ankr.com/fantom")
            .with_name("Ankr")
            .with_priority(58)
            .with_chain_id(chain_id::FANTOM),
        RpcEndpoint::new("https://fantom.api.onfinality.io/public")
            .with_name("OnFinality")
            .with_priority(59)
            .with_chain_id(chain_id::FANTOM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://endpoints.omniatech.io/v1/fantom/mainnet/public")
            .with_name("OmniaTech")
            .with_priority(60)
            .with_chain_id(chain_id::FANTOM),
        RpcEndpoint::new("https://fantom-pokt.nodies.app")
            .with_name("Nodies")
            .with_priority(61)
            .with_chain_id(chain_id::FANTOM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(50), max_block_range: Some(100), ..Default::default() }),
    ]
}

/// Default endpoints for zkSync Era (8 verified endpoints).
pub fn zksync_era_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://mainnet.era.zksync.io")
            .with_name("zkSync Official")
            .with_ws_url("wss://mainnet.era.zksync.io/ws")
            .with_priority(50)
            .with_chain_id(chain_id::ZKSYNC_ERA)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(100), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://1rpc.io/zksync2-era")
            .with_name("1RPC")
            .with_priority(51)
            .with_chain_id(chain_id::ZKSYNC_ERA)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://zksync.drpc.org")
            .with_name("dRPC")
            .with_priority(52)
            .with_chain_id(chain_id::ZKSYNC_ERA)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://zksync-mainnet.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(53)
            .with_chain_id(chain_id::ZKSYNC_ERA),
        RpcEndpoint::new("https://rpc.ankr.com/zksync_era")
            .with_name("Ankr")
            .with_priority(54)
            .with_chain_id(chain_id::ZKSYNC_ERA)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://zksync-era.public.blockpi.network/v1/rpc/public")
            .with_name("BlockPI")
            .with_priority(55)
            .with_chain_id(chain_id::ZKSYNC_ERA),
        RpcEndpoint::new("https://endpoints.omniatech.io/v1/zksync-era/mainnet/public")
            .with_name("OmniaTech")
            .with_priority(56)
            .with_chain_id(chain_id::ZKSYNC_ERA),
        RpcEndpoint::new("https://zksync.meowrpc.com")
            .with_name("MeowRPC")
            .with_priority(57)
            .with_chain_id(chain_id::ZKSYNC_ERA),
    ]
}

/// Default endpoints for Linea (7 verified endpoints).
pub fn linea_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://rpc.linea.build")
            .with_name("Linea Official")
            .with_ws_url("wss://rpc.linea.build")
            .with_priority(50)
            .with_chain_id(chain_id::LINEA)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://linea-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://linea-rpc.publicnode.com")
            .with_priority(51)
            .with_chain_id(chain_id::LINEA)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://1rpc.io/linea")
            .with_name("1RPC")
            .with_priority(52)
            .with_chain_id(chain_id::LINEA)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://linea.drpc.org")
            .with_name("dRPC")
            .with_priority(53)
            .with_chain_id(chain_id::LINEA)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://rpc.ankr.com/linea")
            .with_name("Ankr")
            .with_priority(54)
            .with_chain_id(chain_id::LINEA),
        RpcEndpoint::new("https://linea.public.blockpi.network/v1/rpc/public")
            .with_name("BlockPI")
            .with_priority(55)
            .with_chain_id(chain_id::LINEA),
        RpcEndpoint::new("https://linea-pokt.nodies.app")
            .with_name("Nodies")
            .with_priority(56)
            .with_chain_id(chain_id::LINEA),
    ]
}

/// Default endpoints for Hyperliquid EVM (5 verified endpoints).
pub fn hyperliquid_evm_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://rpc.hyperliquid.xyz/evm")
            .with_name("Hyperliquid Official")
            .with_priority(50)
            .with_chain_id(chain_id::HYPERLIQUID_EVM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://rpc.hypurrscan.io/evm")
            .with_name("HypurrScan")
            .with_priority(51)
            .with_chain_id(chain_id::HYPERLIQUID_EVM),
        RpcEndpoint::new("https://hyperliquid.drpc.org")
            .with_name("dRPC")
            .with_priority(52)
            .with_chain_id(chain_id::HYPERLIQUID_EVM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://rpc.hyper-liquid.xyz/evm")
            .with_name("HyperLend")
            .with_priority(53)
            .with_chain_id(chain_id::HYPERLIQUID_EVM),
        RpcEndpoint::new("https://hyperliquid-rpc.stakely.io/evm")
            .with_name("Stakely")
            .with_priority(54)
            .with_chain_id(chain_id::HYPERLIQUID_EVM),
    ]
}

/// Default endpoints for Scroll (12 verified endpoints).
pub fn scroll_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://rpc.scroll.io")
            .with_name("Scroll Official")
            .with_priority(50)
            .with_chain_id(chain_id::SCROLL)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(5000), ..Default::default() }),
        RpcEndpoint::new("https://scroll-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://scroll-rpc.publicnode.com")
            .with_priority(51)
            .with_chain_id(chain_id::SCROLL)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://1rpc.io/scroll")
            .with_name("1RPC")
            .with_priority(52)
            .with_chain_id(chain_id::SCROLL),
        RpcEndpoint::new("https://scroll.drpc.org")
            .with_name("dRPC")
            .with_priority(53)
            .with_chain_id(chain_id::SCROLL)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(5000), ..Default::default() }),
        RpcEndpoint::new("https://scroll-mainnet.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(54)
            .with_chain_id(chain_id::SCROLL),
        RpcEndpoint::new("https://rpc.ankr.com/scroll")
            .with_name("Ankr")
            .with_priority(55)
            .with_chain_id(chain_id::SCROLL),
        RpcEndpoint::new("https://scroll.api.onfinality.io/public")
            .with_name("OnFinality")
            .with_priority(56)
            .with_chain_id(chain_id::SCROLL)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://scroll-public.nodies.app")
            .with_name("Nodies")
            .with_priority(57)
            .with_chain_id(chain_id::SCROLL)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://endpoints.omniatech.io/v1/scroll/mainnet/public")
            .with_name("OmniaTech")
            .with_priority(58)
            .with_chain_id(chain_id::SCROLL),
        RpcEndpoint::new("https://534352.rpc.thirdweb.com")
            .with_name("thirdweb")
            .with_priority(59)
            .with_chain_id(chain_id::SCROLL)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://scroll.api.pocket.network")
            .with_name("Pocket Network")
            .with_priority(60)
            .with_chain_id(chain_id::SCROLL)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://rpc-scroll.icecreamswap.com")
            .with_name("IceCreamSwap")
            .with_priority(61)
            .with_chain_id(chain_id::SCROLL),
    ]
}

/// Default endpoints for Polygon zkEVM (8 verified endpoints).
pub fn polygon_zkevm_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://zkevm-rpc.com")
            .with_name("Polygon zkEVM Official")
            .with_priority(50)
            .with_chain_id(chain_id::POLYGON_ZKEVM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(100), max_block_range: Some(10000), ..Default::default() }),
        RpcEndpoint::new("https://rpc.polygon-zkevm.gateway.fm")
            .with_name("Gateway.fm")
            .with_priority(51)
            .with_chain_id(chain_id::POLYGON_ZKEVM),
        RpcEndpoint::new("https://1rpc.io/polygon/zkevm")
            .with_name("1RPC")
            .with_priority(52)
            .with_chain_id(chain_id::POLYGON_ZKEVM),
        RpcEndpoint::new("https://polygon-zkevm.drpc.org")
            .with_name("dRPC")
            .with_priority(53)
            .with_chain_id(chain_id::POLYGON_ZKEVM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(5000), ..Default::default() }),
        RpcEndpoint::new("https://polygon-zkevm-public.nodies.app")
            .with_name("Nodies")
            .with_priority(54)
            .with_chain_id(chain_id::POLYGON_ZKEVM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(100), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://endpoints.omniatech.io/v1/polygon-zkevm/mainnet/public")
            .with_name("OmniaTech")
            .with_priority(55)
            .with_chain_id(chain_id::POLYGON_ZKEVM),
        RpcEndpoint::new("https://polygon-zkevm.rpc.thirdweb.com")
            .with_name("thirdweb")
            .with_priority(56)
            .with_chain_id(chain_id::POLYGON_ZKEVM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://poly-zkevm.api.pocket.network")
            .with_name("Pocket Network")
            .with_priority(57)
            .with_chain_id(chain_id::POLYGON_ZKEVM)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(500), max_block_range: Some(100), ..Default::default() }),
    ]
}

/// Default endpoints for Blast (11 verified endpoints).
pub fn blast_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://rpc.blast.io")
            .with_name("Blast Official")
            .with_priority(50)
            .with_chain_id(chain_id::BLAST)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(10000), ..Default::default() }),
        RpcEndpoint::new("https://blast-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://blast-rpc.publicnode.com")
            .with_priority(51)
            .with_chain_id(chain_id::BLAST)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://blast.drpc.org")
            .with_name("dRPC")
            .with_priority(52)
            .with_chain_id(chain_id::BLAST)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(10000), ..Default::default() }),
        RpcEndpoint::new("https://blastl2-mainnet.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(53)
            .with_chain_id(chain_id::BLAST),
        RpcEndpoint::new("https://rpc.ankr.com/blast")
            .with_name("Ankr")
            .with_priority(54)
            .with_chain_id(chain_id::BLAST),
        RpcEndpoint::new("https://endpoints.omniatech.io/v1/blast/mainnet/public")
            .with_name("OmniaTech")
            .with_priority(55)
            .with_chain_id(chain_id::BLAST),
        RpcEndpoint::new("https://blast.api.pocket.network")
            .with_name("Pocket Network")
            .with_priority(56)
            .with_chain_id(chain_id::BLAST)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://blast.blockpi.network/v1/rpc/public")
            .with_name("BlockPI")
            .with_priority(57)
            .with_chain_id(chain_id::BLAST),
        RpcEndpoint::new("https://81457.rpc.thirdweb.com")
            .with_name("thirdweb")
            .with_priority(58)
            .with_chain_id(chain_id::BLAST)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://blast.din.dev/rpc")
            .with_name("DIN")
            .with_priority(59)
            .with_chain_id(chain_id::BLAST)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(10000), ..Default::default() }),
        RpcEndpoint::new("https://blast-public.nodies.app")
            .with_name("Nodies")
            .with_priority(60)
            .with_chain_id(chain_id::BLAST)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(100), ..Default::default() }),
    ]
}

/// Default endpoints for Mantle (12 verified endpoints).
pub fn mantle_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://rpc.mantle.xyz")
            .with_name("Mantle Official")
            .with_priority(50)
            .with_chain_id(chain_id::MANTLE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(10), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://mantle-rpc.publicnode.com")
            .with_name("PublicNode")
            .with_ws_url("wss://mantle-rpc.publicnode.com")
            .with_priority(51)
            .with_chain_id(chain_id::MANTLE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(false), max_batch_size: Some(0), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://1rpc.io/mantle")
            .with_name("1RPC")
            .with_priority(52)
            .with_chain_id(chain_id::MANTLE),
        RpcEndpoint::new("https://mantle.drpc.org")
            .with_name("dRPC")
            .with_priority(53)
            .with_chain_id(chain_id::MANTLE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(10000), ..Default::default() }),
        RpcEndpoint::new("https://mantle-mainnet.public.blastapi.io")
            .with_name("BlastAPI")
            .with_priority(54)
            .with_chain_id(chain_id::MANTLE),
        RpcEndpoint::new("https://mantle-public.nodies.app")
            .with_name("Nodies")
            .with_priority(55)
            .with_chain_id(chain_id::MANTLE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://mantle.api.onfinality.io/public")
            .with_name("OnFinality")
            .with_priority(56)
            .with_chain_id(chain_id::MANTLE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://api.zan.top/mantle-mainnet")
            .with_name("ZAN")
            .with_priority(57)
            .with_chain_id(chain_id::MANTLE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(500), max_block_range: Some(10000), ..Default::default() }),
        RpcEndpoint::new("https://endpoints.omniatech.io/v1/mantle/mainnet/public")
            .with_name("OmniaTech")
            .with_priority(58)
            .with_chain_id(chain_id::MANTLE),
        RpcEndpoint::new("https://mantle.api.pocket.network")
            .with_name("Pocket Network")
            .with_priority(59)
            .with_chain_id(chain_id::MANTLE),
        RpcEndpoint::new("https://rpc.ankr.com/mantle")
            .with_name("Ankr")
            .with_priority(60)
            .with_chain_id(chain_id::MANTLE),
        RpcEndpoint::new("https://5000.rpc.thirdweb.com")
            .with_name("thirdweb")
            .with_priority(61)
            .with_chain_id(chain_id::MANTLE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(1000), ..Default::default() }),
    ]
}

/// Default endpoints for Mode (5 verified endpoints).
pub fn mode_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://mainnet.mode.network")
            .with_name("Mode Official")
            .with_priority(50)
            .with_chain_id(chain_id::MODE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(100), max_block_range: Some(0), ..Default::default() }),
        RpcEndpoint::new("https://1rpc.io/mode")
            .with_name("1RPC")
            .with_priority(51)
            .with_chain_id(chain_id::MODE),
        RpcEndpoint::new("https://mode.drpc.org")
            .with_name("dRPC")
            .with_priority(52)
            .with_chain_id(chain_id::MODE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://34443.rpc.thirdweb.com")
            .with_name("thirdweb")
            .with_priority(53)
            .with_chain_id(chain_id::MODE)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://mode.gateway.tenderly.co")
            .with_name("Tenderly")
            .with_priority(54)
            .with_chain_id(chain_id::MODE),
    ]
}

/// Default endpoints for Manta Pacific (7 verified endpoints).
pub fn manta_pacific_endpoints() -> Vec<RpcEndpoint> {
    vec![
        RpcEndpoint::new("https://pacific-rpc.manta.network/http")
            .with_name("Manta Official")
            .with_ws_url("wss://pacific-rpc.manta.network/ws")
            .with_priority(50)
            .with_chain_id(chain_id::MANTA_PACIFIC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://1rpc.io/manta")
            .with_name("1RPC")
            .with_priority(51)
            .with_chain_id(chain_id::MANTA_PACIFIC),
        RpcEndpoint::new("https://manta-pacific.drpc.org")
            .with_name("dRPC")
            .with_priority(52)
            .with_chain_id(chain_id::MANTA_PACIFIC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(1), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://169.rpc.thirdweb.com")
            .with_name("thirdweb")
            .with_priority(53)
            .with_chain_id(chain_id::MANTA_PACIFIC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(100), ..Default::default() }),
        RpcEndpoint::new("https://manta-pacific-gascap.calderachain.xyz/http")
            .with_name("Caldera")
            .with_priority(54)
            .with_chain_id(chain_id::MANTA_PACIFIC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://manta-pacific-aperture.calderachain.xyz/http")
            .with_name("Caldera Aperture")
            .with_priority(55)
            .with_chain_id(chain_id::MANTA_PACIFIC)
            .with_capabilities(EndpointCapabilities { supports_eth_get_logs: Some(true), max_batch_size: Some(0), max_block_range: Some(1000), ..Default::default() }),
        RpcEndpoint::new("https://rpc.ankr.com/manta")
            .with_name("Ankr")
            .with_priority(56)
            .with_chain_id(chain_id::MANTA_PACIFIC),
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
        assert!(endpoints.len() >= 25, "Should have at least 25 endpoints, got {}", endpoints.len());
    }

    #[test]
    fn test_arbitrum_endpoints() {
        let endpoints = arbitrum_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::ARBITRUM_ONE);
        assert!(endpoints.len() >= 21, "Should have at least 21 endpoints, got {}", endpoints.len());
    }

    #[test]
    fn test_optimism_endpoints() {
        let endpoints = optimism_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::OPTIMISM);
        assert!(endpoints.len() >= 16, "Should have at least 16 endpoints, got {}", endpoints.len());
    }

    #[test]
    fn test_base_endpoints() {
        let endpoints = base_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::BASE);
        assert!(endpoints.len() >= 17, "Should have at least 17 endpoints, got {}", endpoints.len());
    }

    #[test]
    fn test_polygon_endpoints() {
        let endpoints = polygon_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::POLYGON);
        assert!(endpoints.len() >= 12, "Should have at least 12 endpoints, got {}", endpoints.len());
    }

    #[test]
    fn test_bsc_endpoints() {
        let endpoints = bsc_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::BSC);
        assert!(endpoints.len() >= 25, "Should have at least 25 endpoints, got {}", endpoints.len());
    }

    #[test]
    fn test_avalanche_endpoints() {
        let endpoints = avalanche_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::AVALANCHE);
        assert!(endpoints.len() >= 10, "Should have at least 10 endpoints, got {}", endpoints.len());
    }

    #[test]
    fn test_fantom_endpoints() {
        let endpoints = fantom_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::FANTOM);
        assert!(endpoints.len() >= 12, "Should have at least 12 endpoints, got {}", endpoints.len());
    }

    #[test]
    fn test_zksync_era_endpoints() {
        let endpoints = zksync_era_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::ZKSYNC_ERA);
        assert!(endpoints.len() >= 8, "Should have at least 8 endpoints, got {}", endpoints.len());
    }

    #[test]
    fn test_linea_endpoints() {
        let endpoints = linea_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::LINEA);
        assert!(endpoints.len() >= 7, "Should have at least 7 endpoints, got {}", endpoints.len());
    }

    #[test]
    fn test_hyperliquid_evm_endpoints() {
        let endpoints = hyperliquid_evm_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::HYPERLIQUID_EVM);
        assert!(endpoints.len() >= 5, "Should have at least 5 endpoints, got {}", endpoints.len());
    }

    #[test]
    fn test_scroll_endpoints() {
        let endpoints = scroll_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::SCROLL);
        assert!(endpoints.len() >= 12, "Should have at least 12 endpoints, got {}", endpoints.len());
    }

    #[test]
    fn test_polygon_zkevm_endpoints() {
        let endpoints = polygon_zkevm_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::POLYGON_ZKEVM);
        assert!(endpoints.len() >= 8, "Should have at least 8 endpoints, got {}", endpoints.len());
    }

    #[test]
    fn test_blast_endpoints() {
        let endpoints = blast_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::BLAST);
        assert!(endpoints.len() >= 11, "Should have at least 11 endpoints, got {}", endpoints.len());
    }

    #[test]
    fn test_mantle_endpoints() {
        let endpoints = mantle_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::MANTLE);
        assert!(endpoints.len() >= 12, "Should have at least 12 endpoints, got {}", endpoints.len());
    }

    #[test]
    fn test_mode_endpoints() {
        let endpoints = mode_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::MODE);
        assert!(endpoints.len() >= 5, "Should have at least 5 endpoints, got {}", endpoints.len());
    }

    #[test]
    fn test_manta_pacific_endpoints() {
        let endpoints = manta_pacific_endpoints();
        assert_valid_endpoints(&endpoints, chain_id::MANTA_PACIFIC);
        assert!(endpoints.len() >= 7, "Should have at least 7 endpoints, got {}", endpoints.len());
    }

    #[test]
    fn test_default_endpoints() {
        assert!(!default_endpoints(chain_id::ARBITRUM_ONE).is_empty());
        assert!(!default_endpoints(chain_id::AVALANCHE).is_empty());
        assert!(!default_endpoints(chain_id::BASE).is_empty());
        assert!(!default_endpoints(chain_id::BLAST).is_empty());
        assert!(!default_endpoints(chain_id::BSC).is_empty());
        assert!(!default_endpoints(chain_id::ETHEREUM).is_empty());
        assert!(!default_endpoints(chain_id::FANTOM).is_empty());
        assert!(!default_endpoints(chain_id::HYPERLIQUID_EVM).is_empty());
        assert!(!default_endpoints(chain_id::LINEA).is_empty());
        assert!(!default_endpoints(chain_id::MANTA_PACIFIC).is_empty());
        assert!(!default_endpoints(chain_id::MANTLE).is_empty());
        assert!(!default_endpoints(chain_id::MODE).is_empty());
        assert!(!default_endpoints(chain_id::OPTIMISM).is_empty());
        assert!(!default_endpoints(chain_id::POLYGON).is_empty());
        assert!(!default_endpoints(chain_id::POLYGON_ZKEVM).is_empty());
        assert!(!default_endpoints(chain_id::SCROLL).is_empty());
        assert!(!default_endpoints(chain_id::ZKSYNC_ERA).is_empty());
        assert!(default_endpoints(99999).is_empty()); // Unknown chain
    }

    #[test]
    fn test_total_endpoint_count() {
        let total: usize = all_chain_ids()
            .iter()
            .map(|&id| default_endpoints(id).len())
            .sum();

        assert!(total >= 200, "Should have at least 200 total endpoints, got {}", total);
    }

    #[test]
    fn test_all_chain_ids() {
        let ids = all_chain_ids();
        assert!(ids.len() >= 17, "Should have at least 17 chains");
        assert!(ids.contains(&chain_id::ETHEREUM));
        assert!(ids.contains(&chain_id::FANTOM));
        assert!(ids.contains(&chain_id::ZKSYNC_ERA));
        assert!(ids.contains(&chain_id::LINEA));
        assert!(ids.contains(&chain_id::HYPERLIQUID_EVM));
        assert!(ids.contains(&chain_id::SCROLL));
        assert!(ids.contains(&chain_id::POLYGON_ZKEVM));
        assert!(ids.contains(&chain_id::BLAST));
        assert!(ids.contains(&chain_id::MANTLE));
        assert!(ids.contains(&chain_id::MODE));
        assert!(ids.contains(&chain_id::MANTA_PACIFIC));
    }

    #[test]
    fn test_chain_name() {
        assert_eq!(chain_name(chain_id::ETHEREUM), "Ethereum");
        assert_eq!(chain_name(chain_id::FANTOM), "Fantom Opera");
        assert_eq!(chain_name(chain_id::ZKSYNC_ERA), "zkSync Era");
        assert_eq!(chain_name(chain_id::LINEA), "Linea");
        assert_eq!(chain_name(chain_id::HYPERLIQUID_EVM), "Hyperliquid EVM");
        assert_eq!(chain_name(chain_id::SCROLL), "Scroll");
        assert_eq!(chain_name(chain_id::POLYGON_ZKEVM), "Polygon zkEVM");
        assert_eq!(chain_name(chain_id::BLAST), "Blast");
        assert_eq!(chain_name(chain_id::MANTLE), "Mantle");
        assert_eq!(chain_name(chain_id::MODE), "Mode");
        assert_eq!(chain_name(chain_id::MANTA_PACIFIC), "Manta Pacific");
        assert_eq!(chain_name(99999), "Unknown");
    }

    #[test]
    fn test_all_chains_have_endpoints() {
        for &id in &all_chain_ids() {
            let endpoints = default_endpoints(id);
            assert!(
                !endpoints.is_empty(),
                "Chain {} ({}) should have endpoints",
                chain_name(id),
                id
            );
        }
    }
}
