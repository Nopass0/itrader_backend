use itrader_backend::gate::Payout;
use itrader_backend::ocr::pdf::{PdfReceiptParser, ReceiptInfo};
use rust_decimal::Decimal;
use std::collections::HashMap;
use colored::Colorize;
use chrono::{DateTime, Utc};

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

// Mock test with sample data
#[tokio::test]
async fn test_compare_transaction_with_receipt_mock() {
    // Create mock transaction data based on the examples provided
    let mut amount_trader = HashMap::new();
    amount_trader.insert("RUB".to_string(), serde_json::json!("8256"));
    
    let mut total_trader = HashMap::new();
    total_trader.insert("RUB".to_string(), serde_json::json!("8437.63"));
    
    let mock_transaction = Payout {
        id: 2463024,
        payment_method_id: Some(1),
        status: 5,
        wallet: "USDT".to_string(),
        amount: itrader_backend::gate::PayoutAmount { trader: amount_trader },
        total: itrader_backend::gate::PayoutAmount { trader: total_trader },
        method: itrader_backend::gate::PaymentMethod {
            id: Some(1),
            method_type: Some(1),
            name: Some(1),
            label: "Озон Банк".to_string(),
            status: Some(1),
            payment_provider_id: Some(1),
            wallet_currency_id: Some(1),
        },
        meta: Some(itrader_backend::gate::PayoutMeta {
            bank: Some("Озон Банк (Ozon)".to_string()),
            card_number: None,
            courses: None,
            reason: None,
        }),
        approved_at: Some("2025-05-29T12:20:00Z".to_string()),
        expired_at: None,
        created_at: "2025-05-29T12:20:00Z".to_string(),
        updated_at: "2025-05-29T12:21:00Z".to_string(),
        attachments: None,
        trader: Some(itrader_backend::gate::Trader {
            id: 123,
            name: "+7 914 218 48 44".to_string(),
        }),
        bank: Some(itrader_backend::gate::Bank {
            id: Some(1),
            name: "Озон Банк".to_string(),
            code: "OZON".to_string(),
            label: "Озон Банк (Ozon)".to_string(),
            active: true,
            meta: None,
        }),
        tooltip: None,
    };
    
    // Create mock receipt data
    let mock_receipt = ReceiptInfo {
        date_time: DateTime::parse_from_rfc3339("2025-05-29T12:20:00Z").unwrap().with_timezone(&Utc),
        amount: Decimal::from(8256),
        recipient: Some("Получатель платежа".to_string()),
        sender: Some("Отправитель".to_string()),
        transaction_id: Some("2463024".to_string()),
        bank_name: Some("Озон Банк".to_string()),
        card_number: None,
        phone_number: Some("+7 914 218 48 44".to_string()),
        status: Some("Выполнен".to_string()),
        raw_text: "Mock receipt text".to_string(),
    };
    
    // Perform comparisons
    let mut comparisons = Vec::new();
    
    // Compare amounts
    comparisons.push(compare_amounts(&mock_transaction, &mock_receipt));
    
    // Compare phone numbers
    comparisons.push(compare_phone_numbers(&mock_transaction, &mock_receipt));
    
    // Compare card numbers
    comparisons.push(compare_card_numbers(&mock_transaction, &mock_receipt));
    
    // Compare bank names
    comparisons.push(compare_bank_names(&mock_transaction, &mock_receipt));
    
    // Determine overall match
    let overall_match = comparisons.iter().all(|c| c.matches);
    
    let result = OverallResult {
        transaction_id: "2463024".to_string(),
        receipt_file: "mock_receipt.pdf".to_string(),
        comparisons,
        overall_match,
    };
    
    // Print detailed results
    print_comparison_results(&result);
    
    // Assertions
    assert!(result.overall_match, "Transaction and receipt should match");
}

