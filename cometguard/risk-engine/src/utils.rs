use anyhow::{Result, Context};
use ethers::core::types::{Address, U256};
use std::str::FromStr;
use std::fmt::Write;
use tracing::{info, debug, error};

/// Format an Address for display (0x123...abc)
pub fn format_address(address: &Address) -> String {
    let addr_str = format!("{:?}", address);
    let len = addr_str.len();
    if len <= 10 {
        addr_str
    } else {
        format!("{}...{}", &addr_str[..6], &addr_str[len-4..])
    }
}

/// Format a value with a given number of decimals
pub fn format_decimals(value: f64, decimals: usize) -> String {
    format!("{:.*}", decimals, value)
}

/// Format a percentage value (e.g., 0.05 -> "5.00%")
pub fn format_percentage(value: f64) -> String {
    format!("{:.2}%", value * 100.0)
}

/// Format a monetary value with a symbol (e.g., 1000.0 -> "$1,000.00")
pub fn format_money(value: f64, symbol: &str) -> String {
    let abs_value = value.abs();
    let sign = if value < 0.0 { "-" } else { "" };
    
    let mut result = String::new();
    let whole_part = abs_value.trunc() as u64;
    let decimal_part = (abs_value.fract() * 100.0).round() as u64;
    
    let whole_str = whole_part.to_string();
    let chunks: Vec<&str> = whole_str.as_bytes()
        .rchunks(3)
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect();
    
    write!(result, "{}{}", sign, symbol).unwrap();
    for (i, chunk) in chunks.iter().rev().enumerate() {
        if i > 0 {
            write!(result, ",").unwrap();
        }
        write!(result, "{}", chunk).unwrap();
    }
    
    write!(result, ".{:02}", decimal_part).unwrap();
    result
}

/// Convert a string to an Address
pub fn parse_address(address_str: &str) -> Result<Address> {
    Address::from_str(address_str)
        .with_context(|| format!("Failed to parse address: {}", address_str))
}

/// Convert a U256 value to f64, accounting for decimals
pub fn u256_to_f64(value: U256, decimals: u8) -> f64 {
    let decimals_factor = 10u64.pow(decimals as u32) as f64;
    let value_u128 = value.as_u128() as f64;
    value_u128 / decimals_factor
}

/// Convert a f64 value to U256, accounting for decimals
pub fn f64_to_u256(value: f64, decimals: u8) -> U256 {
    let decimals_factor = 10u64.pow(decimals as u32) as f64;
    let value_u128 = (value * decimals_factor).round() as u128;
    U256::from(value_u128)
}

/// Initialize the logger
pub fn init_logger(level: &str) -> Result<()> {
    let level = match level.to_lowercase().as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };
    
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .init();
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_address() {
        let address = Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let formatted = format_address(&address);
        assert_eq!(formatted, "0x1234...5678");
    }
    
    #[test]
    fn test_format_percentage() {
        assert_eq!(format_percentage(0.05), "5.00%");
        assert_eq!(format_percentage(0.123), "12.30%");
    }
    
    #[test]
    fn test_format_money() {
        assert_eq!(format_money(1000.0, "$"), "$1,000.00");
        assert_eq!(format_money(1234567.89, "$"), "$1,234,567.89");
        assert_eq!(format_money(-9876.54, "$"), "-$9,876.54");
    }
    
    #[test]
    fn test_u256_to_f64_and_back() {
        let original = 123.456;
        let decimals = 6;
        let u256_value = f64_to_u256(original, decimals);
        let back_to_f64 = u256_to_f64(u256_value, decimals);
        
        assert!((original - back_to_f64).abs() < 0.000001);
    }
} 