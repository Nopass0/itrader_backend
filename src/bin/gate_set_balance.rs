use anyhow::Result;
use itrader_backend::gate::GateClient;
use itrader_backend::core::config::Config;
use itrader_backend::core::rate_limiter::RateLimiter;
use rust_decimal::Decimal;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env::set_var("RUST_LOG", "info");
    let _ = tracing_subscriber::fmt::try_init();
    
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <amount>", args[0]);
        eprintln!("Example: {} 500000", args[0]);
        eprintln!("        {} 1000000.50", args[0]);
        std::process::exit(1);
    }
    
    let amount_str = &args[1];
    
    println!("=== Gate.io Balance Manager ===");
    
    // Parse amount
    let amount = match Decimal::from_str(amount_str) {
        Ok(amt) => amt,
        Err(e) => {
            eprintln!("Error: Invalid amount '{}': {}", amount_str, e);
            eprintln!("Please provide a valid number");
            std::process::exit(1);
        }
    };
    
    println!("Target balance: {} RUB", amount);
    println!();
    
    // Load configuration
    let config = Config::load()?;
    let rate_limiter = Arc::new(RateLimiter::new(&config.rate_limits));
    
    // Create client
    let client = GateClient::new("https://www.gate.io".to_string(), rate_limiter)?;
    
    // Load cookies
    match client.load_cookies("gate_cookie.json").await {
        Ok(_) => println!("✓ Cookies loaded successfully"),
        Err(e) => {
            eprintln!("⚠ Warning: Failed to load cookies: {}", e);
            eprintln!("  Make sure to run 'cargo run --bin gate_login' first");
        }
    }
    
    // Get current balance first
    println!("\nChecking current balance...");
    match client.get_balance("RUB").await {
        Ok(balance) => {
            println!("Current balance: {} RUB", balance.available);
            println!("Total: {} RUB", balance.balance);
            println!("Locked: {} RUB", balance.locked);
        }
        Err(e) => {
            eprintln!("Failed to get current balance: {}", e);
        }
    }
    
    // Set new balance
    println!("\nSetting new balance...");
    match client.set_balance("RUB", amount.to_f64().unwrap_or(0.0)).await {
        Ok(_) => {
            println!("✓ Successfully set balance to {} RUB", amount);
            
            // Verify the change
            println!("\nVerifying new balance...");
            match client.get_balance("RUB").await {
                Ok(balance) => {
                    println!("New balance: {} RUB", balance.available);
                    if balance.available == amount {
                        println!("✓ Balance verification successful!");
                    } else {
                        println!("⚠ Warning: Balance doesn't match expected amount");
                        println!("  Expected: {} RUB", amount);
                        println!("  Actual: {} RUB", balance.available);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to verify balance: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to set balance: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}