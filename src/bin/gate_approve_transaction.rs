use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use itrader_backend::gate::client::GateClient;
use itrader_backend::ocr::pdf::PdfReceiptParser;
use itrader_backend::core::rate_limiter::RateLimiter;
use rust_decimal::prelude::*;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(author, version, about = "Approve Gate.io transaction with PDF receipt verification", long_about = None)]
struct Args {
    /// Transaction ID to approve
    #[arg(short = 't', long)]
    transaction_id: String,

    /// Path to PDF receipt
    #[arg(short = 'r', long)]
    receipt_path: String,

    /// Skip confirmation prompt
    #[arg(short = 'y', long)]
    yes: bool,

    /// Auth cookie file path
    #[arg(short = 'c', long, default_value = ".gate_cookies.json")]
    cookie_file: String,
}

fn extract_transaction_phone(transaction: &itrader_backend::gate::models::Payout) -> Option<String> {
    // Check wallet field - for SBP it contains phone
    let wallet = &transaction.wallet;
    if wallet.chars().all(|c| c.is_numeric()) && wallet.len() >= 10 {
        eprintln!("DEBUG: Found phone in wallet field: {}", wallet);
        return Some(format!("+{}", wallet));
    }
    
    // Check meta field if exists
    if let Some(ref meta) = transaction.meta {
        // Meta doesn't have phone field directly, but check bank field
        if let Some(ref bank) = meta.bank {
            eprintln!("DEBUG: Checking meta.bank for phone: {}", bank);
        }
    }
    
    None
}

fn extract_transaction_bank(transaction: &itrader_backend::gate::models::Payout) -> Option<String> {
    transaction.bank.as_ref().map(|b| b.label.clone())
}

fn format_amount(amount: rust_decimal::Decimal) -> String {
    format!("{} RUB", amount)
}

fn normalize_phone(phone: &str) -> String {
    // Remove all non-digits
    let digits: String = phone.chars().filter(|c| c.is_numeric()).collect();
    
    // Ensure it starts with country code
    if digits.starts_with("7") {
        format!("+{}", digits)
    } else if digits.starts_with("8") && digits.len() == 11 {
        format!("+7{}", &digits[1..])
    } else {
        format!("+{}", digits)
    }
}

