use crate::config::Config;
use crate::models::{Asset, AssetType, Market, UserPosition, ProtocolMetrics};
use anyhow::{Result, Context};
use ethers::{
    core::types::{Address, U256},
    providers::{Provider, Http, Middleware},
};
use std::{sync::Arc, collections::HashMap, str::FromStr};
use tracing::info;

/// Convert a U256 value to f64, taking into account the number of decimals
fn u256_to_f64(value: U256, decimals: u8) -> f64 {
    let factor = 10u64.pow(decimals as u32) as f64;
    let value_u128 = value.as_u128() as f64;
    value_u128 / factor
}

/// Client for interacting with Compound V3 contracts
pub struct CompoundClient {
    provider: Arc<Provider<Http>>,
    config: Arc<Config>,
    comet_address: Address,
}

impl CompoundClient {
    /// Create a new CompoundClient instance
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        let provider = Provider::<Http>::try_from(&config.compound.rpc_url)
            .context("Failed to create Ethereum provider")?;
        let provider = Arc::new(provider);
        
        let comet_address = Address::from_str(&config.compound.comet_proxy_address)
            .context("Invalid Comet proxy address")?;
        
        Ok(Self {
            provider,
            config,
            comet_address,
        })
    }
    
    /// Get information about all markets (for milestone 1, only one market is supported)
    pub async fn get_markets(&self) -> Result<Vec<Market>> {
        info!("Fetching market data from Compound V3");
        
        // For milestone 1, we'll simplify and use a mock implementation
        // In a production version, this would make real contract calls

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
        
        // Add WBTC as collateral
        let wbtc_address = Address::from_str("0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599").unwrap(); // WBTC
        
        collateral_assets.insert(wbtc_address, Asset {
            address: wbtc_address,
            symbol: "WBTC".to_string(),
            decimals: 8,
            price: 35000.0, // Approximate price
            asset_type: AssetType::Collateral,
            collateral_factor: 0.80,
            liquidation_factor: 0.88,
            liquidation_penalty: 0.05,
            supply_cap: U256::from(500_000_000_000u128), // 5,000 BTC
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
        
        Ok(vec![market])
    }
    
    /// Get information about a user's position in a market
    pub async fn get_user_position(&self, market: &Market, user_address: Address) -> Result<UserPosition> {
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
            health_factor: 100.0, // Healthy position
        };
        
        Ok(position)
    }
    
    /// Get protocol-level metrics for a market
    pub async fn get_protocol_metrics(&self, market: &Market) -> Result<ProtocolMetrics> {
        // Most of this data is already in the market, but we'll calculate a few more metrics
        
        // Get supplier and borrower counts (mocked for milestone 1)
        let suppliers_count = 1000;
        let borrowers_count = 500;
        
        // Calculate TVL (total value of supplied base assets + all collateral)
        let base_tvl = market.total_supply * market.base_asset.price;
        
        // For milestone 1, we'll assume collateral is 50% of base TVL
        let collateral_tvl = base_tvl * 0.5;
        
        let tvl = base_tvl + collateral_tvl;
        
        // Get reserves (simplified for milestone 1)
        let reserves = market.total_supply * 0.05; // Assume 5% of supply is reserves
        
        let metrics = ProtocolMetrics {
            tvl,
            total_borrow: market.total_borrow * market.base_asset.price,
            utilization_rate: market.utilization_rate,
            suppliers_count,
            borrowers_count,
            reserves,
        };
        
        Ok(metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_u256_to_f64() {
        let value = U256::from(1_000_000_000);
        let result = u256_to_f64(value, 6);
        assert_eq!(result, 1000.0);
    }
} 