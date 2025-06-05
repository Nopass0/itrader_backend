use std::sync::Arc;
use anyhow::Result;
use tracing::{info, debug};

use itrader_backend::core::config::Config;
use itrader_backend::core::rate_limiter::RateLimiter;
use itrader_backend::gate::client::GateClient;
use itrader_backend::gate::api::GateAPI;
use itrader_backend::core::account_storage::AccountStorage;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("Testing Gate.io transaction fetching");

    // Load config
    let config = Config::from_file("config/default.toml")?;
    
    // Create rate limiter
    let rate_limiter = Arc::new(RateLimiter::new());
    
    // Create client
    let client = Arc::new(GateClient::new(
        config.gate.api_url.clone(),
        rate_limiter.clone()
    )?);
    
    // Load account from storage
    let account_storage = Arc::new(AccountStorage::new("db"));
    account_storage.init().await?;
    
    let accounts = account_storage.list_gate_accounts().await?;
    if accounts.is_empty() {
        info!("No Gate accounts found");
        return Ok(());
    }
    
    let account = &accounts[0];
    info!("Using account: {}", account.login);
    
    // Login if no cookies
    if account.cookies.is_none() {
        info!("Logging in...");
        let login_resp = client.login(&account.login, account.password.as_deref().unwrap_or("")).await?;
        info!("Login successful: {:?}", login_resp);
        
        // Save cookies
        let cookies = client.get_cookies().await;
        account_storage.update_gate_cookies(&account.id, serde_json::to_value(&cookies)?).await?;
    } else {
        info!("Loading cookies from storage");
        let cookies = serde_json::from_value(account.cookies.clone().unwrap())?;
        client.set_cookies(cookies).await?;
    }
    
    // Test available transactions endpoint directly
    info!("\n=== Testing get_available_transactions ===");
    match client.get_available_transactions().await {
        Ok(payouts) => {
            info!("Found {} available transactions (status 4 or 5)", payouts.len());
            for payout in payouts.iter().take(5) {
                info!("- ID: {}, Status: {}, Amount: {:?}", payout.id, payout.status, payout.amount.trader);
            }
        }
        Err(e) => {
            info!("Error getting available transactions: {}", e);
        }
    }
    
    // Test pending transactions through API
    info!("\n=== Testing get_pending_transactions through API ===");
    let api = GateAPI::new(client.clone());
    match api.get_pending_transactions().await {
        Ok(transactions) => {
            info!("Found {} pending transactions", transactions.len());
            for tx in transactions.iter().take(5) {
                info!("- ID: {}, Amount: {} {}, Status: {}", tx.id, tx.amount, tx.currency, tx.status);
            }
        }
        Err(e) => {
            info!("Error getting pending transactions: {}", e);
        }
    }
    
    // Test all transactions
    info!("\n=== Testing all transactions ===");
    match client.get_transactions().await {
        Ok(transactions) => {
            info!("Found {} total transactions", transactions.len());
            for tx in transactions.iter().take(5) {
                info!("- ID: {}, Amount: {} {}, Status: {}", tx.id, tx.amount, tx.currency, tx.status);
            }
        }
        Err(e) => {
            info!("Error getting transactions: {}", e);
        }
    }
    
    info!("\nTest complete!");
    Ok(())
}