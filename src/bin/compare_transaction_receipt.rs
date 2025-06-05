use itrader_backend::gate::{GateClient, TransactionService, Payout};
use itrader_backend::ocr::pdf::PdfReceiptParser;
use itrader_backend::core::config::Config;
use itrader_backend::core::rate_limiter::RateLimiter;
use rust_decimal::Decimal;
use std::sync::Arc;
use colored::Colorize;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "Compare Gate.io transaction with receipt PDF", long_about = None)]
struct Args {
    /// Transaction ID to compare
    #[arg(short, long)]
    transaction_id: String,
    
    /// Path to receipt PDF file
    #[arg(short, long)]
    receipt_file: String,
}

#[derive(Debug)]
struct ComparisonResult {
    field: String,
    transaction_value: String,
    receipt_value: String,
    matches: bool,
    details: Option<String>,
}

#[derive(Debug)]
struct OverallResult {
    transaction_id: String,
    receipt_file: String,
    comparisons: Vec<ComparisonResult>,
    overall_match: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("Comparing transaction {} with receipt {}", 
             args.transaction_id.cyan(), 
             args.receipt_file.cyan());
    
    // Run the comparison
    let result = compare_transaction_with_receipt(&args.transaction_id, &args.receipt_file).await?;
    
    // Print detailed results
    print_comparison_results(&result);
    
    // Exit with appropriate code
    std::process::exit(if result.overall_match { 0 } else { 1 });
}

async fn compare_transaction_with_receipt(
    transaction_id: &str,
    receipt_file: &str
) -> Result<OverallResult, Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::load()?;
    
    // Create Gate.io client
    let rate_limiter = Arc::new(RateLimiter::new(&config.rate_limits));
    let gate_client = GateClient::new(config.gate.base_url.clone(), rate_limiter)?;
    
    // Load cookies if available
    let cookies_file = "test_data/gate_cookies.json";
    if std::path::Path::new(cookies_file).exists() {
        println!("Loading Gate.io cookies...");
        gate_client.load_cookies(cookies_file).await?;
    } else {
        return Err("Gate.io cookies not found. Please run gate-login test first.".into());
    }
    
    // Create transaction service
    let transaction_service = TransactionService::new(gate_client);
    
    // Fetch transaction
    println!("Fetching transaction from Gate.io...");
    let transaction = transaction_service.get_transaction(transaction_id).await?
        .ok_or_else(|| format!("Transaction {} not found", transaction_id))?;
    
    // Parse receipt
    println!("Parsing receipt PDF...");
    let parser = PdfReceiptParser::new();
    let receipt = parser.parse_receipt(receipt_file).await?;
    
    // Perform comparisons
    let mut comparisons = Vec::new();
    
    // Compare amounts
    comparisons.push(compare_amounts(&transaction, &receipt));
    
    // Compare phone numbers
    comparisons.push(compare_phone_numbers(&transaction, &receipt));
    
    // Compare card numbers
    comparisons.push(compare_card_numbers(&transaction, &receipt));
    
    // Compare bank names
    comparisons.push(compare_bank_names(&transaction, &receipt));
    
    // Compare dates
    comparisons.push(compare_dates(&transaction, &receipt));
    
    // Determine overall match
    let overall_match = comparisons.iter()
        .filter(|c| c.field != "Date/Time") // Date might not always match exactly
        .all(|c| c.matches);
    
    Ok(OverallResult {
        transaction_id: transaction_id.to_string(),
        receipt_file: receipt_file.to_string(),
        comparisons,
        overall_match,
    })
}

fn compare_amounts(transaction: &Payout, receipt: &itrader_backend::ocr::pdf::ReceiptInfo) -> ComparisonResult {
    // Extract amount from transaction
    let transaction_amount = extract_transaction_amount(transaction);
    let receipt_amount = receipt.amount;
    
    let matches = transaction_amount == receipt_amount;
    
    ComparisonResult {
        field: "Amount".to_string(),
        transaction_value: format!("{} RUB", transaction_amount),
        receipt_value: format!("{} RUB", receipt_amount),
        matches,
        details: if !matches {
            Some(format!("Difference: {} RUB", (transaction_amount - receipt_amount).abs()))
        } else {
            None
        },
    }
}

