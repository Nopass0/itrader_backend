use anyhow::Result;
use itrader_backend::gate::GateClient;
use itrader_backend::core::rate_limiter::RateLimiter;
use itrader_backend::core::config::RateLimitsConfig;
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env::set_var("RUST_LOG", "info");
    let _ = tracing_subscriber::fmt::try_init();
    
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <transaction_id>", args[0]);
        eprintln!("Example: {} 2450530", args[0]);
        std::process::exit(1);
    }
    
    let transaction_id = &args[1];
    
    println!("=== Gate.io Transaction Search ===");
    println!("Transaction ID: {}", transaction_id);
    println!();
    
    // Create client with minimal config
    let base_url = env::var("GATE_API_URL").unwrap_or_else(|_| "https://panel.gate.cx/api/v1".to_string());
    let rate_limits = RateLimitsConfig {
        gate_requests_per_minute: 240,
        bybit_requests_per_minute: 120,
        default_burst_size: 10,
    };
    let rate_limiter = Arc::new(RateLimiter::new(&rate_limits));
    let client = GateClient::new(base_url, rate_limiter)?;
    
    // Load cookies
    let cookie_file = env::var("COOKIE_FILE").unwrap_or_else(|_| ".gate_cookies.json".to_string());
    match client.load_cookies(&cookie_file).await {
        Ok(_) => println!("✓ Cookies loaded successfully"),
        Err(e) => {
            eprintln!("⚠ Warning: Failed to load cookies: {}", e);
            eprintln!("  Make sure to run 'cargo run --bin gate_login' first");
        }
    }
    
    println!("\nSearching for transaction...");
    
    // Search for transaction
    match client.search_transaction_by_id(transaction_id).await? {
        Some(transaction) => {
            println!("\n✓ Transaction found!");
            println!("\n=== Transaction Details ===");
            println!("ID: {}", transaction.id);
            println!("Status: {}", transaction.status);
            println!("Wallet: {}", transaction.wallet);
            
            // Amount information
            println!("\nAmount:");
            for (currency, value) in &transaction.amount.trader {
                let currency_name = match currency.as_str() {
                    "643" => "RUB",
                    "000001" => "USDT",
                    _ => currency
                };
                println!("  {}: {}", currency_name, value);
            }
            
            // Total information
            println!("\nTotal:");
            for (currency, value) in &transaction.total.trader {
                let currency_name = match currency.as_str() {
                    "643" => "RUB",
                    "000001" => "USDT",
                    _ => currency
                };
                println!("  {}: {}", currency_name, value);
            }
            
            // Dates
            println!("\nDates:");
            println!("  Created: {}", transaction.created_at);
            println!("  Updated: {}", transaction.updated_at);
            if let Some(expired) = &transaction.expired_at {
                println!("  Expired: {}", expired);
            }
            if let Some(approved) = &transaction.approved_at {
                println!("  Approved: {}", approved);
            }
            
            // Payment method
            println!("\nPayment Method:");
            if let Some(id) = transaction.method.id {
                println!("  ID: {}", id);
            }
            println!("  Label: {}", transaction.method.label);
            
            // Bank information
            if let Some(bank) = &transaction.bank {
                println!("\nBank:");
                println!("  Name: {}", bank.name);
                println!("  Label: {}", bank.label);
            }
            
            // Trader information
            if let Some(trader) = &transaction.trader {
                println!("\nTrader:");
                println!("  ID: {}", trader.id);
                println!("  Name: {}", trader.name);
            }
            
            // Attachments
            if let Some(attachments) = &transaction.attachments {
                if !attachments.is_empty() {
                    println!("\nAttachments: {} files", attachments.len());
                    for (i, attachment) in attachments.iter().enumerate() {
                        println!("  {}. {} ({} bytes)", i + 1, attachment.file_name, attachment.size);
                    }
                }
            }
            
            // Success statistics
            if let Some(tooltip) = &transaction.tooltip {
                if let Some(payments) = &tooltip.payments {
                    println!("\nSuccess Rate:");
                    if let Some(success) = payments.success {
                        println!("  Success: {}", success);
                    }
                    if let Some(rejected) = payments.rejected {
                        println!("  Rejected: {}", rejected);
                    }
                    if let Some(percent) = payments.percent {
                        println!("  Percent: {:.1}%", percent);
                    }
                }
            }
        }
        None => {
            println!("\n✗ Transaction not found!");
            println!("  Transaction ID {} does not exist or is not accessible.", transaction_id);
        }
    }
    
    Ok(())
}