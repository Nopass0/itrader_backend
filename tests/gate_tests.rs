use itrader_backend::gate::client::GateClient;
use itrader_backend::utils::error::Result;
use itrader_backend::core::rate_limiter::RateLimiter;
use itrader_backend::core::config::RateLimitsConfig;
use std::sync::Arc;
use std::str::FromStr;
use tokio::fs;
use anyhow::Context;

async fn setup_client() -> Result<Arc<GateClient>> {
    // Load cookies
    let cookie_data = fs::read_to_string("test_data/gate_cookie.json")
        .await
        .context("Failed to read gate_cookie.json")?;
    
    let cookies: Vec<itrader_backend::gate::models::Cookie> = serde_json::from_str(&cookie_data)
        .context("Failed to parse cookies")?;
    
    // Create rate limiter config
    let rate_limits_config = RateLimitsConfig {
        gate_requests_per_minute: 120,  // 2 per second
        bybit_requests_per_minute: 300, // 5 per second
        default_burst_size: 10,
    };
    
    // Create client with real Gate.io API URL
    let rate_limiter = Arc::new(RateLimiter::new(&rate_limits_config));
    let client = Arc::new(GateClient::new(
        "https://panel.gate.cx/api/v1".to_string(),
        rate_limiter
    )?);
    
    // Set cookies
    client.set_cookies(cookies).await?;
    
    Ok(client)
}

#[tokio::test]
async fn test_gate_login_with_credentials() -> Result<()> {
    println!("\n=== Testing Gate.io Login with Credentials ===\n");
    
    // Create rate limiter config
    let rate_limits_config = RateLimitsConfig {
        gate_requests_per_minute: 120,  // 2 per second
        bybit_requests_per_minute: 300, // 5 per second
        default_burst_size: 10,
    };
    
    // Create client
    let rate_limiter = Arc::new(RateLimiter::new(&rate_limits_config));
    let client = GateClient::new(
        "https://panel.gate.cx/api/v1".to_string(),
        rate_limiter
    )?;
    
    // First try to load existing cookies
    let cookies_file = "test_data/gate_cookie.json";
    if std::path::Path::new(cookies_file).exists() {
        println!("  Found existing cookies, loading...");
        match client.load_cookies(cookies_file).await {
            Ok(_) => {
                println!("✓ Successfully loaded cookies");
                
                // Test if cookies are valid by making a simple request
                match client.get_balance("RUB").await {
                    Ok(balance) => {
                        println!("✓ Cookies are valid!");
                        println!("  Current balance: {} RUB", balance.available);
                        return Ok(());
                    }
                    Err(e) => {
                        println!("✗ Cookies are invalid or expired: {}", e);
                        println!("  Will attempt fresh login...");
                    }
                }
            }
            Err(e) => {
                println!("✗ Failed to load cookies: {}", e);
            }
        }
    }
    
    // If cookies don't work, attempt login
    // Load credentials
    let creds_data = fs::read_to_string("test_data/gate_creditials.json")
        .await
        .context("Failed to read gate_creditials.json")?;
    let creds: serde_json::Value = serde_json::from_str(&creds_data)
        .context("Failed to parse credentials")?;
    
    let login = creds["login"].as_str().unwrap();
    let password = creds["password"].as_str().unwrap();
    
    println!("  Attempting login with email: {}", login);
    
    // Attempt login
    let response = client.login(login, password).await?;
    println!("✓ Login successful!");
    println!("  User ID: {:?}", response.user_id);
    
    // Save cookies
    let cookies = client.get_cookies().await;
    let cookie_json = serde_json::to_string_pretty(&cookies)
        .context("Failed to serialize cookies")?;
    fs::write("test_data/gate_cookie.json", cookie_json).await
        .context("Failed to write cookies")?;
    println!("✓ Saved {} cookies to test_data/gate_cookie.json", cookies.len());
    
    Ok(())
}

#[tokio::test]
async fn test_gate_auth_with_cookies() -> Result<()> {
    println!("\n=== Testing Gate.io Authentication with Cookies ===\n");
    
    let client = setup_client().await?;
    
    // Get balance to verify authentication
    let balance = client.get_balance("RUB").await?;
    println!("✓ Successfully authenticated with cookies");
    println!("  Current balance: {} RUB", balance.available);
    println!("  Total balance: {} RUB", balance.balance);
    println!("  Locked: {} RUB", balance.locked);
    
    Ok(())
}

#[tokio::test]
async fn test_gate_set_balance() -> Result<()> {
    println!("\n=== Testing Gate.io Set Balance ===\n");
    
    let client = setup_client().await?;
    
    // Get balance amount from environment variable or use default
    let amount_str = std::env::var("BALANCE_AMOUNT")
        .unwrap_or_else(|_| "100000".to_string());
    
    let test_amount = rust_decimal::Decimal::from_str(&amount_str)
        .unwrap_or_else(|_| rust_decimal::Decimal::from(100000));
    
    println!("Setting balance to: {} RUB", test_amount);
    
    // Set balance
    client.set_balance("RUB", test_amount).await?;
    println!("✓ Successfully set balance to {} RUB", test_amount);
    
    // Verify
    let balance = client.get_balance("RUB").await?;
    println!("  Verified balance: {} RUB", balance.available);
    
    Ok(())
}