fn compare_phone_numbers(transaction: &Payout, receipt: &itrader_backend::ocr::pdf::ReceiptInfo) -> ComparisonResult {
    // Extract phone from transaction - check in meta and trader name
    let transaction_phone = extract_transaction_phone(transaction);
    let receipt_phone = receipt.phone_number.as_ref().map(|p| normalize_phone_number(p));
    
    let matches = match (&transaction_phone, &receipt_phone) {
        (Some(t_phone), Some(r_phone)) => {
            normalize_phone_number(t_phone) == *r_phone
        },
        (None, None) => true, // Both missing
        _ => false,
    };
    
    ComparisonResult {
        field: "Phone Number".to_string(),
        transaction_value: transaction_phone.clone().unwrap_or_else(|| "Not found".to_string()),
        receipt_value: receipt.phone_number.clone().unwrap_or_else(|| "Not found".to_string()),
        matches,
        details: if !matches && transaction_phone.is_some() && receipt_phone.is_some() {
            Some("Phone numbers normalized for comparison".to_string())
        } else {
            None
        },
    }
}

fn compare_card_numbers(transaction: &Payout, receipt: &itrader_backend::ocr::pdf::ReceiptInfo) -> ComparisonResult {
    // Extract card number from transaction
    let transaction_card = extract_transaction_card(transaction);
    let receipt_card = receipt.card_number.clone();
    
    let matches = match (&transaction_card, &receipt_card) {
        (Some(t_card), Some(r_card)) => {
            // Compare last 4 digits
            let t_last4 = extract_last_4_digits(t_card);
            let r_last4 = extract_last_4_digits(r_card);
            t_last4 == r_last4
        },
        (None, None) => true, // Both missing
        _ => false,
    };
    
    ComparisonResult {
        field: "Card Number".to_string(),
        transaction_value: transaction_card.clone().unwrap_or_else(|| "Not found".to_string()),
        receipt_value: receipt_card.clone().unwrap_or_else(|| "Not found".to_string()),
        matches,
        details: if !matches && transaction_card.is_some() && receipt_card.is_some() {
            Some("Comparing last 4 digits only".to_string())
        } else {
            None
        },
    }
}

