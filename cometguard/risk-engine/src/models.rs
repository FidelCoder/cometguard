use ethers::types::{Address, U256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Asset type in Compound V3
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AssetType {
    /// Base asset (e.g., USDC in the USDC market)
    Base,
    /// Collateral asset that can be used to borrow the base asset
    Collateral,
}

/// Asset details in a Compound V3 market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    /// Asset address
    pub address: Address,
    /// Asset symbol (e.g., "WETH", "USDC")
    pub symbol: String,
    /// Asset decimals
    pub decimals: u8,
    /// Asset price in USD
    pub price: f64,
    /// Asset type (base or collateral)
    pub asset_type: AssetType,
    /// Collateral factor (0 for base assets)
    pub collateral_factor: f64,
    /// Liquidation factor (0 for base assets)
    pub liquidation_factor: f64,
    /// Liquidation penalty (0 for base assets)
    pub liquidation_penalty: f64,
    /// Supply cap in asset units
    pub supply_cap: U256,
    /// Borrow cap in asset units (for base assets)
    pub borrow_cap: U256,
}

/// Market information for a Compound V3 deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    /// Market name (e.g., "USDC.e")
    pub name: String,
    /// Comet proxy address
    pub comet_address: Address,
    /// Base asset info
    pub base_asset: Asset,
    /// Collateral assets mapping from address to asset
    pub collateral_assets: HashMap<Address, Asset>,
    /// Total supply of the base asset
    pub total_supply: f64,
    /// Total borrow of the base asset
    pub total_borrow: f64,
    /// Utilization rate (total_borrow / total_supply)
    pub utilization_rate: f64,
    /// Supply APR
    pub supply_apr: f64,
    /// Borrow APR
    pub borrow_apr: f64,
    /// Base tracking supply speed
    pub base_tracking_supply_speed: U256,
    /// Base tracking borrow speed
    pub base_tracking_borrow_speed: U256,
    /// Base min interest rate
    pub base_min_interest_rate: U256,
    /// Base max interest rate
    pub base_max_interest_rate: U256,
}

/// User account position in a Compound V3 market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPosition {
    /// User address
    pub address: Address,
    /// Base asset balance (positive for supply, negative for borrow)
    pub base_balance: f64,
    /// Collateral balances by asset address
    pub collateral_balances: HashMap<Address, f64>,
    /// Total collateral value in USD
    pub total_collateral_value: f64,
    /// Total borrow value in USD
    pub total_borrow_value: f64,
    /// Health factor (>1 is healthy, <1 is liquidatable)
    pub health_factor: f64,
}

/// Price change over time for an asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistory {
    /// Asset address
    pub asset_address: Address,
    /// Asset symbol
    pub symbol: String,
    /// Price points with timestamps and USD values
    pub price_points: Vec<(DateTime<Utc>, f64)>,
    /// 24h price change percentage
    pub price_change_24h: f64,
    /// 7d price change percentage
    pub price_change_7d: f64,
    /// 30d price volatility (standard deviation of daily returns)
    pub volatility_30d: f64,
}

/// Protocol-level metrics for a Compound V3 deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMetrics {
    /// Total Value Locked in USD
    pub tvl: f64,
    /// Total borrow in USD
    pub total_borrow: f64,
    /// Protocol-wide utilization rate
    pub utilization_rate: f64,
    /// Number of active suppliers
    pub suppliers_count: u64,
    /// Number of active borrowers
    pub borrowers_count: u64,
    /// Reserves amount in base asset
    pub reserves: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_market_creation() {
        let base_asset = Asset {
            address: Address::from_str("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48").unwrap(),
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

        let weth_address = Address::from_str("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap();
        let weth_asset = Asset {
            address: weth_address,
            symbol: "WETH".to_string(),
            decimals: 18,
            price: 2000.0,
            asset_type: AssetType::Collateral,
            collateral_factor: 0.825,
            liquidation_factor: 0.91,
            liquidation_penalty: 0.05,
            supply_cap: U256::from(10_000_000_000_000_000_000_000u128), // 10,000 ETH
            borrow_cap: U256::from(0),
        };

        let mut collateral_assets = HashMap::new();
        collateral_assets.insert(weth_address, weth_asset);

        let market = Market {
            name: "USDC".to_string(),
            comet_address: Address::from_str("0xc3d688b66703497daa19211eedff47f25384cdc3").unwrap(),
            base_asset,
            collateral_assets,
            total_supply: 1_000_000_000.0,
            total_borrow: 500_000_000.0,
            utilization_rate: 0.5,
            supply_apr: 0.0125,
            borrow_apr: 0.0325,
            base_tracking_supply_speed: U256::from(0),
            base_tracking_borrow_speed: U256::from(0),
            base_min_interest_rate: U256::from(0),
            base_max_interest_rate: U256::from(0),
        };

        assert_eq!(market.name, "USDC");
        assert_eq!(market.utilization_rate, 0.5);
        assert_eq!(market.collateral_assets.len(), 1);
    }
} 