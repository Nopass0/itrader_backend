use itrader_backend::gate::GateClient;
use itrader_backend::core::{config::Config, rate_limiter::RateLimiter};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::load()?;
    
    // Create rate limiter
    let rate_limiter = Arc::new(RateLimiter::new(&config.rate_limits));
    
    // Create Gate.io client
    let gate_client = GateClient::new(config.gate.base_url.clone(), rate_limiter)?;
    
    // Load cookies
    let cookies_file = "test_data/gate_cookie.json";
    if std::path::Path::new(cookies_file).exists() {
        gate_client.load_cookies(cookies_file).await?;
        println!("Loaded cookies from {}", cookies_file);
    } else {
        eprintln!("Cookie file not found at {}. Please run gate-login test first.", cookies_file);
        return Ok(());
    }
    
    println!("Fetching transactions from Gate.io...\n");
    
    // Get available (completed) transactions
    println!("=== AVAILABLE (COMPLETED) TRANSACTIONS ===");
    match gate_client.get_available_transactions().await {
        Ok(transactions) => {
            if transactions.is_empty() {
                println!("No completed transactions found.");
            } else {
                println!("Found {} completed transactions:\n", transactions.len());
                for (i, tx) in transactions.iter().take(10).enumerate() {
                    println!("{}. Transaction ID: {}", i + 1, tx.id);
                    println!("   Status: {}", tx.status);
                    println!("   Created: {}", tx.created_at);
                    println!("   Amount: {:?}", tx.amount.trader);
                    if let Some(trader) = &tx.trader {
                        println!("   Trader: {} (ID: {})", trader.name, trader.id);
                    }
                    if let Some(bank) = &tx.bank {
                        println!("   Bank: {}", bank.label);
                    }
                    println!();
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch available transactions: {}", e);
        }
    }
    
    // Get in-progress transactions
    println!("\n=== IN-PROGRESS TRANSACTIONS ===");
    match gate_client.get_in_progress_transactions().await {
        Ok(transactions) => {
            if transactions.is_empty() {
                println!("No in-progress transactions found.");
            } else {
                println!("Found {} in-progress transactions:\n", transactions.len());
                for (i, tx) in transactions.iter().take(10).enumerate() {
                    println!("{}. Transaction ID: {}", i + 1, tx.id);
                    println!("   Status: {}", tx.status);
                    println!("   Created: {}", tx.created_at);
                    println!("   Amount: {:?}", tx.amount.trader);
                    if let Some(trader) = &tx.trader {
                        println!("   Trader: {} (ID: {})", trader.name, trader.id);
                    }
                    if let Some(bank) = &tx.bank {
                        println!("   Bank: {}", bank.label);
                    }
                    println!();
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch in-progress transactions: {}", e);
        }
    }
    
    Ok(())
}