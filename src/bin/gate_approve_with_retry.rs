use anyhow::Context;
use clap::Parser;
use colored::Colorize;
use itrader_backend::core::config::Config;
use itrader_backend::core::rate_limiter::RateLimiter;
use itrader_backend::gate::client::GateClient;
use itrader_backend::gate::models::Cookie;
use itrader_backend::ocr::pdf::PdfReceiptParser;
use itrader_backend::utils::error::AppError;
use serde_json;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(name = "gate_approve_with_retry")]
#[command(about = "Approve Gate.io transaction with automatic session refresh on expiration")]
struct Args {
    #[arg(long)]
    transaction_id: String,
    
    #[arg(long)]
    receipt_path: String,
    
    #[arg(long, default_value = ".gate_cookies.json")]
    cookie_file: String,
    
    #[arg(long, help = "Skip confirmation prompt")]
    yes: bool,
}

fn format_amount(amount: rust_decimal::Decimal) -> String {
    format!("{} RUB", amount)
}

async fn save_cookies(cookies: &[Cookie], file_path: &str) -> anyhow::Result<()> {
    let json_data = serde_json::to_string_pretty(&cookies)?;
    std::fs::write(file_path, json_data)?;
    info!("Saved {} cookies to {}", cookies.len(), file_path);
    Ok(())
}

async fn refresh_session(client: &GateClient, config: &Config, cookie_file: &str) -> anyhow::Result<()> {
    println!("{}", "Session expired. Attempting to refresh...".yellow());
    
    // Try to load credentials from file
    let creds_path = "test_data/gate_creditials.json";
    if Path::new(creds_path).exists() {
        let creds_data = std::fs::read_to_string(creds_path)
            .context("Failed to read credentials file")?;
        let creds: serde_json::Value = serde_json::from_str(&creds_data)
            .context("Failed to parse credentials")?;
        
        if let (Some(email), Some(password)) = (creds["login"].as_str(), creds["password"].as_str()) {
            println!("Attempting to re-authenticate as {}...", email);
            
            match client.login(email, password).await {
                Ok(_) => {
                    println!("{}", "✓ Re-authentication successful!".green());
                    
                    // Save new cookies
                    let new_cookies = client.get_cookies().await;
                    save_cookies(&new_cookies, cookie_file).await?;
                    
                    Ok(())
                }
                Err(e) => {
                    eprintln!("{}", format!("✗ Re-authentication failed: {}", e).red());
                    Err(e.into())
                }
            }
        } else {
            eprintln!("{}", "✗ Invalid credentials format in file".red());
            Err(anyhow::anyhow!("Invalid credentials format"))
        }
    } else {
        eprintln!("{}", format!("✗ No credentials file found at: {}", creds_path).red());
        eprintln!("Please create a credentials file with 'login' and 'password' fields.");
        Err(anyhow::anyhow!("No credentials available"))
    }
}

