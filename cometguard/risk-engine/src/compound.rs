use crate::config::Config;
use crate::models::{Asset, AssetType, Market, UserPosition, ProtocolMetrics};
use anyhow::{Result, Context};
use ethers::{
    core::types::{Address, U256},
    providers::{Provider, Http},
    contract::abigen,
};
use std::{sync::Arc, collections::HashMap, str::FromStr};
use tracing::info;
use moka::future::Cache;
use std::time::Duration;

// Generate contracts with inline ABI definitions
abigen!(
    Comet,
    r#"[
        function balanceOf(address) view returns (uint256)
        function baseToken() view returns (address)
        function baseTokenPriceFeed() view returns (address)
        function collateralBalanceOf(address, address) view returns (uint256)
        function totalSupply() view returns (uint256)
        function totalBorrow() view returns (uint256)
    ]"#
);

abigen!(
    CometConfigurator,
    r#"[
        function getConfiguration(address) view returns (tuple(address,address,address,address,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256,uint256))
    ]"#
);

abigen!(
    ERC20,
    r#"[
        function balanceOf(address) view returns (uint256)
        function decimals() view returns (uint8)
        function symbol() view returns (string)
        function name() view returns (string)
    ]"#
);

/// Convert a U256 value to f64, accounting for decimals
pub fn u256_to_f64(value: U256, decimals: u8) -> f64 {
    let decimals_factor = 10u64.pow(decimals as u32) as f64;
    let value_u128 = value.as_u128() as f64;
    value_u128 / decimals_factor
}

/// Client for interacting with Compound V3 contracts
pub struct CompoundClient {
    #[allow(dead_code)]
    provider: Arc<Provider<Http>>,
    #[allow(dead_code)]
    config: Arc<Config>,
    comet_address: Address,
    cache: Cache<String, Arc<Market>>,
}

impl CompoundClient {
    /// Create a new CompoundClient instance
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        let provider = Provider::<Http>::try_from(&config.compound.rpc_url)
            .context("Failed to create Ethereum provider")?;
        let provider = Arc::new(provider);
        
        let comet_address = Address::from_str(&config.compound.comet_proxy_address)
            .context("Invalid Comet proxy address")?;
        
