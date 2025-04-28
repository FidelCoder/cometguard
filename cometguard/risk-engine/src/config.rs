use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::{Result, Context};
use std::fs;

/// Configuration for the Compound V3 deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundConfig {
    /// RPC URL for the Ethereum network (e.g., Mainnet, Goerli)
    pub rpc_url: String,
    /// Address of the Comet Proxy contract
    pub comet_proxy_address: String,
    /// Address of the Configurator contract
    pub configurator_address: String,
    /// Chain ID of the network
    pub chain_id: u64,
}

/// Risk assessment configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// Maximum allowed utilization rate before flagging high risk (0.0-1.0)
    pub max_utilization_threshold: f64,
    /// Liquidation threshold buffer (how close to liquidation to flag as risky)
    pub liquidation_threshold_buffer: f64,
    /// Maximum price volatility percentage to consider high risk
    pub max_price_volatility: f64,
}

/// Main configuration for the Risk Engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Compound-specific configuration
    pub compound: CompoundConfig,
    /// Risk assessment parameters
    pub risk: RiskConfig,
    /// Log level (error, warn, info, debug, trace)
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            compound: CompoundConfig {
                rpc_url: "https://eth-mainnet.alchemyapi.io/v2/demo".to_string(),
                comet_proxy_address: "0xc3d688B66703497DAA19211EEdff47f25384cdc3".to_string(), // Mainnet USDC Comet proxy
                configurator_address: "0x316f9708bB98af7dA9c68C1C3b5e79039cD336E3".to_string(), // Mainnet USDC Configurator
                chain_id: 1,
            },
            risk: RiskConfig {
                max_utilization_threshold: 0.85,
                liquidation_threshold_buffer: 0.05,
                max_price_volatility: 0.1,
            },
            log_level: "info".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let config_str = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config = serde_json::from_str(&config_str)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        Ok(config)
    }

    /// Save configuration to a file
    pub fn to_file(&self, path: &PathBuf) -> Result<()> {
        let config_str = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(path, config_str)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.compound.chain_id, 1);
        assert!(config.risk.max_utilization_threshold > 0.0);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("config.json");
        
        assert!(config.to_file(&file_path).is_ok());
        let loaded_config = Config::from_file(&file_path);
        assert!(loaded_config.is_ok());
        
        let loaded_config = loaded_config.unwrap();
        assert_eq!(config.compound.chain_id, loaded_config.compound.chain_id);
    }
} 