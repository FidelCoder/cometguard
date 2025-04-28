use crate::config::Config;
use crate::models::{Asset, AssetType, Market, UserPosition, ProtocolMetrics};
use anyhow::{Result, Context};
use ethers::{
    core::types::{Address, U256},
    providers::{Provider, Http},
    contract::Contract,
    abi::parse_abi,
};
use std::{sync::Arc, collections::HashMap, str::FromStr};
use tracing::info;

// Simplified ABI for Comet (only the functions we need)
const COMET_ABI: &str = r#"[
    {
        "inputs": [],
        "name": "totalSupply",
        "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "totalBorrow",
        "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "baseToken",
        "outputs": [{"internalType": "address", "name": "", "type": "address"}],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "numAssets",
        "outputs": [{"internalType": "uint8", "name": "", "type": "uint8"}],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [{"internalType": "uint8", "name": "i", "type": "uint8"}],
        "name": "getAssetInfo",
        "outputs": [
            {"internalType": "address", "name": "asset", "type": "address"},
            {"internalType": "uint8", "name": "decimals", "type": "uint8"},
            {"internalType": "uint64", "name": "borrowCollateralFactor", "type": "uint64"},
            {"internalType": "uint64", "name": "liquidateCollateralFactor", "type": "uint64"},
            {"internalType": "uint64", "name": "liquidationFactor", "type": "uint64"},
            {"internalType": "uint128", "name": "supplyCap", "type": "uint128"}
        ],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [{"internalType": "address", "name": "asset", "type": "address"}],
        "name": "getPrice",
        "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "getSupplyRate",
        "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "getBorrowRate",
        "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [{"internalType": "address", "name": "account", "type": "address"}],
        "name": "balanceOf",
        "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [{"internalType": "address", "name": "account", "type": "address"}],
        "name": "borrowBalanceOf",
        "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [{"internalType": "address", "name": "account", "type": "address"}, {"internalType": "address", "name": "asset", "type": "address"}],
        "name": "collateralBalanceOf",
        "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
        "stateMutability": "view",
        "type": "function"
    }
]"#;

// Simplified ABI for ERC20 (only the functions we need)
const ERC20_ABI: &str = r#"[
    {
        "inputs": [],
        "name": "symbol",
        "outputs": [{"internalType": "string", "name": "", "type": "string"}],
        "stateMutability": "view",
        "type": "function"
    },
    {
        "inputs": [],
        "name": "decimals",
        "outputs": [{"internalType": "uint8", "name": "", "type": "uint8"}],
        "stateMutability": "view",
        "type": "function"
    }
]"#;

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
    comet_contract: Contract<Arc<Provider<Http>>>,
}

impl CompoundClient {
    /// Create a new CompoundClient instance
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        let provider = Provider::<Http>::try_from(&config.compound.rpc_url)
            .context("Failed to create Ethereum provider")?;
        let provider = Arc::new(provider);
        
        // Parse Comet ABI and create contract instance
        let comet_abi = parse_abi(&[COMET_ABI])?;
        let comet_address = Address::from_str(&config.compound.comet_proxy_address)
            .context("Invalid Comet proxy address")?;
        let comet_contract = Contract::new(comet_address, comet_abi, provider.clone());
        
