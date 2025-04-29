# CometGuard Risk Engine

A predictive risk management toolkit for Compound V3 that analyzes market conditions, simulates risk scenarios, and identifies potential issues.

## Features (Milestone 1)

- Connect to Compound V3 deployments via RPC
- Fetch and analyze market data
- Assess market risks based on configurable parameters
- Simulate basic risk scenarios
- Command-line interface for risk assessment

## Detailed Setup Guide

### Prerequisites

- Rust (1.65 or later) - [Install Rust](https://www.rust-lang.org/tools/install)
- Cargo (included with Rust)
- Git (for cloning the repository)
- An Ethereum RPC endpoint (public or private)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/your-username/cometguard.git
cd cometguard/risk-engine

# Build the project in development mode
cargo build

# Or build with optimizations for production use
cargo build --release
```

The compiled binary will be available at:
- Development build: `target/debug/risk-engine-cli`
- Release build: `target/release/risk-engine-cli`

## Configuration

The Risk Engine uses a configuration file to set various parameters. By default, it looks for `config.json` in the current directory.

### Sample Configuration

```json
{
  "compound": {
    "rpc_url": "https://ethereum.publicnode.com",
    "comet_proxy_address": "0xc3d688B66703497DAA19211EEdff47f25384cdc3",
    "configurator_address": "0x316f9708bB98af7dA9c68C1C3b5e79039cD336E3",
    "chain_id": 1
  },
  "risk": {
    "max_utilization_threshold": 0.85,
    "liquidation_threshold_buffer": 0.05,
    "max_price_volatility": 0.1
  },
  "logging": {
    "level": "info"
  },
  "cache": {
    "ttl_seconds": 60,
    "max_capacity": 100
  },
  "performance": {
    "allow_parallel_requests": true,
    "timeout_seconds": 15
  }
}
```

### Configuration Parameters

#### Compound Settings
- `rpc_url`: URL of the Ethereum RPC endpoint
- `comet_proxy_address`: Address of the Compound V3 Comet proxy contract
- `configurator_address`: Address of the Compound V3 configurator contract
- `chain_id`: Ethereum chain ID (1 for mainnet)

#### Risk Parameters
- `max_utilization_threshold`: Maximum safe utilization rate (0.0-1.0)
- `liquidation_threshold_buffer`: Buffer to maintain above liquidation threshold
- `max_price_volatility`: Maximum acceptable price volatility for collateral

#### Logging
- `level`: Log level (error, warn, info, debug, trace)

#### Cache Settings
- `ttl_seconds`: Time-to-live for cached data in seconds
- `max_capacity`: Maximum number of items to cache

#### Performance Settings
- `allow_parallel_requests`: Whether to make parallel RPC requests
- `timeout_seconds`: Timeout for RPC requests in seconds

## Usage Examples

### Command-line Interface

```bash
# Display help information
cargo run --bin risk-engine-cli -- --help

# Assess risks for all markets
cargo run --bin risk-engine-cli -- assess

# Assess a specific market (using its address)
cargo run --bin risk-engine-cli -- assess --market 0xc3d688B66703497DAA19211EEdff47f25384cdc3

# Check a user's position (replace with actual address)
cargo run --bin risk-engine-cli -- check-user --user 0x1234567890abcdef1234567890abcdef12345678

# Simulate market conditions
cargo run --bin risk-engine-cli -- simulate

# Use a custom configuration file
cargo run --bin risk-engine-cli -- --config custom-config.json assess

# Set a different log level
cargo run --bin risk-engine-cli -- --log-level debug assess
```

### Understanding the Output

#### Risk Assessment Output

When running the `assess` command, you'll see output like:

```
=== RISK ASSESSMENT REPORT ===

Market: USDC (0xc3d6...cdc3)
Risk Score: 15/100

Risks Identified:
1. [Medium] Utilization rate of 85% is approaching maximum threshold
2. [Low] Collateral concentration in ETH exceeds 70% of total value
```

The risk score ranges from 0-100, with higher scores indicating greater risk.

#### User Position Check Output

The `check-user` command produces output like:

```
=== USER POSITION CHECK ===
Market: USDC (0xc3d6...cdc3)
User: 0x1234...5678

Position:
Base Balance: 1,000.00 USDC
Collateral:
- 0.5 ETH (worth $1,000)
Health Factor: 2.00

Position Status: ✅ Healthy
```

A health factor above 1.0 indicates a healthy position. The closer it gets to 1.0, the riskier the position becomes.

#### Simulation Output

The `simulate` command shows:

```
=== MARKET SIMULATION ===
Market: USDC (0xc3d6...cdc3)

Simulation Results:
1. [Medium] If utilization increases by 10%, risk score would increase by 15 points
2. [High] If largest collateral price drops by 20%, 5% of positions would be liquidated
3. [Low] Stress test shows current market can handle up to 25% price drop before cascade
```

These simulations help predict how different market conditions might affect risk levels.

## For Developers

### Project Structure

```
src/
├── abi/              # Ethereum ABI definitions
├── bin/              # CLI application
├── compound.rs       # Compound V3 client implementation
├── config.rs         # Configuration handling
├── lib.rs            # Library entry point
├── models.rs         # Data models
├── risk.rs           # Risk assessment logic
└── utils.rs          # Utility functions
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test --package risk-engine --lib compound

# Run with verbose output
cargo test -- --nocapture
```

### Using as a Library

You can incorporate the Risk Engine into your own Rust applications:

```rust
use risk_engine::{RiskEngine, config::Config};
use std::path::Path;

async fn main() -> anyhow::Result<()> {
    // Load configuration from file
    let config = Config::from_file(Path::new("config.json"))?;
    
    // Create a new risk engine
    let engine = RiskEngine::new(config).await?;
    
    // Assess risks for all markets
    let assessments = engine.assess_risks().await?;
    
    // Process results
    for assessment in assessments {
        println!("Market: {}", assessment.market_name);
        println!("Risk Score: {}/100", assessment.risk_score);
        
        if !assessment.findings.is_empty() {
            println!("Findings:");
            for finding in &assessment.findings {
                println!("- {}", finding.description);
            }
        }
    }
    
    Ok(())
}
```

## Troubleshooting

### Common Issues

1. **RPC Connection Problems**:
   - Check your internet connection
   - Verify the RPC URL in your configuration
   - Try a different RPC provider

2. **Permission Errors**:
   - Ensure you have read/write permissions in the project directory

3. **Build Errors**:
   - Make sure you have the latest Rust version: `rustup update`
   - Clear target directory: `cargo clean`
   - Verify dependencies: `cargo update`

## Future Development

- Real-time monitoring of positions
- Advanced simulation scenarios
- Historical data analysis
- Integration with other DeFi protocols

## License

MIT 