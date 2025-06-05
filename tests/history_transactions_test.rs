use itrader_backend::gate::{GateClient, Payout};
use itrader_backend::ocr::pdf::PdfReceiptParser;
use itrader_backend::core::config::Config;
use itrader_backend::core::rate_limiter::RateLimiter;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::sync::Arc;
use colored::Colorize;
use std::str::FromStr;
use tempfile::NamedTempFile;
use std::io::Write;

#[derive(Debug)]
struct TransactionComparison {
    transaction_id: i64,
    wallet: String,
    amount: Decimal,
    bank: String,
    has_receipt: bool,
    receipt_url: Option<String>,
    phone_or_card: Option<String>,
    receipt_amount: Option<Decimal>,
    receipt_bank: Option<String>,
    receipt_phone_or_card: Option<String>,
    matches: bool,
}

#[tokio::test]
async fn test_history_transactions_with_receipts() {
    // Load configuration
    let config = Config::load().expect("Failed to load config");
    
    // Create Gate.io client
    let rate_limiter = Arc::new(RateLimiter::new(&config.rate_limits));
    let gate_client = GateClient::new(config.gate.base_url.clone(), rate_limiter.clone())
        .expect("Failed to create Gate client");
    
    // Load cookies
    let cookies_file = "test_data/gate_cookie.json";
    if !std::path::Path::new(cookies_file).exists() {
        eprintln!("Gate.io cookies not found. Please run gate-login test first.");
        return;
    }
    
    gate_client.load_cookies(cookies_file).await
        .expect("Failed to load cookies");
    
    println!("\n{}", "=== FETCHING HISTORY TRANSACTIONS ===".bold().cyan());
    
    // Get history transactions
    let transactions = gate_client.get_history_transactions(Some(1)).await
        .expect("Failed to get history transactions");
    
    if transactions.is_empty() {
        println!("No history transactions found.");
        return;
    }
    
    println!("Found {} history transactions\n", transactions.len());
    
    let mut comparisons = Vec::new();
    let pdf_parser = PdfReceiptParser::new();
    
    // Process each transaction
    for (idx, transaction) in transactions.iter().enumerate() {
        println!("{}", format!("Processing transaction {}/{}", idx + 1, transactions.len()).dimmed());
        
        let mut comparison = TransactionComparison {
            transaction_id: transaction.id,
            wallet: transaction.wallet.clone(),
            amount: extract_transaction_amount(transaction),
            bank: transaction.bank.as_ref()
                .map(|b| b.label.clone())
                .unwrap_or_else(|| "Unknown".to_string()),
            has_receipt: false,
            receipt_url: None,
            phone_or_card: extract_phone_or_card(transaction),
            receipt_amount: None,
            receipt_bank: None,
            receipt_phone_or_card: None,
            matches: false,
        };
        
        // Check for PDF receipt attachment
        if let Some(attachments) = &transaction.attachments {
            for attachment in attachments {
                if attachment.extension == "pdf" {
                    comparison.has_receipt = true;
                    let receipt_url = format!("https://cdn.gate.cx/{}", attachment.original_url);
                    comparison.receipt_url = Some(receipt_url.clone());
                    
                    // Download and process receipt
                    match download_and_process_receipt(&receipt_url, &pdf_parser).await {
                        Ok(receipt_info) => {
                            comparison.receipt_amount = Some(receipt_info.amount);
                            comparison.receipt_bank = receipt_info.bank_name;
                            comparison.receipt_phone_or_card = receipt_info.phone_number
                                .or(receipt_info.card_number);
                            
                            // Check if all fields match
                            let amount_matches = comparison.amount == receipt_info.amount;
                            let bank_matches = check_bank_match(&comparison.bank, 
                                &comparison.receipt_bank.as_ref().unwrap_or(&"".to_string()));
                            let phone_card_matches = match (&comparison.phone_or_card, &comparison.receipt_phone_or_card) {
                                (Some(t), Some(r)) => normalize_phone_or_card(t) == normalize_phone_or_card(r),
                                (None, None) => true,
                                _ => false,
                            };
                            
                            comparison.matches = amount_matches && bank_matches && phone_card_matches;
                        }
                        Err(e) => {
                            eprintln!("Failed to process receipt: {}", e);
                        }
                    }
                    
                    break; // Only process first PDF
                }
            }
        }
        
        comparisons.push(comparison);
        
        // Add a small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    // Print summary
    print_comparison_summary(&comparisons);
}

async fn download_and_process_receipt(
    url: &str, 
    parser: &PdfReceiptParser
) -> Result<itrader_backend::ocr::pdf::ReceiptInfo, Box<dyn std::error::Error>> {
    // Download PDF
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    
    // Save to temporary file
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(&bytes)?;
    temp_file.flush()?;
    
    // Parse PDF
    let receipt = parser.parse_receipt(temp_file.path()).await?;
    
    Ok(receipt)
}

fn extract_transaction_amount(transaction: &Payout) -> Decimal {
    // Try to get RUB amount from trader HashMap (code "643" or "RUB")
    for key in &["643", "RUB"] {
        if let Some(rub_value) = transaction.amount.trader.get(*key) {
            if let Ok(amount_num) = serde_json::from_value::<f64>(rub_value.clone()) {
                if let Some(amount) = Decimal::from_f64(amount_num) {
                    return amount;
                }
            }
            
            if let Ok(amount_str) = serde_json::from_value::<String>(rub_value.clone()) {
                if let Ok(amount) = Decimal::from_str(&amount_str) {
                    return amount;
                }
            }
        }
    }
    
    Decimal::ZERO
}

fn extract_phone_or_card(transaction: &Payout) -> Option<String> {
    let wallet = &transaction.wallet;
    
    // Check if wallet contains a phone number (11 digits starting with 7)
    if wallet.len() == 11 && wallet.starts_with('7') {
        return Some(format!("+{}", wallet));
    }
    
    // Check if wallet contains a card number (16 digits)
    if wallet.len() == 16 && wallet.chars().all(|c| c.is_digit(10)) {
        return Some(format!("****{}", &wallet[12..]));
    }
    
    // Check if wallet contains last 4 digits
    if wallet.len() == 4 && wallet.chars().all(|c| c.is_digit(10)) {
        return Some(format!("****{}", wallet));
    }
    
    None
}

fn check_bank_match(bank1: &str, bank2: &str) -> bool {
    let words1: Vec<String> = bank1.to_lowercase().split_whitespace()
        .map(|w| w.to_string())
        .collect();
    let words2: Vec<String> = bank2.to_lowercase().split_whitespace()
        .map(|w| w.to_string())
        .collect();
    
    // Check if any significant word matches
    for word1 in &words1 {
        if word1.len() < 3 { continue; }
        for word2 in &words2 {
            if word2.len() < 3 { continue; }
            if word1 == word2 || word1.contains(word2) || word2.contains(word1) {
                return true;
            }
        }
    }
    
    false
}

fn normalize_phone_or_card(value: &str) -> String {
    // Remove all non-digit characters
    let digits: String = value.chars().filter(|c| c.is_digit(10)).collect();
    
    // If it's a phone number (10-11 digits), normalize without country code
    if digits.len() == 11 && (digits.starts_with('7') || digits.starts_with('8')) {
        digits[1..].to_string()
    } else if digits.len() == 10 {
        digits
    } else if digits.len() == 4 {
        // Card last 4 digits
        digits
    } else {
        value.to_string()
    }
}

fn print_comparison_summary(comparisons: &[TransactionComparison]) {
    println!("\n{}", "=".repeat(100));
    println!("{}", "HISTORY TRANSACTIONS COMPARISON SUMMARY".bold());
    println!("{}", "=".repeat(100));
    
    let total = comparisons.len();
    let with_receipts = comparisons.iter().filter(|c| c.has_receipt).count();
    let matching = comparisons.iter().filter(|c| c.matches).count();
    
    println!("\n{}", "Statistics:".yellow());
    println!("  Total transactions: {}", total);
    println!("  With PDF receipts: {}", with_receipts);
    println!("  Fully matching: {} ({}%)", matching, 
        if with_receipts > 0 { matching * 100 / with_receipts } else { 0 });
    
    println!("\n{}", "Detailed Results:".yellow());
    println!("{}", "-".repeat(100));
    
    for comparison in comparisons {
        let status = if comparison.matches {
            "✓ MATCH".green()
        } else if comparison.has_receipt {
            "✗ MISMATCH".red()
        } else {
            "- NO RECEIPT".dimmed()
        };
        
        println!("\nTransaction {} [{}]", comparison.transaction_id, status);
        println!("  Wallet: {}", comparison.wallet);
        println!("  Amount: {} RUB", comparison.amount);
        println!("  Bank: {}", comparison.bank);
        
        if let Some(phone_or_card) = &comparison.phone_or_card {
            println!("  Phone/Card: {}", phone_or_card.cyan());
        }
        
        if comparison.has_receipt {
            println!("  {}", "Receipt Data:".green());
            if let Some(amount) = comparison.receipt_amount {
                let amount_status = if amount == comparison.amount { "✓" } else { "✗" };
                println!("    Amount: {} RUB {}", amount, amount_status);
            }
            if let Some(bank) = &comparison.receipt_bank {
                let bank_status = if check_bank_match(&comparison.bank, bank) { "✓" } else { "✗" };
                println!("    Bank: {} {}", bank, bank_status);
            }
            if let Some(phone_or_card) = &comparison.receipt_phone_or_card {
                let pc_status = match &comparison.phone_or_card {
                    Some(t) => if normalize_phone_or_card(t) == normalize_phone_or_card(phone_or_card) { "✓" } else { "✗" },
                    None => "✗"
                };
                println!("    Phone/Card: {} {}", phone_or_card, pc_status);
            }
            
            if let Some(url) = &comparison.receipt_url {
                println!("    Receipt URL: {}", url.dimmed());
            }
        }
    }
    
    println!("\n{}", "=".repeat(100));
}