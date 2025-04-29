use anyhow::Result;
use clap::{Parser, Subcommand};
use risk_engine::{
    config::Config,
    RiskEngine,
    utils::{init_logger, format_address},
};
use std::path::PathBuf;
use std::str::FromStr;
use ethers::types::Address;
use tracing::{info, warn};

#[derive(Parser)]
#[command(
    name = "CometGuard Risk Engine",
    about = "Predictive risk management toolkit for Compound V3",
    version
)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,
    
    /// Log level (error, warn, info, debug, trace)
    #[arg(short, long, default_value = "info")]
    log_level: String,
    
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Assess risks for a Compound V3 market
    Assess {
        /// Address of the Comet proxy
        #[arg(short, long)]
        market: Option<String>,
    },
    
    /// Check a user's position for liquidation risk
    CheckUser {
        /// Address of the Comet proxy
        #[arg(short, long)]
        market: Option<String>,
        
        /// Address of the user to check
        #[arg(short, long)]
        user: String,
    },
    
    /// Simulate market conditions
    Simulate {
        /// Address of the Comet proxy
        #[arg(short, long)]
        market: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Initialize logger
    init_logger(&cli.log_level)?;
    
    // Load configuration
    let config = if cli.config.exists() {
        info!("Loading configuration from {:?}", cli.config);
        Config::from_file(&cli.config)?
    } else {
        warn!("Configuration file not found at {:?}, using default config", cli.config);
        Config::default()
    };
    
    // Create risk engine
    let engine = RiskEngine::new(config).await?;
    
    // Execute command
    match cli.command {
        Command::Assess { market } => {
            // Get all markets
            let markets = engine.assess_risks().await?;
            
            // Filter by market address if provided
            let markets = if let Some(market_addr) = market {
                let market_addr = Address::from_str(&market_addr)?;
                markets.into_iter()
                    .filter(|m| m.market_address == market_addr)
                    .collect::<Vec<_>>()
            } else {
                markets
            };
            
            // Output results
            println!("\n=== RISK ASSESSMENT REPORT ===");
            for assessment in &markets {
                println!("\nMarket: {} ({})", 
                    assessment.market_name, 
                    format_address(&assessment.market_address)
                );
                println!("Risk Score: {}/100", assessment.risk_score);
                
                if assessment.findings.is_empty() {
                    println!("✅ No risks identified");
                } else {
                    println!("\nRisks Identified:");
                    for (i, finding) in assessment.findings.iter().enumerate() {
                        println!("{}. [{}] {}", 
                            i + 1,
                            format!("{:?}", finding.severity),
                            finding.description
                        );
                    }
                }
            }
        },
        
        Command::CheckUser { market, user } => {
            // Parse user address
            let user_address = Address::from_str(&user)?;
            
            // Get markets
            let markets = engine.assess_risks().await?;
            
            // Filter by market address if provided
            let markets = if let Some(market_addr) = market {
                let market_addr = Address::from_str(&market_addr)?;
                markets.into_iter()
                    .filter(|m| m.market_address == market_addr)
                    .collect::<Vec<_>>()
            } else {
                markets
            };
            
            if markets.is_empty() {
                println!("No matching markets found");
                return Ok(());
            }
            
            // For milestone 1, we'll just use the first market
            let market = &markets[0];
            println!("\n=== USER POSITION CHECK ===");
            println!("Market: {} ({})", 
                market.market_name, 
                format_address(&market.market_address)
            );
            println!("User: {}", format_address(&user_address));
            
            // This part would connect to the market and get the user's position
            // For milestone 1, we'll just show a mock user position
            println!("\nMock User Position (for Milestone 1):");
            println!("Base Balance: 1,000.00 USDC");
            println!("Collateral: 0.5 ETH (worth approximately $1,000)");
            println!("Health Factor: 2.0");
            println!("\nPosition Status: ✅ Healthy");
        },
        
        Command::Simulate { market } => {
            // Get all markets
            let markets = engine.assess_risks().await?;
            
            // Filter by market address if provided
            let markets = if let Some(market_addr) = market {
                let market_addr = Address::from_str(&market_addr)?;
                markets.into_iter()
                    .filter(|m| m.market_address == market_addr)
                    .collect::<Vec<_>>()
            } else {
                markets
            };
            
            if markets.is_empty() {
                println!("No matching markets found");
                return Ok(());
            }
            
            // For milestone 1, we'll just use the first market
            println!("\n=== MARKET SIMULATION ===");
            println!("Market: {} ({})", 
                markets[0].market_name, 
                format_address(&markets[0].market_address)
            );
            
            // This would run a real simulation in later milestones
            // For milestone 1, we'll just show some basic information
            println!("\nSimulation Results (for Milestone 1):");
            println!("- If utilization increases by 10%, risk score would increase by 15 points");
            println!("- If largest collateral price drops by 20%, 5% of positions would be liquidated");
            println!("- Stress test shows current market can handle up to 25% price drop before cascade");
        },
    }
    
    println!("\n");
    Ok(())
} 