fn compare_bank_names(transaction: &Payout, receipt: &itrader_backend::ocr::pdf::ReceiptInfo) -> ComparisonResult {
    // Extract bank name from transaction
    let transaction_bank = transaction.bank.as_ref()
        .map(|b| b.label.clone())
        .or_else(|| transaction.meta.as_ref()?.bank.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    
    let receipt_bank = receipt.bank_name.clone().unwrap_or_else(|| "Unknown".to_string());
    
    // Check if any word matches
    let matches = check_bank_name_match(&transaction_bank, &receipt_bank);
    
    ComparisonResult {
        field: "Bank Name".to_string(),
        transaction_value: transaction_bank,
        receipt_value: receipt_bank,
        matches,
        details: if !matches {
            Some("No matching words found in bank names".to_string())
        } else {
            None
        },
    }
}

fn compare_dates(transaction: &Payout, receipt: &itrader_backend::ocr::pdf::ReceiptInfo) -> ComparisonResult {
    // Parse transaction date
    let transaction_date = transaction.created_at.clone();
    let receipt_date = receipt.date_time.format("%d.%m.%Y %H:%M").to_string();
    
    // Check if dates are on the same day
    let t_date = chrono::DateTime::parse_from_rfc3339(&transaction.created_at)
        .map(|d| d.format("%d.%m.%Y").to_string())
        .unwrap_or_else(|_| transaction_date.clone());
    
    let r_date = receipt.date_time.format("%d.%m.%Y").to_string();
    
    let matches = t_date == r_date;
    
    ComparisonResult {
        field: "Date/Time".to_string(),
        transaction_value: transaction_date,
        receipt_value: receipt_date,
        matches,
        details: if !matches {
            Some("Dates are on different days".to_string())
        } else {
            None
        },
    }
}

// Helper functions
fn extract_transaction_amount(transaction: &Payout) -> Decimal {
    // Try to get RUB amount from trader HashMap
    if let Some(rub_value) = transaction.amount.trader.get("RUB") {
        if let Ok(amount_str) = serde_json::from_value::<String>(rub_value.clone()) {
            if let Ok(amount) = Decimal::from_str_exact(&amount_str) {
                return amount;
            }
        }
    }
    
    // Try total amount
    if let Some(rub_value) = transaction.total.trader.get("RUB") {
        if let Ok(amount_str) = serde_json::from_value::<String>(rub_value.clone()) {
            if let Ok(amount) = Decimal::from_str_exact(&amount_str) {
                return amount;
            }
        }
    }
    
    Decimal::ZERO
}

fn extract_transaction_phone(transaction: &Payout) -> Option<String> {
    // Check if trader name contains a phone number
    if let Some(trader) = &transaction.trader {
        let name = &trader.name;
        // Extract phone number from trader name using regex
        let phone_regex = regex::Regex::new(r"(?:\+7|8)?[\s\-]?\(?(\d{3})\)?[\s\-]?(\d{3})[\s\-]?(\d{2})[\s\-]?(\d{2})").unwrap();
        if let Some(captures) = phone_regex.captures(name) {
            return Some(captures.get(0).unwrap().as_str().to_string());
        }
    }
    
    // Check in meta fields
    None
}

fn extract_transaction_card(transaction: &Payout) -> Option<String> {
    transaction.meta.as_ref()?.card_number.clone()
}

fn normalize_phone_number(phone: &str) -> String {
    // Remove all non-digit characters
    let digits: String = phone.chars().filter(|c| c.is_digit(10)).collect();
    
    // Normalize to format without country code
    if digits.len() == 11 && (digits.starts_with('7') || digits.starts_with('8')) {
        digits[1..].to_string()
    } else {
        digits
    }
}

fn extract_last_4_digits(card: &str) -> String {
    let digits: String = card.chars().filter(|c| c.is_digit(10)).collect();
    if digits.len() >= 4 {
        digits[digits.len() - 4..].to_string()
    } else {
        digits
    }
}

fn check_bank_name_match(bank1: &str, bank2: &str) -> bool {
    let words1: Vec<String> = bank1.to_lowercase().split_whitespace()
        .map(|w| w.to_string())
        .collect();
    let words2: Vec<String> = bank2.to_lowercase().split_whitespace()
        .map(|w| w.to_string())
        .collect();
    
    // Check if any significant word matches
    for word1 in &words1 {
        if word1.len() < 3 { continue; } // Skip short words
        for word2 in &words2 {
            if word2.len() < 3 { continue; }
            if word1 == word2 || word1.contains(word2) || word2.contains(word1) {
                return true;
            }
        }
    }
    
    // Check common bank name variations
    let normalized1 = normalize_bank_name(bank1);
    let normalized2 = normalize_bank_name(bank2);
    
    normalized1 == normalized2
}

fn normalize_bank_name(bank: &str) -> String {
    let lower = bank.to_lowercase();
    
    // Map common variations to canonical names
    if lower.contains("тинькофф") || lower.contains("tinkoff") || lower.contains("т-банк") || lower.contains("t-bank") {
        "tbank".to_string()
    } else if lower.contains("сбер") || lower.contains("sber") {
        "sber".to_string()
    } else if lower.contains("альфа") || lower.contains("alfa") || lower.contains("alpha") {
        "alfa".to_string()
    } else if lower.contains("втб") || lower.contains("vtb") {
        "vtb".to_string()
    } else if lower.contains("райф") || lower.contains("raif") {
        "raiffeisen".to_string()
    } else if lower.contains("озон") || lower.contains("ozon") {
        "ozon".to_string()
    } else {
        lower.replace(" ", "").replace("-", "").replace("банк", "").replace("bank", "")
    }
}

fn print_comparison_results(result: &OverallResult) {
    println!("\n{}", "=".repeat(80));
    println!("{}", "TRANSACTION vs RECEIPT COMPARISON RESULTS".bold());
    println!("{}", "=".repeat(80));
    
    println!("\n{}: {}", "Transaction ID".cyan(), result.transaction_id);
    println!("{}: {}", "Receipt File".cyan(), result.receipt_file);
    println!();
    
    // Print each comparison
    for comparison in &result.comparisons {
        let status = if comparison.matches {
            "✓ MATCH".green()
        } else {
            "✗ MISMATCH".red()
        };
        
        println!("{} [{}]", comparison.field.yellow(), status);
        println!("  Transaction: {}", comparison.transaction_value);
        println!("  Receipt:     {}", comparison.receipt_value);
        
        if let Some(details) = &comparison.details {
            println!("  Details:     {}", details.dimmed());
        }
        println!();
    }
    
    // Print overall result
    println!("{}", "=".repeat(80));
    let overall_status = if result.overall_match {
        "✓ ALL FIELDS MATCH".green().bold()
    } else {
        "✗ FIELDS DO NOT MATCH".red().bold()
    };
    println!("OVERALL RESULT: {}", overall_status);
    println!("{}", "=".repeat(80));
}