async fn try_approve_with_retry(
    client: &GateClient, 
    transaction_id: &str, 
    receipt_path: &str,
    config: &Config,
    cookie_file: &str
) -> anyhow::Result<itrader_backend::gate::models::Payout> {
    // First attempt
    match client.approve_transaction_with_receipt(transaction_id, receipt_path).await {
        Ok(payout) => Ok(payout),
        Err(AppError::SessionExpired) => {
            // Try to refresh session
            refresh_session(client, config, cookie_file).await?;
            
            // Retry approval
            println!("{}", "Retrying transaction approval...".yellow());
            client.approve_transaction_with_receipt(transaction_id, receipt_path)
                .await
                .context("Failed to approve transaction after session refresh")
        }
        Err(e) => Err(e.into())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("{}", "=== Gate.io Transaction Approval Tool (with Auto-Retry) ===".bright_blue().bold());
    println!();
    
    let args = Args::parse();
    
    // Parse PDF receipt
    println!("{}", "Parsing PDF receipt...".yellow());
    let parser = PdfReceiptParser::new();
    let receipt_info = parser.parse_receipt(&args.receipt_path).await
        .context("Failed to parse PDF receipt")?;
    
    // Initialize Gate client
    let config = Config::load()
        .context("Failed to load config")?;
    let rate_limiter = Arc::new(RateLimiter::new(&config.rate_limits));
    let client = GateClient::new(config.gate.base_url.clone(), rate_limiter)
        .context("Failed to create Gate client")?;
    
    // Load cookies
    if Path::new(&args.cookie_file).exists() {
        let cookie_data = std::fs::read_to_string(&args.cookie_file)
            .context("Failed to read cookie file")?;
        let cookies: Vec<Cookie> = serde_json::from_str(&cookie_data)
            .context("Failed to parse cookies")?;
        client.set_cookies(cookies).await?;
        println!("{}", "✓ Loaded authentication cookies".green());
    } else {
        eprintln!("{}", "Error: Cookie file not found. Please authenticate first.".red());
        
        // Try to authenticate with credentials file
        println!("Attempting to authenticate with credentials file...");
        refresh_session(&client, &config, &args.cookie_file).await?;
    }
    
    // Display receipt information
    println!();
    println!("{}", "=== PDF Receipt Information ===".bright_cyan().bold());
    println!("Date/Time:    {}", receipt_info.date_time.to_string());
    println!("Amount:       {}", format_amount(receipt_info.amount));
    println!("Bank:         {}", receipt_info.bank_name.as_ref().unwrap_or(&"Not found".to_string()));
    println!("Phone:        {}", receipt_info.phone_number.as_ref().unwrap_or(&"Not found".to_string()));
    println!("Card:         {}", receipt_info.card_number.as_ref().unwrap_or(&"Not found".to_string()));
    println!("Recipient:    {}", receipt_info.recipient.as_ref().unwrap_or(&"Not found".to_string()));
    println!("Status:       {}", receipt_info.status.as_ref().unwrap_or(&"Not found".to_string()));
    
    // Display transaction ID to approve
    println!();
    println!("{}", "=== Transaction to Approve ===".bright_cyan().bold());
    println!("ID: {}", args.transaction_id);
    println!();
    
    // Confirm approval
    if !args.yes {
        println!("{}", "⚠️  WARNING: This will approve the transaction!".bright_yellow().bold());
        println!("Please verify that the receipt matches the transaction details.");
        println!();
        print!("Do you want to proceed with approval? (yes/no): ");
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        
        if !response.trim().eq_ignore_ascii_case("yes") {
            println!("{}", "❌ Approval cancelled by user".red());
            return Ok(());
        }
    }
    
    // Approve transaction with receipt (with retry logic)
    println!();
    println!("{}", "Approving transaction...".yellow());
    
    match try_approve_with_retry(&client, &args.transaction_id, &args.receipt_path, &config, &args.cookie_file).await {
        Ok(approved_transaction) => {
            println!();
            println!("{}", "✓ Transaction approved successfully!".green().bold());
            println!("New status: {}", approved_transaction.status);
            if let Some(approved_at) = approved_transaction.approved_at {
                println!("Approved at: {}", approved_at);
            }
            
            // Display updated transaction details
            println!();
            println!("{}", "=== Approved Transaction Details ===".bright_green().bold());
            println!("ID:           {}", approved_transaction.id);
            println!("Wallet:       {}", approved_transaction.wallet);
            
            // Extract amount
            if let Some(rub_amount) = approved_transaction.amount.trader.get("643") {
                if let Some(amount) = rub_amount.as_f64() {
                    println!("Amount:       {} RUB", amount);
                }
            }
            
            println!("Status:       {} (approved)", approved_transaction.status);
        }
        Err(e) => {
            eprintln!("{}", format!("✗ Failed to approve transaction: {}", e).red());
            return Err(e.into());
        }
    }
    
    Ok(())
}