fn check_match(transaction_value: &Option<String>, receipt_value: &Option<String>, field_name: &str) -> (bool, String) {
    match (transaction_value, receipt_value) {
        (Some(t), Some(r)) => {
            let matches = if field_name == "Phone" {
                // Normalize phones for comparison
                normalize_phone(t) == normalize_phone(r)
            } else if field_name == "Bank" {
                // Fuzzy bank matching
                let t_lower = t.to_lowercase();
                let r_lower = r.to_lowercase();
                t_lower.contains(&r_lower) || r_lower.contains(&t_lower) ||
                t_lower.split_whitespace().any(|word| r_lower.contains(word))
            } else {
                t == r
            };
            
            if matches {
                (true, format!("✓ {} matches", field_name))
            } else {
                (false, format!("✗ {} mismatch: Transaction: {}, Receipt: {}", field_name, t, r))
            }
        }
        (Some(t), None) => (false, format!("✗ {} missing in receipt: Transaction: {}", field_name, t)),
        (None, Some(r)) => (false, format!("✗ {} missing in transaction: Receipt: {}", field_name, r)),
        (None, None) => (true, format!("- {} not found in either", field_name)),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    println!("{}", "=== Gate.io Transaction Approval Tool ===".bright_blue().bold());
    println!();
    
    // Parse the PDF receipt
    println!("{}", "Parsing PDF receipt...".yellow());
    let parser = PdfReceiptParser::new();
    let receipt_info = parser.parse_receipt(&args.receipt_path).await
        .context("Failed to parse PDF receipt")?;
    
    // Initialize Gate client
    let config = itrader_backend::core::config::Config::load()
        .context("Failed to load config")?;
    let rate_limiter = Arc::new(RateLimiter::new(&config.rate_limits));
    let client = GateClient::new(config.gate.base_url.clone(), rate_limiter)
        .context("Failed to create Gate client")?;
    
    // Load cookies
    if Path::new(&args.cookie_file).exists() {
        client.load_cookies(&args.cookie_file).await
            .context("Failed to load cookies")?;
        println!("{}", "✓ Loaded authentication cookies".green());
    } else {
        eprintln!("{}", "Error: Cookie file not found. Please authenticate first.".red());
        return Ok(());
    }
    
    // Get transaction for approval
    println!("{}", format!("Looking for transaction {}...", args.transaction_id).yellow());
    let transaction = match client.get_transaction_for_approval(&args.transaction_id).await {
        Ok(Some(tx)) => tx,
        Ok(None) => {
            eprintln!("{}", format!("✗ Transaction {} not found", args.transaction_id).red());
            eprintln!("\nPlease ensure:");
            eprintln!("  • The transaction ID is correct");
            eprintln!("  • The transaction exists in your Gate.io account");
            eprintln!("  • The transaction is in available status");
            eprintln!("\nTo list pending transactions, run:");
            eprintln!("  ./test.sh gate-pending");
            return Err(anyhow::anyhow!("Transaction not found"));
        }
        Err(e) => {
            eprintln!("Error: Failed to get transaction\n\nCaused by:\n    {:#}", e);
            return Err(e.into());
        }
    };
    
    // Check if transaction has correct status (5 = pending approval)
    if transaction.status != 5 {
        eprintln!("{}", format!("Error: Transaction status is {} (expected 5 for pending approval)", transaction.status).red());
        return Ok(());
    }
    
    // Display receipt information
    println!();
    println!("{}", "=== PDF Receipt Information ===".bright_cyan().bold());
    println!("Date/Time:    {}", receipt_info.date_time.to_string());
    println!("Amount:       {}", format!("{} RUB", receipt_info.amount));
    println!("Bank:         {}", receipt_info.bank_name.as_ref().unwrap_or(&"Not found".to_string()));
    println!("Phone:        {}", receipt_info.phone_number.as_ref().unwrap_or(&"Not found".to_string()));
    println!("Card:         {}", receipt_info.card_number.as_ref().unwrap_or(&"Not found".to_string()));
    println!("Recipient:    {}", receipt_info.recipient.as_ref().unwrap_or(&"Not found".to_string()));
    println!("Status:       {}", receipt_info.status.as_ref().unwrap_or(&"Not found".to_string()));
    
    // Display transaction information
    println!();
    println!("{}", "=== Transaction Information ===".bright_cyan().bold());
    println!("ID:           {}", transaction.id);
    println!("Wallet:       {}", &transaction.wallet);
    
    // Extract amount from transaction
    let transaction_amount = if let Some(rub_amount) = transaction.amount.trader.get("643") {
        // Convert serde_json::Value to Decimal
        match rub_amount {
            serde_json::Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    rust_decimal::Decimal::try_from(f).unwrap_or(rust_decimal::Decimal::ZERO)
                } else {
                    rust_decimal::Decimal::ZERO
                }
            }
            _ => rust_decimal::Decimal::ZERO
        }
    } else {
        rust_decimal::Decimal::ZERO
    };
    
    println!("Amount:       {}", format_amount(transaction_amount));
    println!("Bank:         {}", extract_transaction_bank(&transaction).unwrap_or_else(|| "Not found".to_string()));
    
    // Display wallet/phone/card info
    if let Some(phone) = extract_transaction_phone(&transaction) {
        let digits: String = phone.chars().filter(|c| c.is_numeric()).collect();
        if digits.len() >= 10 && digits.len() <= 12 {
            println!("Phone:        {}", phone);
        } else if digits.len() >= 16 {
            // It's likely a card number
            println!("Card Number:  {}", transaction.wallet);
        } else {
            println!("Wallet:       {}", transaction.wallet);
        }
    } else {
        println!("Wallet:       {}", transaction.wallet);
    }
    
    println!("Status:       {}", transaction.status);
    
    // Compare fields
    println!();
    println!("{}", "=== Verification Results ===".bright_yellow().bold());
    
    let mut all_match = true;
    let mut messages = Vec::new();
    
    // Check amount
    let amount_matches = transaction_amount == receipt_info.amount;
    if amount_matches {
        messages.push(format!("{} Amount matches", "✓".green()));
    } else {
        messages.push(format!("{} Amount mismatch: Transaction: {}, Receipt: {}", 
            "✗".red(), 
            format_amount(transaction_amount), 
            format_amount(receipt_info.amount)
        ));
        all_match = false;
    }
    
    // Check bank
    let transaction_bank = extract_transaction_bank(&transaction);
    let (bank_matches, bank_msg) = check_match(&transaction_bank, &receipt_info.bank_name, "Bank");
    messages.push(bank_msg);
    if !bank_matches {
        all_match = false;
    }
    
    // Check phone/card
    // In receipts, there's either a phone number OR a card number (with last 4 digits)
    let transaction_phone = extract_transaction_phone(&transaction);
    
    // First try to match against phone number if present in receipt
    if receipt_info.phone_number.is_some() {
        let (phone_matches, phone_msg) = check_match(&transaction_phone, &receipt_info.phone_number, "Phone");
        messages.push(phone_msg);
        if !phone_matches {
            all_match = false;
        }
    } 
    // If no phone in receipt but there's a card number, try to match last 4 digits
    else if let Some(ref receipt_card) = receipt_info.card_number {
        // Extract last 4 digits from receipt card (format: ****5999)
        let receipt_last4 = receipt_card.chars()
            .filter(|c| c.is_numeric())
            .collect::<String>();
        
        // Check if transaction wallet ends with the same digits
        if let Some(ref tx_phone) = transaction_phone {
            let tx_digits: String = tx_phone.chars()
                .filter(|c| c.is_numeric())
                .collect();
            
            if !receipt_last4.is_empty() && tx_digits.ends_with(&receipt_last4) {
                messages.push(format!("{} Card last 4 digits match: {}", "✓".green(), receipt_last4));
            } else {
                messages.push(format!("{} Card mismatch: Transaction ends with {}, Receipt: {}", 
                    "✗".red(), 
                    if tx_digits.len() >= 4 { &tx_digits[tx_digits.len()-4..] } else { &tx_digits },
                    receipt_last4
                ));
                all_match = false;
            }
        } else {
            messages.push(format!("{} No card/phone in transaction to match against receipt card: {}", 
                "✗".red(), 
                receipt_card
            ));
            all_match = false;
        }
    }
    // If neither phone nor card in receipt
    else {
        messages.push(format!("{} No phone or card number found in receipt for verification", "✗".red()));
        all_match = false;
    }
    
    // Display verification results
    for msg in &messages {
        println!("{}", msg);
    }
    
    println!();
    if all_match {
        println!("{}", "✓ All fields match!".green().bold());
    } else {
        println!("{}", "✗ Some fields do not match!".red().bold());
    }
    
    // Ask for confirmation
    if !args.yes {
        println!();
        print!("{}", "Do you want to approve this transaction? (y/N): ".yellow());
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Transaction approval cancelled.".yellow());
            return Ok(());
        }
    }
    
    // Approve the transaction
    println!();
    println!("{}", format!("Approving transaction {} with receipt...", args.transaction_id).yellow());
    
    match client.approve_transaction_with_receipt(&args.transaction_id, &args.receipt_path).await {
        Ok(approved_transaction) => {
            println!("{}", "✓ Transaction approved successfully!".green().bold());
            println!("New status: {}", approved_transaction.status);
            if let Some(approved_at) = approved_transaction.approved_at {
                println!("Approved at: {}", approved_at);
            }
        }
        Err(e) => {
            eprintln!("{}", format!("✗ Failed to approve transaction: {}", e).red());
            return Err(e.into());
        }
    }
    
    Ok(())
}