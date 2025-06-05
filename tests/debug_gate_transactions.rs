use itrader_backend::gate::GateClient;
use itrader_backend::core::config::Config;
use itrader_backend::core::rate_limiter::RateLimiter;
use std::sync::Arc;

#[tokio::test]
async fn test_list_available_transactions() {
    // Load configuration
    let config = Config::load().expect("Failed to load config");
    
    // Create Gate.io client
    let rate_limiter = Arc::new(RateLimiter::new(&config.rate_limits));
    let gate_client = GateClient::new(config.gate.base_url.clone(), rate_limiter).expect("Failed to create client");
    
    // Load cookies if available
    let cookies_file = "test_data/gate_cookie.json";
    if std::path::Path::new(cookies_file).exists() {
        gate_client.load_cookies(cookies_file).await.expect("Failed to load cookies");
    } else {
        eprintln!("Gate.io cookies not found. Please run gate-login test first.");
        return;
    }
    
    // Get available transactions
    eprintln!("\n=== FETCHING AVAILABLE TRANSACTIONS ===");
    match gate_client.get_available_transactions().await {
        Ok(transactions) => {
            eprintln!("Found {} available transactions", transactions.len());
            for (i, tx) in transactions.iter().enumerate() {
                eprintln!("\nTransaction {}:", i + 1);
                eprintln!("  ID: {}", tx.id);
                eprintln!("  Status: {}", tx.status);
                eprintln!("  Created at: {}", tx.created_at);
                eprintln!("  Method: {}", tx.method.label);
                if let Some(ref trader) = tx.trader {
                    eprintln!("  Trader ID: {}", trader.id);
                    eprintln!("  Trader Name: {}", trader.name);
                }
                if let Some(ref meta) = tx.meta {
                    eprintln!("  Meta Bank: {:?}", meta.bank);
                    eprintln!("  Meta Card: {:?}", meta.card_number);
                }
                eprintln!("  Amount trader fields: {:?}", tx.amount.trader.keys().collect::<Vec<_>>());
            }
        }
        Err(e) => {
            eprintln!("Failed to get transactions: {}", e);
        }
    }
    
    // Get in-progress transactions
    eprintln!("\n=== FETCHING IN-PROGRESS TRANSACTIONS ===");
    match gate_client.get_in_progress_transactions().await {
        Ok(transactions) => {
            eprintln!("Found {} in-progress transactions", transactions.len());
            for (i, tx) in transactions.iter().enumerate() {
                eprintln!("\nIn-Progress Transaction {}:", i + 1);
                eprintln!("  ID: {}", tx.id);
                eprintln!("  Status: {}", tx.status);
                eprintln!("  Created at: {}", tx.created_at);
                eprintln!("  Method: {}", tx.method.label);
                if let Some(ref trader) = tx.trader {
                    eprintln!("  Trader ID: {}", trader.id);
                    eprintln!("  Trader Name: {}", trader.name);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to get in-progress transactions: {}", e);
        }
    }
    
    // Get all transactions (first page)
    eprintln!("\n=== FETCHING ALL TRANSACTIONS (PAGE 1) ===");
    match gate_client.get_transactions().await {
        Ok(transactions) => {
            eprintln!("Found {} transactions on page 1", transactions.len());
            for (i, tx) in transactions.iter().take(5).enumerate() {
                eprintln!("\nTransaction {}:", i + 1);
                eprintln!("  ID: {}", tx.id);
                eprintln!("  Status: {}", tx.status);
                eprintln!("  Created at: {}", tx.created_at);
                eprintln!("  Payment method: {}", tx.payment_method);
                eprintln!("  Amount: {} {}", tx.amount, tx.currency);
            }
        }
        Err(e) => {
            eprintln!("Failed to get all transactions: {}", e);
        }
    }
}