use crate::config::Config;
use crate::models::{Market, UserPosition};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use chrono::{DateTime, Utc};
use ethers::types::Address;

/// Risk severity level
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RiskSeverity {
    /// No significant risk identified
    Low,
    /// Potential risk that should be monitored
    Medium,
    /// High risk that requires attention
    High,
    /// Critical risk that requires immediate action
    Critical,
}

/// Risk category
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskCategory {
    /// Market utilization is too high
    HighUtilization,
    /// Asset price volatility is concerning
    PriceVolatility,
    /// Concentration risk (too many assets concentrated in few accounts)
    Concentration,
    /// Liquidation cascade risk
    LiquidationCascade,
    /// Oracle reliability issues
    OracleReliability,
    /// Smart contract vulnerability or issue
    SmartContractRisk,
}

/// Individual risk finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFinding {
    /// Risk category
    pub category: RiskCategory,
    /// Risk severity
    pub severity: RiskSeverity,
    /// Human-readable description of the risk
    pub description: String,
    /// Additional metadata about the risk (JSON object)
    pub metadata: serde_json::Value,
    /// Timestamp when the risk was identified
    pub timestamp: DateTime<Utc>,
}

/// Market risk assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Market name
    pub market_name: String,
    /// Market address
    pub market_address: Address,
    /// List of identified risks
    pub findings: Vec<RiskFinding>,
    /// Overall risk score (0-100, higher is riskier)
    pub risk_score: u8,
    /// Timestamp of the assessment
    pub timestamp: DateTime<Utc>,
}

/// Risk processor for assessing Compound V3 markets
pub struct RiskProcessor {
    config: Arc<Config>,
}

impl RiskProcessor {
    /// Create a new RiskProcessor instance
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
    
    /// Assess a market for risks
    pub async fn assess_market(&self, market: &Market) -> Result<RiskAssessment> {
        info!("Assessing risks for market: {}", market.name);
        
        let mut findings = Vec::new();
        let now = Utc::now();
        
        // Check for high utilization
        self.check_utilization(market, &mut findings, now);
        
        // For milestone 1, we'll focus on utilization risk only
        // In later milestones, we'll add more risk checks:
        // - Price volatility
        // - Concentration
        // - Liquidation cascade
        // - Oracle reliability
        // - Smart contract risks
        
        // Calculate an overall risk score based on findings
        let risk_score = self.calculate_risk_score(&findings);
        
        let assessment = RiskAssessment {
            market_name: market.name.clone(),
            market_address: market.comet_address,
            findings,
            risk_score,
            timestamp: now,
        };
        
        Ok(assessment)
    }
    
    /// Check for high utilization risk
    fn check_utilization(&self, market: &Market, findings: &mut Vec<RiskFinding>, timestamp: DateTime<Utc>) {
        let utilization = market.utilization_rate;
        let threshold = self.config.risk.max_utilization_threshold;
        
        if utilization > threshold {
            // High utilization is a risk
            let severity = if utilization > threshold + 0.1 {
                RiskSeverity::Critical
            } else if utilization > threshold + 0.05 {
                RiskSeverity::High
            } else {
                RiskSeverity::Medium
            };
            
            let description = format!(
                "Market utilization is {:.2}%, which exceeds the recommended threshold of {:.2}%",
                utilization * 100.0,
                threshold * 100.0
            );
            
            let metadata = serde_json::json!({
                "current_utilization": utilization,
                "threshold": threshold,
                "base_asset": market.base_asset.symbol,
                "total_supply": market.total_supply,
                "total_borrow": market.total_borrow,
            });
            
            findings.push(RiskFinding {
                category: RiskCategory::HighUtilization,
                severity,
                description,
                metadata,
                timestamp,
            });
        }
    }
    
    /// Calculate risk score from findings (0-100, higher is riskier)
    fn calculate_risk_score(&self, findings: &[RiskFinding]) -> u8 {
        if findings.is_empty() {
            return 0;
        }
        
        // Calculate score based on severity and number of findings
        let base_score = findings.iter().map(|f| match f.severity {
            RiskSeverity::Low => 5,
            RiskSeverity::Medium => 15,
            RiskSeverity::High => 30,
            RiskSeverity::Critical => 50,
        }).sum::<u8>();
        
        // Cap at 100
        base_score.min(100)
    }
    
