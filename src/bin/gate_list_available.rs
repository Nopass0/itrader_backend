use anyhow::Context;
use colored::Colorize;
use itrader_backend::core::config::Config;
use itrader_backend::core::rate_limiter::RateLimiter;
use itrader_backend::gate::client::GateClient;
use std::path::Path;
use std::sync::Arc;
use tracing::debug;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("{}", "=== Gate.io Available Transactions (Debug) ===".bright_blue().bold());
    println!();

    // Load config and create client
    let config = Config::load().context("Failed to load config")?;
    let rate_limiter = Arc::new(RateLimiter::new(&config.rate_limits));
    let client = GateClient::new(config.gate.base_url.clone(), rate_limiter)
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

    // Try different endpoints
    println!("\nTrying different endpoints...\n");
    
    // Test 1: Try the /requests endpoint like in the browser
    println!("{}", "Test 1: /requests endpoint".yellow());
    match client.test_requests_endpoint().await {
        Ok(text) => {
            println!("{}", "✓ Success!".green());
            println!("Response preview: {}", &text[..text.len().min(200)]);
        }
        Err(e) => {
            println!("{}", format!("✗ Failed: {}", e).red());
        }
    }
    
    println!();
    
    // Test 2: Try with the standard method
    println!("{}", "Test 2: Standard get_available_transactions".yellow());
    match client.get_available_transactions().await {
        Ok(transactions) => {
            println!("{}", format!("✓ Found {} transactions", transactions.len()).green());
            
            for (i, tx) in transactions.iter().take(3).enumerate() {
                println!("\nTransaction {}:", i + 1);
                println!("  ID: {}", tx.id);
                println!("  Status: {}", tx.status);
                println!("  Wallet: {}", tx.wallet);
                if let Some(amount) = tx.amount.trader.get("643").and_then(|v| v.as_f64()) {
                    println!("  Amount: {} RUB", amount);
                }
            }
        }
        Err(e) => {
            println!("{}", format!("✗ Failed: {}", e).red());
        }
    }

    Ok(())
}