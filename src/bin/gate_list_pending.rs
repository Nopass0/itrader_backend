use anyhow::Context;
use colored::Colorize;
use itrader_backend::core::rate_limiter::RateLimiter;
use itrader_backend::core::config::RateLimitsConfig;
use itrader_backend::gate::client::GateClient;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Enable debug logging
    tracing_subscriber::fmt()
        .with_env_filter("itrader_backend=debug")
        .init();
    println!("{}", "=== Gate.io Pending Transactions ===".bright_blue().bold());
    println!();

    // Create client with default config
    let base_url = std::env::var("GATE_API_URL").unwrap_or_else(|_| "https://panel.gate.cx/api/v1".to_string());
    let rate_limits = RateLimitsConfig {
        gate_requests_per_minute: 240,
        bybit_requests_per_minute: 120,
        default_burst_size: 10,
    };
    let rate_limiter = Arc::new(RateLimiter::new(&rate_limits));
    let client = GateClient::new(base_url.clone(), rate_limiter)
        .context("Failed to create Gate client")?;

    // Load cookies
    let cookie_file = ".gate_cookies.json";
    if Path::new(cookie_file).exists() {
        let cookie_data = std::fs::read_to_string(cookie_file)
            .context("Failed to read cookie file")?;
        let cookies = serde_json::from_str(&cookie_data)
            .context("Failed to parse cookies")?;
        let _ = client.set_cookies(cookies).await;
        println!("{}", "✓ Loaded authentication cookies".green());
    } else {
        eprintln!("{}", "Error: Cookie file not found. Please run gate-login first.".red());
        return Ok(());
    }

    // Get available transactions
    println!("\nFetching transactions...");
    println!("API Base URL: {}", base_url);
    match client.get_available_transactions().await {
        Ok(transactions) => {
            let pending = transactions.iter()
                .filter(|t| t.status == 5)
                .collect::<Vec<_>>();
            
            if pending.is_empty() {
                println!("{}", "No pending transactions found (status = 5)".yellow());
            } else {
                println!("{}", format!("Found {} pending transactions:", pending.len()).green());
                println!();
                
                for tx in pending {
                    println!("{}", format!("ID: {}", tx.id).bright_cyan());
                    println!("  Wallet: {}", tx.wallet);
                    
                    // Extract amount
                    if let Some(rub_amount) = tx.amount.trader.get("643") {
                        if let Some(amount) = rub_amount.as_f64() {
                            println!("  Amount: {} RUB", amount);
                        }
                    }
                    
                    println!("  Status: {} (pending approval)", tx.status);
                    println!("  Created: {}", tx.created_at);
                    println!();
                }
            }
            
            // Also show recently approved transactions
            let approved = transactions.iter()
                .filter(|t| t.status == 6)
                .take(3)
                .collect::<Vec<_>>();
            
            if !approved.is_empty() {
                println!("{}", "\nRecently approved transactions:".dimmed());
                for tx in approved {
                    println!("{}", format!("  ID: {} (approved at {})", 
                        tx.id, 
                        tx.approved_at.as_ref().unwrap_or(&"unknown".to_string())
                    ).dimmed());
                }
            }
        }
        Err(e) => {
            eprintln!("{}", format!("Failed to get transactions:").red());
            eprintln!("{}", format!("{:#}", e).red());
            
            // Print the error chain
            let mut source = e.source();
            while let Some(err) = source {
                eprintln!("{}", format!("Caused by: {}", err).red());
                source = err.source();
            }
        }
    }

    Ok(())
}