        // Initialize cache with 60 second TTL
        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(60))
            .build();
            
        Ok(Self {
            provider,
            config,
            comet_address,
            cache,
        })
    }
    
    /// Get information about all markets (for milestone 1, only one market is supported)
    pub async fn get_markets(&self) -> Result<Vec<Market>> {
        info!("Fetching market data from Compound V3");
        
        // Check cache first
        let cache_key = format!("markets:{}", self.comet_address);
        if let Some(cached) = self.cache.get(&cache_key) {
            info!("Using cached market data");
            return Ok(vec![cached.as_ref().clone()]);
        }
        
        // Use mock data for milestone 1
        let market = self.create_mock_market().await?;
        
        // Store in cache
        let _ = self.cache.insert(cache_key, Arc::new(market.clone()));
        
        Ok(vec![market])
    }
    
    /// Create a mock market for testing
    async fn create_mock_market(&self) -> Result<Market> {
        // Mocked USDC market
        let base_asset = Asset {
            address: Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(), // USDC
            symbol: "USDC".to_string(),
            decimals: 6,
            price: 1.0,
            asset_type: AssetType::Base,
            collateral_factor: 0.0,
            liquidation_factor: 0.0,
            liquidation_penalty: 0.0,
            supply_cap: U256::from(0),
            borrow_cap: U256::from(0),
        };
        
        // Add WETH as collateral
        let mut collateral_assets = HashMap::new();
        let weth_address = Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap(); // WETH
        
        collateral_assets.insert(weth_address, Asset {
            address: weth_address,
            symbol: "WETH".to_string(),
            decimals: 18,
            price: 2000.0, // Approximate price
            asset_type: AssetType::Collateral,
            collateral_factor: 0.825,
            liquidation_factor: 0.91,
            liquidation_penalty: 0.05,
            supply_cap: U256::from(10_000_000_000_000_000_000_000u128), // 10,000 ETH
            borrow_cap: U256::from(0),
        });
        
        // Create market with mock data
        let market = Market {
            name: "USDC".to_string(),
            comet_address: self.comet_address,
            base_asset,
            collateral_assets,
            total_supply: 1_000_000_000.0,
            total_borrow: 750_000_000.0,
            utilization_rate: 0.75,
            supply_apr: 0.0125,
            borrow_apr: 0.0325,
            base_tracking_supply_speed: U256::from(0),
            base_tracking_borrow_speed: U256::from(0),
            base_min_interest_rate: U256::from(0),
            base_max_interest_rate: U256::from(0),
        };
        
        Ok(market)
    }
    
    /// Get information about a user's position in a market
    pub async fn get_user_position(&self, _market: &Market, user_address: Address) -> Result<UserPosition> {
        // For milestone 1, we'll return a mock user position
        // In a production version, this would make real contract calls
        
        // Mock a user with some USDC supplied and WETH as collateral
        let weth_address = Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap();
        
        let mut collateral_balances = HashMap::new();
        collateral_balances.insert(weth_address, 0.5); // 0.5 ETH collateral
        
        let position = UserPosition {
            address: user_address,
            base_balance: 1000.0, // 1000 USDC supplied
            collateral_balances,
            total_collateral_value: 0.5 * 2000.0, // 0.5 ETH * $2000/ETH
            total_borrow_value: 0.0, // No borrowing
            health_factor: 2.0, // Healthy position
        };
        
        Ok(position)
    }
    
    /// Get protocol metrics for a market
    pub async fn get_protocol_metrics(&self, market: &Market) -> Result<ProtocolMetrics> {
        // For milestone 1, return mock metrics
        let metrics = ProtocolMetrics {
            tvl: market.total_supply * market.base_asset.price,
            total_borrow: market.total_borrow * market.base_asset.price,
            utilization_rate: market.utilization_rate,
            suppliers_count: 1250,
            borrowers_count: 750,
            reserves: 25000000.0,
        };
        
        Ok(metrics)
    }
    
    /// Calculate health factor for a user position
    pub fn calculate_health_factor(&self, base_balance: f64, collateral_balances: &HashMap<Address, f64>, market: &Market) -> f64 {
        // If no borrow, health factor is high
        if base_balance >= 0.0 {
            return 100.0;
        }
        
        // Calculate total collateral value
        let mut total_collateral_value = 0.0;
        for (address, &amount) in collateral_balances {
            if let Some(asset) = market.collateral_assets.get(address) {
                // Apply collateral factor
                total_collateral_value += amount * asset.price * asset.collateral_factor;
            }
        }
        
        // Calculate borrow value
        let borrow_value = -base_balance * market.base_asset.price;
        
        // Health factor is collateral value / borrow value
        if borrow_value > 0.0 {
            total_collateral_value / borrow_value
        } else {
            100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_u256_to_f64() {
        let value = U256::from(1_000_000); // 1 USDC with 6 decimals
        let result = u256_to_f64(value, 6);
        assert_eq!(result, 1.0);
    }
    
    #[tokio::test]
    async fn test_create_mock_market() {
        let config = Arc::new(Config::default());
        let client = CompoundClient::new(config).await.unwrap();
        let market = client.create_mock_market().await.unwrap();
        
        assert_eq!(market.name, "USDC");
        assert_eq!(market.utilization_rate, 0.75);
    }
    
    #[tokio::test]
    async fn test_calculate_health_factor() {
        let config = Arc::new(Config::default());
        let client = CompoundClient::new(config).await.unwrap();
        let market = client.create_mock_market().await.unwrap();
        
        let mut collateral_balances = HashMap::new();
        let weth_address = Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap();
        collateral_balances.insert(weth_address, 1.0); // 1 ETH
        
        // 1000 USDC borrow
        let health_factor = client.calculate_health_factor(-1000.0, &collateral_balances, &market);
        
        // 1 ETH at $2000 with 0.825 collateral factor = $1650
        // $1650 / $1000 = 1.65
        assert!(health_factor > 1.6 && health_factor < 1.7);
    }
} 