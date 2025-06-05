use itrader_backend::gate::{GateClient, TransactionService, Payout};
use itrader_backend::ocr::pdf::PdfReceiptParser;
use itrader_backend::core::config::Config;
use itrader_backend::core::rate_limiter::RateLimiter;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::sync::Arc;
use colored::Colorize;
use std::str::FromStr;

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
    gate_account_id: Option<i64>,
    gate_account_email: Option<String>,
    transaction_date: Option<String>,
}

// Test function that accepts transaction ID and receipt filename as arguments
#[tokio::test]
async fn test_compare_transaction_with_receipt() {
    // Get arguments from environment variables (set these when running the test)
    let transaction_id = std::env::var("TRANSACTION_ID")
        .unwrap_or_else(|_| "2463024".to_string());
    let receipt_file = std::env::var("RECEIPT_FILE")
        .unwrap_or_else(|_| "test_data/receipt_27.05.2025.pdf".to_string());
    
    // Run the comparison
    let result = compare_transaction_with_receipt(&transaction_id, &receipt_file).await
        .expect("Failed to compare transaction with receipt");
    
    // Print detailed results
    print_comparison_results(&result);
}

async fn compare_transaction_with_receipt(
    transaction_id: &str,
    receipt_file: &str
) -> Result<OverallResult, Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::load()?;
    
    // Create Gate.io client
    let rate_limiter = Arc::new(RateLimiter::new(&config.rate_limits));
    let gate_client = GateClient::new(config.gate.base_url.clone(), rate_limiter.clone())?;
    
    // Load cookies if available
    let cookies_file = "test_data/gate_cookie.json";
    if std::path::Path::new(cookies_file).exists() {
        gate_client.load_cookies(cookies_file).await?;
    } else {
        return Err("Gate.io cookies not found. Please run gate-login test first.".into());
    }
    
    // Get account info (for now just set to None, as we need to extract from balance request)
    let (account_id, account_email) = (None, None);
    
    // Create transaction service
    let transaction_service = TransactionService::new(gate_client);
    
    // Fetch transaction
    let transaction = transaction_service.get_transaction(transaction_id).await?
        .ok_or_else(|| format!("Transaction {} not found", transaction_id))?;
    
    // Also try to get more detailed transaction info directly
    eprintln!("\n=== TRYING TO GET DETAILED TRANSACTION INFO ===");
    // Create a new client for the detailed fetch
    let gate_client2 = GateClient::new(config.gate.base_url.clone(), rate_limiter.clone())?;
    if std::path::Path::new(cookies_file).exists() {
        gate_client2.load_cookies(cookies_file).await?;
    }
    match gate_client2.get_transaction_details(transaction_id).await {
        Ok(detailed_transaction) => {
            eprintln!("Got detailed transaction info");
            eprintln!("Detailed transaction JSON:");
            match serde_json::to_string_pretty(&detailed_transaction) {
                Ok(json) => eprintln!("{}", json),
                Err(e) => eprintln!("Failed to serialize detailed transaction: {}", e),
            }
        },
        Err(e) => {
            eprintln!("Failed to get detailed transaction info: {}", e);
        }
    }
    eprintln!("=== END DETAILED TRANSACTION INFO ===");
    
    // Try to fetch attachment content if there are any
    if let Some(ref attachments) = transaction.attachments {
        eprintln!("\n=== CHECKING ATTACHMENTS FOR ADDITIONAL INFO ===");
        for (i, attachment) in attachments.iter().enumerate() {
            eprintln!("\nAttachment {}: {} ({})", i + 1, attachment.file_name, attachment.extension);
            eprintln!("URL: {}", attachment.original_url);
            
            // If it's an image, it might contain receipt info that needs OCR
            if attachment.extension == "jpg" || attachment.extension == "jpeg" || 
               attachment.extension == "png" || attachment.extension == "pdf" {
                eprintln!("This is an image/pdf attachment that might contain receipt details");
                eprintln!("Consider downloading and processing with OCR to extract phone/card info");
            }
        }
        eprintln!("=== END ATTACHMENTS CHECK ===");
    }
    
    // Debug: print ALL transaction fields comprehensively
    eprintln!("\n=== COMPREHENSIVE TRANSACTION DEBUG ===");
    eprintln!("Transaction ID: {}", transaction.id);
    eprintln!("Status: {}", transaction.status);
    eprintln!("Wallet: {}", transaction.wallet);
    eprintln!("Created at: {}", transaction.created_at);
    eprintln!("Updated at: {}", transaction.updated_at);
    eprintln!("Approved at: {:?}", transaction.approved_at);
    eprintln!("Expired at: {:?}", transaction.expired_at);
    eprintln!("Payment method ID: {:?}", transaction.payment_method_id);
    
    eprintln!("\n--- Amount (trader field) ---");
    for (key, value) in &transaction.amount.trader {
        eprintln!("  {}: {}", key, value);
    }
    
    eprintln!("\n--- Total (trader field) ---");
    for (key, value) in &transaction.total.trader {
        eprintln!("  {}: {}", key, value);
    }
    
    eprintln!("\n--- Method ---");
    if let Some(id) = transaction.method.id {
        eprintln!("  ID: {}", id);
    }
    eprintln!("  Type: {:?}", transaction.method.method_type);
    eprintln!("  Name: {:?}", transaction.method.name);
    eprintln!("  Label: {}", transaction.method.label);
    eprintln!("  Status: {:?}", transaction.method.status);
    eprintln!("  Payment provider ID: {:?}", transaction.method.payment_provider_id);
    eprintln!("  Wallet currency ID: {:?}", transaction.method.wallet_currency_id);
    
    eprintln!("\n--- Trader ---");
    if let Some(ref trader) = transaction.trader {
        eprintln!("  ID: {}", trader.id);
        eprintln!("  Name: {}", trader.name);
        eprintln!("  [Checking for phone in name]: {}", trader.name);
    } else {
        eprintln!("  Trader: None");
    }
    
    eprintln!("\n--- Bank ---");
    if let Some(ref bank) = transaction.bank {
        if let Some(id) = bank.id {
            eprintln!("  ID: {}", id);
        }
        eprintln!("  Name: {}", bank.name);
        eprintln!("  Code: {}", bank.code);
        eprintln!("  Label: {}", bank.label);
        eprintln!("  Active: {}", bank.active);
        eprintln!("  Meta: {:?}", bank.meta);
    } else {
        eprintln!("  Bank: None");
    }
    
    eprintln!("\n--- Meta ---");
    if let Some(ref meta) = transaction.meta {
        eprintln!("  Bank: {:?}", meta.bank);
        eprintln!("  Card number: {:?}", meta.card_number);
        eprintln!("  Courses: {:?}", meta.courses);
        if let Some(ref reason) = meta.reason {
            eprintln!("  Reason trader: {:?}", reason.trader);
            eprintln!("  Reason support: {:?}", reason.support);
        }
    } else {
        eprintln!("  Meta: None");
    }
    
    eprintln!("\n--- Attachments ---");
    if let Some(ref attachments) = transaction.attachments {
        eprintln!("  Number of attachments: {}", attachments.len());
        for (i, attachment) in attachments.iter().enumerate() {
            eprintln!("  Attachment {}:", i + 1);
            eprintln!("    Name: {}", attachment.name);
            eprintln!("    File name: {}", attachment.file_name);
            eprintln!("    Original URL: {}", attachment.original_url);
            eprintln!("    Extension: {}", attachment.extension);
            eprintln!("    Size: {} bytes", attachment.size);
            eprintln!("    Created at: {}", attachment.created_at);
            if let Some(ref props) = attachment.custom_properties {
                eprintln!("    Fake: {}", props.fake);
            }
        }
    } else {
        eprintln!("  Attachments: None");
    }
    
    eprintln!("\n--- Tooltip ---");
    if let Some(ref tooltip) = transaction.tooltip {
        if let Some(ref payments) = tooltip.payments {
            eprintln!("  Payments:");
            if let Some(success) = payments.success {
                eprintln!("    Success: {}", success);
            }
            if let Some(rejected) = payments.rejected {
                eprintln!("    Rejected: {}", rejected);
            }
            if let Some(percent) = payments.percent {
                eprintln!("    Percent: {}%", percent);
            }
        }
        eprintln!("  Reasons: {:?}", tooltip.reasons);
    } else {
        eprintln!("  Tooltip: None");
    }
    
    eprintln!("\n=== END TRANSACTION DEBUG ===");
    
    // Also print the raw JSON if we can get it
    eprintln!("\n=== RAW TRANSACTION JSON ===");
    match serde_json::to_string_pretty(&transaction) {
        Ok(json) => eprintln!("{}", json),
        Err(e) => eprintln!("Failed to serialize transaction to JSON: {}", e),
    }
    eprintln!("=== END RAW JSON ===");
    
    // Look for phone/card in all string fields - check various fields
    eprintln!("\n=== SEARCHING FOR PHONE/CARD IN ALL STRING FIELDS ===");
    let phone_regex = regex::Regex::new(r"(\+7|8)[\s\-]?\(?\d{3}\)?[\s\-]?\d{3}[\s\-]?\d{2}[\s\-]?\d{2}").unwrap();
    let card_regex = regex::Regex::new(r"\*+\s*\d{4}|\d{4}\s*\d{4}\s*\d{4}\s*\d{4}").unwrap();
    
    // Check tooltip reasons
    if let Some(ref tooltip) = transaction.tooltip {
        for (i, reason) in tooltip.reasons.iter().enumerate() {
            eprintln!("\nChecking tooltip reason[{}]: {}", i, reason);
            if let Some(phone_match) = phone_regex.find(reason) {
                eprintln!("  Found phone pattern: {}", phone_match.as_str());
            }
            if let Some(card_match) = card_regex.find(reason) {
                eprintln!("  Found card pattern: {}", card_match.as_str());
            }
        }
    }
    
    // Check meta.courses field which might contain additional data
    if let Some(ref meta) = transaction.meta {
        if let Some(ref courses) = meta.courses {
            eprintln!("\nChecking meta.courses field: {:?}", courses);
        }
        
        // Check meta.reason fields
        if let Some(ref reason) = meta.reason {
            if let Some(ref trader_reason) = reason.trader {
                eprintln!("\nChecking meta.reason.trader: {}", trader_reason);
                if let Some(phone_match) = phone_regex.find(trader_reason) {
                    eprintln!("  Found phone pattern: {}", phone_match.as_str());
                }
                if let Some(card_match) = card_regex.find(trader_reason) {
                    eprintln!("  Found card pattern: {}", card_match.as_str());
                }
            }
            if let Some(ref support_reason) = reason.support {
                eprintln!("\nChecking meta.reason.support: {}", support_reason);
                if let Some(phone_match) = phone_regex.find(support_reason) {
                    eprintln!("  Found phone pattern: {}", phone_match.as_str());
                }
                if let Some(card_match) = card_regex.find(support_reason) {
                    eprintln!("  Found card pattern: {}", card_match.as_str());
                }
            }
        }
    }
    eprintln!("=== END PHONE/CARD SEARCH ===");
    
    // Parse receipt
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
    
    // Determine overall match
    let overall_match = comparisons.iter().all(|c| c.matches);
    
    // Get transaction date
    let transaction_date = Some(transaction.created_at.clone());
    
    Ok(OverallResult {
        transaction_id: transaction_id.to_string(),
        receipt_file: receipt_file.to_string(),
        comparisons,
        overall_match,
        gate_account_id: account_id,
        gate_account_email: account_email,
        transaction_date,
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
    // Extract phone from transaction metadata
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
        transaction_value: transaction_phone.clone().unwrap_or_else(|| "Not found (may be in attachments)".to_string()),
        receipt_value: receipt.phone_number.clone().unwrap_or_else(|| "Not found".to_string()),
        matches,
        details: if !matches && transaction_phone.is_none() && receipt.phone_number.is_some() {
            Some("Phone may be in transaction attachments".to_string())
        } else if !matches && transaction_phone.is_some() && receipt_phone.is_some() {
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
        transaction_value: transaction_card.clone().unwrap_or_else(|| "Not found (may be in attachments)".to_string()),
        receipt_value: receipt_card.clone().unwrap_or_else(|| "Not found".to_string()),
        matches,
        details: if !matches && transaction_card.is_none() && receipt_card.is_some() {
            Some("Card may be in transaction attachments".to_string())
        } else if !matches && transaction_card.is_some() && receipt_card.is_some() {
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

// Helper functions
fn extract_transaction_amount(transaction: &Payout) -> Decimal {
    // Try to get RUB amount from trader HashMap (code "643" or "RUB")
    for key in &["643", "RUB"] {
        if let Some(rub_value) = transaction.amount.trader.get(*key) {
            // Try to parse as number first
            if let Ok(amount_num) = serde_json::from_value::<f64>(rub_value.clone()) {
                if let Some(amount) = Decimal::from_f64(amount_num) {
                    return amount;
                }
            }
            
            // Try to parse as string
            if let Ok(amount_str) = serde_json::from_value::<String>(rub_value.clone()) {
                if let Ok(amount) = Decimal::from_str(&amount_str) {
                    return amount;
                }
            }
        }
    }
    
    // Try total amount
    for key in &["643", "RUB"] {
        if let Some(rub_value) = transaction.total.trader.get(*key) {
            // Try to parse as number first
            if let Ok(amount_num) = serde_json::from_value::<f64>(rub_value.clone()) {
                if let Some(amount) = Decimal::from_f64(amount_num) {
                    return amount;
                }
            }
            
            // Try to parse as string
            if let Ok(amount_str) = serde_json::from_value::<String>(rub_value.clone()) {
                if let Ok(amount) = Decimal::from_str(&amount_str) {
                    return amount;
                }
            }
        }
    }
    
    Decimal::ZERO
}

fn extract_transaction_phone(transaction: &Payout) -> Option<String> {
    // Check wallet field - it contains phone number for SBP (СБП) transactions
    let wallet = &transaction.wallet;
    eprintln!("DEBUG: Checking wallet field for phone: {}", wallet);
    
    // Check if wallet contains a phone number (11 digits starting with 7)
    if wallet.len() == 11 && wallet.starts_with('7') {
        // This is a phone number, format it
        let formatted = format!("+{}", wallet);
        eprintln!("DEBUG: Found phone in wallet field: {}", formatted);
        return Some(formatted);
    }
    
    // Check if it's already formatted with +7
    if wallet.starts_with("+7") && wallet.len() == 12 {
        eprintln!("DEBUG: Found formatted phone in wallet field: {}", wallet);
        return Some(wallet.clone());
    }
    
    // First check meta field for phone
    if let Some(ref meta) = transaction.meta {
        // Meta doesn't have phone field directly, but check bank field
        if let Some(ref bank) = meta.bank {
            eprintln!("DEBUG: Checking meta.bank for phone: {}", bank);
        }
    }
    
    // Extract phone from trader name field
    if let Some(trader) = &transaction.trader {
        eprintln!("DEBUG: Searching for phone in trader name: '{}'", trader.name);
        
        let phone_regex = regex::Regex::new(r"(\+7|8)[\s\-]?\(?\d{3}\)?[\s\-]?\d{3}[\s\-]?\d{2}[\s\-]?\d{2}").ok()?;
        if let Some(captures) = phone_regex.find(&trader.name) {
            eprintln!("DEBUG: Found phone in trader name: {}", captures.as_str());
            return Some(captures.as_str().to_string());
        }
    }
    
    // Note: Phone might be in wallet field for some transaction types
    eprintln!("DEBUG: Phone not found. Wallet field contains: {}", wallet);
    
    None
}

fn extract_transaction_card(transaction: &Payout) -> Option<String> {
    // Check wallet field - it might contain card number for card transactions
    let wallet = &transaction.wallet;
    eprintln!("DEBUG: Checking wallet field for card: {}", wallet);
    
    // Check if wallet contains a card number (16 digits)
    if wallet.len() == 16 && wallet.chars().all(|c| c.is_digit(10)) {
        // This is a full card number, mask it
        let masked = format!("****{}", &wallet[12..]);
        eprintln!("DEBUG: Found card in wallet field: {}", masked);
        return Some(masked);
    }
    
    // Check if wallet contains last 4 digits (common pattern)
    if wallet.len() == 4 && wallet.chars().all(|c| c.is_digit(10)) {
        let masked = format!("****{}", wallet);
        eprintln!("DEBUG: Found card last 4 in wallet field: {}", masked);
        return Some(masked);
    }
    
    // First check meta.card_number
    if let Some(ref meta) = transaction.meta {
        eprintln!("DEBUG: Meta card_number field: {:?}", meta.card_number);
        if let Some(ref card) = meta.card_number {
            return Some(card.clone());
        }
    }
    
    // Check if card info might be in bank label or name
    if let Some(ref bank) = transaction.bank {
        eprintln!("DEBUG: Checking bank label for card: {}", bank.label);
        
        // Look for patterns like "**** 1234" or "************1234"
        let card_regex = regex::Regex::new(r"\*+\s*\d{4}").ok()?;
        if let Some(captures) = card_regex.find(&bank.label) {
            eprintln!("DEBUG: Found card pattern in bank label: {}", captures.as_str());
            return Some(captures.as_str().to_string());
        }
    }
    
    // Note: For SBP transactions, wallet contains phone; for card transactions, it might contain card info
    eprintln!("DEBUG: Card not found. Wallet field contains: {}", wallet);
    
    None
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
    
    // Print account info
    if let Some(account_id) = result.gate_account_id {
        println!("{}: {}", "Gate Account ID".cyan(), account_id);
    }
    if let Some(ref email) = result.gate_account_email {
        println!("{}: {}", "Gate Account Email".cyan(), email);
    }
    
    // Print transaction date
    if let Some(ref date) = result.transaction_date {
        println!("{}: {}", "Transaction Date".cyan(), date);
    }
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

#[cfg(test)]
mod test_helpers {
    use super::*;
    
    #[test]
    fn test_normalize_phone_number() {
        assert_eq!(normalize_phone_number("+7 914 218 48 44"), "9142184844");
        assert_eq!(normalize_phone_number("8 914 218 48 44"), "9142184844");
        assert_eq!(normalize_phone_number("914-218-48-44"), "9142184844");
        assert_eq!(normalize_phone_number("+7(914)218-48-44"), "9142184844");
    }
    
    #[test]
    fn test_extract_last_4_digits() {
        assert_eq!(extract_last_4_digits("2200 7011 1756 3558"), "3558");
        assert_eq!(extract_last_4_digits("************3558"), "3558");
        assert_eq!(extract_last_4_digits("**** **** **** 3558"), "3558");
    }
    
    #[test]
    fn test_bank_name_matching() {
        assert!(check_bank_name_match("Т-Банк (Тинькофф)", "T-Bank"));
        assert!(check_bank_name_match("T-Банк (Тинькофф) (MIR)", "Тинькофф"));
        assert!(check_bank_name_match("Озон Банк (Ozon)", "Ozon Bank"));
        assert!(check_bank_name_match("Сбербанк", "Sber"));
        assert!(!check_bank_name_match("Альфа-Банк", "ВТБ"));
    }
}