        Ok(Self {
            provider,
            config,
            comet_contract,
        })
    }
    
    /// Get information about all markets (for milestone 1, only one market is supported)
    pub async fn get_markets(&self) -> Result<Vec<Market>> {
        info!("Fetching market data from Compound V3");
        let comet_address = Address::from_str(&self.config.compound.comet_proxy_address)
            .context("Invalid Comet proxy address")?;
        
        // Get base token address
        let base_token: Address = self.comet_contract.method("baseToken", ())?.call().await?;
        
        // Get base token details
        let base_token_contract = self.get_erc20_contract(base_token).await?;
        let base_token_symbol: String = base_token_contract.method("symbol", ())?.call().await?;
        let base_token_decimals: u8 = base_token_contract.method("decimals", ())?.call().await?;
        
        // Get base token price
        let base_token_price: U256 = self.comet_contract.method("getPrice", (base_token,))?.call().await?;
        let base_token_price_f64 = u256_to_f64(base_token_price, 8); // Price scale is 8 decimals in Compound
        
        // Create base asset
        let base_asset = Asset {
            address: base_token,
            symbol: base_token_symbol.clone(),
            decimals: base_token_decimals,
            price: base_token_price_f64,
            asset_type: AssetType::Base,
            collateral_factor: 0.0,
            liquidation_factor: 0.0,
            liquidation_penalty: 0.0,
            supply_cap: U256::from(0),
            borrow_cap: U256::from(0),
        };
        
        // Get total supply and borrow
        let total_supply: U256 = self.comet_contract.method("totalSupply", ())?.call().await?;
        let total_borrow: U256 = self.comet_contract.method("totalBorrow", ())?.call().await?;
        
        let total_supply_f64 = u256_to_f64(total_supply, base_token_decimals);
        let total_borrow_f64 = u256_to_f64(total_borrow, base_token_decimals);
        
        // Calculate utilization rate
        let utilization_rate = if total_supply_f64 > 0.0 {
            total_borrow_f64 / total_supply_f64
        } else {
            0.0
        };
        
        // Get supply and borrow rates
        let supply_rate: U256 = self.comet_contract.method("getSupplyRate", ())?.call().await?;
        let borrow_rate: U256 = self.comet_contract.method("getBorrowRate", ())?.call().await?;
        
        // Convert rates from per-second to APR (multiply by seconds in a year)
        const SECONDS_PER_YEAR: f64 = 60.0 * 60.0 * 24.0 * 365.0;
        let supply_apr = u256_to_f64(supply_rate, 18) * SECONDS_PER_YEAR;
        let borrow_apr = u256_to_f64(borrow_rate, 18) * SECONDS_PER_YEAR;
        
        // Get collateral assets
        let num_assets: u8 = self.comet_contract.method("numAssets", ())?.call().await?;
        let mut collateral_assets = HashMap::new();
        
        for i in 0..num_assets {
            let asset_info: (Address, u8, u64, u64, u64, u128) = 
                self.comet_contract.method("getAssetInfo", (i,))?.call().await?;
            
            let (asset_address, asset_decimals, borrow_cf, liquidate_cf, liquidation_factor, supply_cap) = asset_info;
            
            // Get asset details from ERC20 contract
            let asset_contract = self.get_erc20_contract(asset_address).await?;
            let asset_symbol: String = asset_contract.method("symbol", ())?.call().await?;
            
            // Get asset price
            let asset_price: U256 = self.comet_contract.method("getPrice", (asset_address,))?.call().await?;
            let asset_price_f64 = u256_to_f64(asset_price, 8); // Price scale is 8 decimals
            
            // Create asset
            let asset = Asset {
                address: asset_address,
                symbol: asset_symbol,
                decimals: asset_decimals,
                price: asset_price_f64,
                asset_type: AssetType::Collateral,
                collateral_factor: u256_to_f64(U256::from(borrow_cf), 18),
                liquidation_factor: u256_to_f64(U256::from(liquidate_cf), 18),
                liquidation_penalty: u256_to_f64(U256::from(liquidation_factor), 18),
                supply_cap: U256::from(supply_cap),
                borrow_cap: U256::from(0),
            };
            
            collateral_assets.insert(asset_address, asset);
        }
        
        // Create market
        let market = Market {
            name: base_token_symbol,
            comet_address,
            base_asset,
            collateral_assets,
            total_supply: total_supply_f64,
            total_borrow: total_borrow_f64,
            utilization_rate,
            supply_apr,
            borrow_apr,
            base_tracking_supply_speed: U256::from(0),
            base_tracking_borrow_speed: U256::from(0),
            base_min_interest_rate: U256::from(0),
            base_max_interest_rate: U256::from(0),
        };
        
        Ok(vec![market])
    }
    
    /// Get information about a user's position in a market
    pub async fn get_user_position(&self, market: &Market, user_address: Address) -> Result<UserPosition> {
        let comet_contract = &self.comet_contract;
        
        // Get base balance (supply or borrow)
        let base_balance: U256 = comet_contract.method("balanceOf", (user_address,))?.call().await?;
        let borrow_balance: U256 = comet_contract.method("borrowBalanceOf", (user_address,))?.call().await?;
        
        let base_decimals = market.base_asset.decimals;
        let base_balance_f64 = if borrow_balance.is_zero() {
            u256_to_f64(base_balance, base_decimals)
        } else {
            -u256_to_f64(borrow_balance, base_decimals)
        };
        
        // Get collateral balances
        let mut collateral_balances = HashMap::new();
        let mut total_collateral_value = 0.0;
        
        for (asset_address, asset) in &market.collateral_assets {
            let collateral_balance: U256 = comet_contract
                .method("collateralBalanceOf", (user_address, *asset_address))?.call().await?;
            
            let collateral_balance_f64 = u256_to_f64(collateral_balance, asset.decimals);
            if collateral_balance_f64 > 0.0 {
                collateral_balances.insert(*asset_address, collateral_balance_f64);
                total_collateral_value += collateral_balance_f64 * asset.price;
            }
        }
        
        // Calculate total borrow value
        let total_borrow_value = if base_balance_f64 < 0.0 {
            -base_balance_f64 * market.base_asset.price
        } else {
            0.0
        };
        
        // Calculate health factor
        let health_factor = if total_borrow_value > 0.0 {
            // For each collateral, we need to apply its liquidation factor
            let adjusted_collateral_value = collateral_balances.iter().fold(0.0, |acc, (address, amount)| {
                let asset = market.collateral_assets.get(address).unwrap();
                acc + (amount * asset.price * asset.liquidation_factor)
            });
            
            adjusted_collateral_value / total_borrow_value
        } else {
            // No borrow, so health factor is effectively infinite, but we return a large number
            100.0
        };
        
        let position = UserPosition {
            address: user_address,
            base_balance: base_balance_f64,
            collateral_balances,
            total_collateral_value,
            total_borrow_value,
            health_factor,
        };
        
        Ok(position)
    }
    
    /// Get protocol-level metrics for a market
    pub async fn get_protocol_metrics(&self, market: &Market) -> Result<ProtocolMetrics> {
        // Most of this data is already in the market, but we'll calculate a few more metrics
        
        // Get supplier and borrower counts (simplified for milestone 1)
        // In a full implementation, we would use events or a subgraph to get accurate counts
        let suppliers_count = 1000;
        let borrowers_count = 500;
        
        // Calculate TVL (total value of supplied base assets + all collateral)
        let base_tvl = market.total_supply * market.base_asset.price;
        
        // For milestone 1, we'll assume we don't know the actual collateral TVL
        // In a full implementation, we would query the contract or a subgraph for this data
        let collateral_tvl = base_tvl * 0.5; // Assume collateral is 50% of base TVL for now
        
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
    
    /// Helper to create an ERC20 contract instance
    async fn get_erc20_contract(&self, token_address: Address) -> Result<Contract<Arc<Provider<Http>>>> {
        let erc20_abi = parse_abi(&[ERC20_ABI])?;
        let contract = Contract::new(token_address, erc20_abi, self.provider.clone());
        Ok(contract)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;
    
    // This would be a more extensive test suite in a real implementation
    // For milestone 1, we're keeping it simple
    #[tokio::test]
    async fn test_u256_to_f64() {
        let value = U256::from(1_000_000_000);
        let result = u256_to_f64(value, 6);
        assert_eq!(result, 1000.0);
    }
} 