#[tokio::test]
async fn test_gate_get_transactions() -> Result<()> {
    println!("\n=== Testing Gate.io Get Transactions ===\n");
    
    let client = setup_client().await?;
    
    let transactions = client.get_transactions().await?;
    println!("Found {} transactions", transactions.len());
    
    for (i, tx) in transactions.iter().take(3).enumerate() {
        println!("\n  Transaction {}:", i + 1);
        println!("    ID: {}", tx.id);
        println!("    Amount: {} {}", tx.amount, tx.currency);
        println!("    Status: {}", tx.status);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_gate_accept_transaction() -> Result<()> {
    println!("\n=== Testing Gate.io Accept Transaction ===\n");
    
    let client = setup_client().await?;
    
    // First, get transactions to find one to accept
    let transactions = client.get_transactions().await?;
    
    if let Some(transaction) = transactions.first() {
        println!("Testing with transaction ID: {}", transaction.id);
        
        match client.accept_transaction(&transaction.id).await {
            Ok(_) => {
                println!("✓ Successfully accepted transaction");
            }
            Err(e) => {
                println!("✗ Failed to accept transaction: {}", e);
                println!("  This might be expected if the transaction is already in a final state");
            }
        }
    } else {
        println!("No transactions found to test accept functionality");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_gate_take_available_transactions() -> Result<()> {
    println!("\n=== Testing Gate.io Take All Available Transactions ===\n");
    
    let client = setup_client().await?;
    
    // Get all available transactions (status 4 with approved_at = null)
    let available_transactions = client.get_available_transactions().await?;
    
    println!("Found {} available transactions", available_transactions.len());
    
    if available_transactions.is_empty() {
        println!("No available transactions to take (status 4 with approved_at = null)");
        return Ok(());
    }
    
    let total_available = available_transactions.len();
    let mut accepted_count = 0;
    let mut failed_count = 0;
    
    for transaction in available_transactions {
        println!("\n--- Transaction {} ---", transaction.id);
        println!("  Status: {}", transaction.status);
        println!("  Approved at: {:?}", transaction.approved_at);
        println!("  Created at: {}", transaction.created_at);
        println!("  Payment method: {}", transaction.method.label);
        
        match client.accept_transaction(&transaction.id.to_string()).await {
            Ok(_) => {
                println!("  ✓ Successfully accepted transaction {}", transaction.id);
                accepted_count += 1;
            }
            Err(e) => {
                println!("  ✗ Failed to accept transaction {}: {}", transaction.id, e);
                failed_count += 1;
            }
        }
    }
    
    println!("\n=== Summary ===");
    println!("Total available transactions: {}", total_available);
    println!("Successfully accepted: {}", accepted_count);
    println!("Failed to accept: {}", failed_count);
    
    Ok(())
}

#[tokio::test]
async fn test_gate_get_in_progress_transactions() -> Result<()> {
    println!("\n=== Testing Gate.io Get In Progress Transactions (Status 5) ===\n");
    
    let client = setup_client().await?;
    
    // Get all in progress transactions (status 5)
    let in_progress = client.get_in_progress_transactions().await?;
    
    println!("Found {} in progress transactions", in_progress.len());
    
    if in_progress.is_empty() {
        println!("No in progress transactions found (status 5)");
        return Ok(());
    }
    
    // Display details of in progress transactions
    for (i, transaction) in in_progress.iter().enumerate() {
        println!("\n--- In Progress Transaction {} ---", i + 1);
        println!("  ID: {}", transaction.id);
        println!("  Status: {}", transaction.status);
        println!("  Wallet: {}", transaction.wallet);
        println!("  Created at: {}", transaction.created_at);
        println!("  Updated at: {}", transaction.updated_at);
        println!("  Approved at: {:?}", transaction.approved_at);
        println!("  Expired at: {:?}", transaction.expired_at);
        print!("  Payment method: {}", transaction.method.label);
        if let Some(id) = transaction.method.id {
            print!(" (ID: {})", id);
        }
        println!();
        
        // Display amounts
        if !transaction.amount.trader.is_empty() {
            println!("  Amount:");
            for (currency, value) in &transaction.amount.trader {
                println!("    {}: {}", currency, value);
            }
        } else {
            println!("  Amount: [empty]");
        }
        
        // Display totals
        if !transaction.total.trader.is_empty() {
            println!("  Total:");
            for (currency, value) in &transaction.total.trader {
                println!("    {}: {}", currency, value);
            }
        }
        
        // Display metadata
        if let Some(meta) = &transaction.meta {
            println!("  Meta info:");
            if let Some(bank) = &meta.bank {
                println!("    Bank: {}", bank);
            }
            if let Some(card) = &meta.card_number {
                println!("    Card: {}", card);
            }
            if let Some(courses) = &meta.courses {
                println!("    Courses: {:?}", courses);
            }
            if let Some(reason) = &meta.reason {
                if reason.trader.is_some() || reason.support.is_some() {
                    println!("    Reason:");
                    if let Some(trader_reason) = &reason.trader {
                        println!("      Trader: {}", trader_reason);
                    }
                    if let Some(support_reason) = &reason.support {
                        println!("      Support: {}", support_reason);
                    }
                }
            }
        }
        
        // Display attachments
        if let Some(attachments) = &transaction.attachments {
            if !attachments.is_empty() {
                println!("  Attachments: {} files", attachments.len());
                for attachment in attachments {
                    println!("    - {} ({} bytes, {})", attachment.file_name, attachment.size, attachment.extension);
                    println!("      URL: {}", attachment.original_url);
                    if let Some(props) = &attachment.custom_properties {
                        println!("      Fake: {}", props.fake);
                    }
                }
            }
        }
        
        // Display trader info
        if let Some(trader) = &transaction.trader {
            println!("  Trader: {} (ID: {})", trader.name, trader.id);
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_gate_search_transaction() -> Result<()> {
    println!("\n=== Testing Gate.io Search Transaction by ID ===\n");
    
    let client = setup_client().await?;
    
    // Get transaction ID from environment variable or use default
    let transaction_id = std::env::var("TRANSACTION_ID")
        .unwrap_or_else(|_| "2450530".to_string());
    println!("Searching for transaction ID: {}", transaction_id);
    
    match client.search_transaction_by_id(&transaction_id).await? {
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
            if let Some(method_type) = transaction.method.method_type {
                println!("  Type: {}", method_type);
            }
            if let Some(status) = transaction.method.status {
                println!("  Status: {}", status);
            }
            if let Some(provider_id) = transaction.method.payment_provider_id {
                println!("  Provider ID: {}", provider_id);
            }
            if let Some(wallet_currency_id) = transaction.method.wallet_currency_id {
                println!("  Wallet Currency ID: {}", wallet_currency_id);
            }
            
            // Bank information
            if let Some(bank) = &transaction.bank {
                println!("\nBank:");
                if let Some(id) = bank.id {
                    println!("  ID: {}", id);
                }
                println!("  Name: {}", bank.name);
                println!("  Label: {}", bank.label);
                println!("  Code: {}", bank.code);
                println!("  Active: {}", bank.active);
                if let Some(meta) = &bank.meta {
                    if let Some(parser) = meta.get("parser") {
                        println!("  Parser support:");
                        if let Some(sms) = parser.get("sms") {
                            println!("    SMS: {}", sms);
                        }
                        if let Some(push) = parser.get("push") {
                            println!("    PUSH: {}", push);
                        }
                    }
                }
            }
            
            // Trader information
            if let Some(trader) = &transaction.trader {
                println!("\nTrader:");
                println!("  ID: {}", trader.id);
                println!("  Name: {}", trader.name);
            }
            
            // Meta information
            if let Some(meta) = &transaction.meta {
                println!("\nMeta Information:");
                
                // Courses
                if let Some(courses) = &meta.courses {
                    if let Some(trader_course) = courses.get("trader") {
                        println!("  Course (trader): {}", trader_course);
                    }
                }
                
                // Reason
                if let Some(reason) = &meta.reason {
                    if reason.trader.is_some() || reason.support.is_some() {
                        println!("  Reason:");
                        if let Some(trader_reason) = &reason.trader {
                            println!("    Trader: {}", trader_reason);
                        }
                        if let Some(support_reason) = &reason.support {
                            println!("    Support: {}", support_reason);
                        }
                    }
                }
                
                // Other meta fields
                if let Some(bank) = &meta.bank {
                    println!("  Bank: {}", bank);
                }
                if let Some(card) = &meta.card_number {
                    println!("  Card: {}", card);
                }
            }
            
            // Attachments
            if let Some(attachments) = &transaction.attachments {
                if !attachments.is_empty() {
                    println!("\nAttachments ({}):", attachments.len());
                    for (i, attachment) in attachments.iter().enumerate() {
                        println!("  {}. {} ({}, {} bytes)", 
                            i + 1, 
                            attachment.file_name,
                            attachment.extension,
                            attachment.size
                        );
                        println!("     URL: {}", attachment.original_url);
                        println!("     Created: {}", attachment.created_at);
                        if let Some(props) = &attachment.custom_properties {
                            println!("     Fake: {}", props.fake);
                        }
                    }
                }
            }
            
            // Tooltip information
            if let Some(tooltip) = &transaction.tooltip {
                if let Some(payments) = &tooltip.payments {
                    println!("\nPayment Statistics:");
                    if let Some(success) = payments.success {
                        println!("  Success: {}", success);
                    }
                    if let Some(rejected) = payments.rejected {
                        println!("  Rejected: {}", rejected);
                    }
                    if let Some(percent) = payments.percent {
                        println!("  Success Rate: {}%", percent);
                    }
                }
                if !tooltip.reasons.is_empty() {
                    println!("  Rejection Reasons:");
                    for reason in &tooltip.reasons {
                        println!("    - {}", reason);
                    }
                }
            }
        }
        None => {
            println!("✗ Transaction {} not found", transaction_id);
        }
    }
    
    Ok(())
}