# CometGuard Risk Engine

A predictive risk management toolkit for Compound V3 that analyzes market conditions, simulates risk scenarios, and identifies potential issues.

## Features (Milestone 1)

- Connect to Compound V3 deployments via RPC
- Fetch and analyze market data
- Assess market risks based on configurable parameters
- Simulate basic risk scenarios
- Command-line interface for risk assessment

## Installation

### Prerequisites

- Rust (1.65 or later)
- Cargo (included with Rust)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/FidelCoder/cometguard.git
cd cometguard/risk-engine

# Build the project
cargo build --release
```

The compiled binary will be available at `target/release/risk-engine-cli`.

## Configuration

Create a `config.json` file in the working directory:

```json
{
  "compound": {
    "rpc_url": "https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY",
    "comet_proxy_address": "0xc3d688B66703497DAA19211EEdff47f25384cdc3",
    "configurator_address": "0x316f9708bB98af7dA9c68C1C3b5e79039cD336E3",
    "chain_id": 1
  },
  "risk": {
    "max_utilization_threshold": 0.85,
    "liquidation_threshold_buffer": 0.05,
    "max_price_volatility": 0.1
  },
  "log_level": "info"
}
```

## Usage

### Command-line Interface

```bash
# Show help
cargo run --bin risk-engine-cli -- --help

# Assess risks for a market
cargo run --bin risk-engine-cli -- assess

# Check a user's position (replace with actual address)
cargo run --bin risk-engine-cli -- check-user --user 0x1234567890abcdef1234567890abcdef12345678

# Simulate market conditions
cargo run --bin risk-engine-cli -- simulate
```

### As a Library

You can use the Risk Engine as a library in your Rust code:

```rust
use risk_engine::{RiskEngine, config::Config};

async fn example() {
    // Load or create configuration
    let config = Config::default();
    
    // Create a new risk engine
    let engine = RiskEngine::new(config).await.unwrap();
    
    // Assess risks
    let assessments = engine.assess_risks().await.unwrap();
    
    // Process the assessment results
    for assessment in assessments {
        println!("Market: {}", assessment.market_name);
        println!("Risk Score: {}/100", assessment.risk_score);
    }
}
```

## License

MIT

## Roadmap

- **Milestone 1 (Current)**: Basic risk assessment for Compound V3 markets
- **Milestone 2**: Enhanced risk simulation and stress testing
- **Milestone 3**: Event monitoring and alert system
- **Milestone 4**: Integration with dashboard for visualization 