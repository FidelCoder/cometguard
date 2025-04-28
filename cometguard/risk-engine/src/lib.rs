pub mod compound;
pub mod config;
pub mod models;
pub mod risk;
pub mod utils;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main RiskEngine type that orchestrates all risk assessment operations
pub struct RiskEngine {
    config: Arc<config::Config>,
    compound: Arc<RwLock<compound::CompoundClient>>,
}

impl RiskEngine {
    /// Create a new RiskEngine instance with the provided configuration
    pub async fn new(config: config::Config) -> Result<Self> {
        let config = Arc::new(config);
        let compound = Arc::new(RwLock::new(
            compound::CompoundClient::new(config.clone()).await?,
        ));

        Ok(Self { config, compound })
    }

    /// Run a risk assessment for the specified Compound deployment
    pub async fn assess_risks(&self) -> Result<Vec<risk::RiskAssessment>> {
        let compound = self.compound.read().await;
        let markets = compound.get_markets().await?;
        
        let mut assessments = Vec::new();
        for market in markets {
            let assessment = self.assess_market(&market).await?;
            assessments.push(assessment);
        }
        
        Ok(assessments)
    }
    
    /// Assess a specific market for risks
    async fn assess_market(&self, market: &models::Market) -> Result<risk::RiskAssessment> {
        // For milestone 1, we'll implement a simplified risk assessment
        let risk_processor = risk::RiskProcessor::new(self.config.clone());
        risk_processor.assess_market(market).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_risk_engine_creation() {
        let config = config::Config::default();
        let engine = RiskEngine::new(config).await;
        assert!(engine.is_ok());
    }
}