    /// Simulate market conditions with various parameters
    /// This is a placeholder for milestone 1, will be expanded in milestone 2
    pub async fn simulate_market_conditions(&self, market: &Market) -> Result<Vec<RiskFinding>> {
        info!("Simulating market conditions for: {}", market.name);
        
        // For milestone 1, we'll return a simple simulation result
        let mut findings = Vec::new();
        let now = Utc::now();
        
        // Simulate increasing utilization by 10%
        let simulated_utilization = market.utilization_rate + 0.1;
        if simulated_utilization > self.config.risk.max_utilization_threshold {
            let description = format!(
                "Simulated 10% increase in utilization would result in {:.2}% utilization, exceeding threshold",
                simulated_utilization * 100.0
            );
            
            findings.push(RiskFinding {
                category: RiskCategory::HighUtilization,
                severity: RiskSeverity::Medium,
                description,
                metadata: serde_json::json!({
                    "simulated_utilization": simulated_utilization,
                    "current_utilization": market.utilization_rate,
                    "threshold": self.config.risk.max_utilization_threshold,
                }),
                timestamp: now,
            });
        }
        
        Ok(findings)
    }
    
    /// Check if a user's position is at risk of liquidation
    pub fn check_user_liquidation_risk(&self, user: &UserPosition) -> Option<RiskFinding> {
        // If user has no borrow, they can't be liquidated
        if user.total_borrow_value <= 0.0 {
            return None;
        }
        
        // Check if health factor is close to liquidation threshold
        let buffer = self.config.risk.liquidation_threshold_buffer;
        
        if user.health_factor < 1.0 + buffer {
            let severity = if user.health_factor < 1.0 {
                RiskSeverity::Critical
            } else if user.health_factor < 1.0 + (buffer / 2.0) {
                RiskSeverity::High
            } else {
                RiskSeverity::Medium
            };
            
            let description = format!(
                "User position has a health factor of {:.2}, which is close to or below the liquidation threshold",
                user.health_factor
            );
            
            return Some(RiskFinding {
                category: RiskCategory::LiquidationCascade,
                severity,
                description,
                metadata: serde_json::json!({
                    "health_factor": user.health_factor,
                    "buffer": buffer,
                    "collateral_value": user.total_collateral_value,
                    "borrow_value": user.total_borrow_value,
                }),
                timestamp: Utc::now(),
            });
        }
        
        None
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Asset, AssetType};
    use ethers::types::U256;
    use std::collections::HashMap;
    use std::str::FromStr;
    
    fn create_test_market() -> Market {
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
        
        Market {
            name: "USDC".to_string(),
            comet_address: Address::from_str("0xc3d688b66703497daa19211eedff47f25384cdc3").unwrap(),
            base_asset,
            collateral_assets: HashMap::new(),
            total_supply: 1_000_000_000.0,
            total_borrow: 900_000_000.0,
            utilization_rate: 0.9,
            supply_apr: 0.05,
            borrow_apr: 0.08,
            base_tracking_supply_speed: U256::from(0),
            base_tracking_borrow_speed: U256::from(0),
            base_min_interest_rate: U256::from(0),
            base_max_interest_rate: U256::from(0),
        }
    }
    
    #[test]
    fn test_check_utilization() {
        let config = Arc::new(Config::default());
        let processor = RiskProcessor::new(config);
        let market = create_test_market();
        
        let mut findings = Vec::new();
        let now = Utc::now();
        
        processor.check_utilization(&market, &mut findings, now);
        
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, RiskCategory::HighUtilization);
        assert_eq!(findings[0].severity, RiskSeverity::High);
    }
    
    #[test]
    fn test_calculate_risk_score() {
        let config = Arc::new(Config::default());
        let processor = RiskProcessor::new(config);
        
        let findings = vec![
            RiskFinding {
                category: RiskCategory::HighUtilization,
                severity: RiskSeverity::High,
                description: "Test finding".to_string(),
                metadata: serde_json::json!({}),
                timestamp: Utc::now(),
            },
            RiskFinding {
                category: RiskCategory::LiquidationCascade,
                severity: RiskSeverity::Medium,
                description: "Test finding 2".to_string(),
                metadata: serde_json::json!({}),
                timestamp: Utc::now(),
            },
        ];
        
        let score = processor.calculate_risk_score(&findings);
        assert_eq!(score, 45); // 30 (High) + 15 (Medium) = 45
    }
} 