// Test with card number example
#[tokio::test]
async fn test_compare_transaction_with_card_mock() {
    // Create mock transaction data with card number
    let mut amount_trader = HashMap::new();
    amount_trader.insert("RUB".to_string(), serde_json::json!("37152"));
    
    let mut total_trader = HashMap::new();
    total_trader.insert("RUB".to_string(), serde_json::json!("37969.34"));
    
    let mock_transaction = Payout {
        id: 2436856,
        payment_method_id: Some(2),
        status: 5,
        wallet: "USDT".to_string(),
        amount: itrader_backend::gate::PayoutAmount { trader: amount_trader },
        total: itrader_backend::gate::PayoutAmount { trader: total_trader },
        method: itrader_backend::gate::PaymentMethod {
            id: Some(2),
            method_type: Some(1),
            name: Some(1),
            label: "T-Банк".to_string(),
            status: Some(1),
            payment_provider_id: Some(1),
            wallet_currency_id: Some(1),
        },
        meta: Some(itrader_backend::gate::PayoutMeta {
            bank: Some("T-Банк (Тинькофф) (MIR)".to_string()),
            card_number: Some("2200 7011 1756 3558".to_string()),
            courses: None,
            reason: None,
        }),
        approved_at: Some("2025-05-27T12:14:00Z".to_string()),
        expired_at: Some("2025-05-27T12:40:00Z".to_string()),
        created_at: "2025-05-27T12:14:00Z".to_string(),
        updated_at: "2025-05-27T12:40:00Z".to_string(),
        attachments: None,
        trader: None,
        bank: Some(itrader_backend::gate::Bank {
            id: Some(2),
            name: "T-Банк".to_string(),
            code: "TBANK".to_string(),
            label: "T-Банк (Тинькофф) (MIR)".to_string(),
            active: true,
            meta: None,
        }),
        tooltip: None,
    };
    
    // Create mock receipt data with masked card
    let mock_receipt = ReceiptInfo {
        date_time: DateTime::parse_from_rfc3339("2025-05-27T12:30:00Z").unwrap().with_timezone(&Utc),
        amount: Decimal::from(37152),
        recipient: Some("Получатель платежа".to_string()),
        sender: Some("Отправитель".to_string()),
        transaction_id: Some("2436856".to_string()),
        bank_name: Some("Тинькофф".to_string()),
        card_number: Some("************3558".to_string()),
        phone_number: None,
        status: Some("Выполнен".to_string()),
        raw_text: "Mock receipt text".to_string(),
    };
    
    // Perform comparisons
    let mut comparisons = Vec::new();
    
    comparisons.push(compare_amounts(&mock_transaction, &mock_receipt));
    comparisons.push(compare_phone_numbers(&mock_transaction, &mock_receipt));
    comparisons.push(compare_card_numbers(&mock_transaction, &mock_receipt));
    comparisons.push(compare_bank_names(&mock_transaction, &mock_receipt));
    
    let overall_match = comparisons.iter()
        .filter(|c| c.field != "Date/Time")
        .all(|c| c.matches);
    
    let result = OverallResult {
        transaction_id: "2436856".to_string(),
        receipt_file: "mock_receipt_card.pdf".to_string(),
        comparisons,
        overall_match,
    };
    
    print_comparison_results(&result);
    
    // Assertions
    assert!(result.overall_match, "Transaction and receipt should match");
    
    // Specific assertions
    let card_comparison = result.comparisons.iter()
        .find(|c| c.field == "Card Number")
        .expect("Card number comparison should exist");
    assert!(card_comparison.matches, "Card numbers should match (last 4 digits)");
}

// Helper functions (same as in the original file)
fn compare_amounts(transaction: &Payout, receipt: &ReceiptInfo) -> ComparisonResult {
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

fn compare_phone_numbers(transaction: &Payout, receipt: &ReceiptInfo) -> ComparisonResult {
    let transaction_phone = extract_transaction_phone(transaction);
    let receipt_phone = receipt.phone_number.as_ref().map(|p| normalize_phone_number(p));
    
    let matches = match (&transaction_phone, &receipt_phone) {
        (Some(t_phone), Some(r_phone)) => {
            normalize_phone_number(t_phone) == *r_phone
        },
        (None, None) => true,
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

fn compare_card_numbers(transaction: &Payout, receipt: &ReceiptInfo) -> ComparisonResult {
    let transaction_card = extract_transaction_card(transaction);
    let receipt_card = receipt.card_number.clone();
    
    let matches = match (&transaction_card, &receipt_card) {
        (Some(t_card), Some(r_card)) => {
            let t_last4 = extract_last_4_digits(t_card);
            let r_last4 = extract_last_4_digits(r_card);
            t_last4 == r_last4
        },
        (None, None) => true,
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

fn compare_bank_names(transaction: &Payout, receipt: &ReceiptInfo) -> ComparisonResult {
    let transaction_bank = transaction.bank.as_ref()
        .map(|b| b.label.clone())
        .or_else(|| transaction.meta.as_ref()?.bank.clone())
        .unwrap_or_else(|| "Unknown".to_string());
    
    let receipt_bank = receipt.bank_name.clone().unwrap_or_else(|| "Unknown".to_string());
    
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

fn extract_transaction_amount(transaction: &Payout) -> Decimal {
    if let Some(rub_value) = transaction.amount.trader.get("RUB") {
        if let Ok(amount_str) = serde_json::from_value::<String>(rub_value.clone()) {
            if let Ok(amount) = Decimal::from_str_exact(&amount_str) {
                return amount;
            }
        }
    }
    
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
    if let Some(trader) = &transaction.trader {
        let name = &trader.name;
        let phone_regex = regex::Regex::new(r"(?:\+7|8)?[\s\-]?\(?(\d{3})\)?[\s\-]?(\d{3})[\s\-]?(\d{2})[\s\-]?(\d{2})").unwrap();
        if let Some(captures) = phone_regex.captures(name) {
            return Some(captures.get(0).unwrap().as_str().to_string());
        }
    }
    None
}

fn extract_transaction_card(transaction: &Payout) -> Option<String> {
    transaction.meta.as_ref()?.card_number.clone()
}

fn normalize_phone_number(phone: &str) -> String {
    let digits: String = phone.chars().filter(|c| c.is_digit(10)).collect();
    
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
    
    for word1 in &words1 {
        if word1.len() < 3 { continue; }
        for word2 in &words2 {
            if word2.len() < 3 { continue; }
            if word1 == word2 || word1.contains(word2) || word2.contains(word1) {
                return true;
            }
        }
    }
    
    let normalized1 = normalize_bank_name(bank1);
    let normalized2 = normalize_bank_name(bank2);
    
    normalized1 == normalized2
}

fn normalize_bank_name(bank: &str) -> String {
    let lower = bank.to_lowercase();
    
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
    
    println!("{}", "=".repeat(80));
    let overall_status = if result.overall_match {
        "✓ ALL FIELDS MATCH".green().bold()
    } else {
        "✗ FIELDS DO NOT MATCH".red().bold()
    };
    println!("OVERALL RESULT: {}", overall_status);
    println!("{}", "=".repeat(80));
}