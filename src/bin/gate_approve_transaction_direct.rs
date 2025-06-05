use anyhow::Context;
use clap::Parser;
use colored::Colorize;
use itrader_backend::core::config::Config;
use itrader_backend::core::rate_limiter::RateLimiter;
use itrader_backend::gate::client::GateClient;
use itrader_backend::ocr::pdf::PdfReceiptParser;
use std::path::Path;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(name = "gate_approve_transaction_direct")]
#[command(about = "Approve Gate.io transaction directly with receipt")]
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", "=== Gate.io Direct Transaction Approval Tool ===".bright_blue().bold());
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
        let cookies = serde_json::from_str(&cookie_data)
            .context("Failed to parse cookies")?;
        client.set_cookies(cookies).await;
        println!("{}", "✓ Loaded authentication cookies".green());
    } else {
        eprintln!("{}", "Error: Cookie file not found. Please authenticate first.".red());
        return Ok(());
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
    
    // Approve transaction with receipt
    println!();
    println!("{}", "Approving transaction...".yellow());
    
    match client.approve_transaction_with_receipt(&args.transaction_id, &args.receipt_path).await {
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