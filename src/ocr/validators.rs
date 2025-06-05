use rust_decimal::Decimal;
use crate::utils::error::{Result, AppError};
use crate::ocr::processor::ReceiptData;
use regex::Regex;
use tracing::{debug, warn};

pub fn validate_tbank_receipt(text: &str) -> bool {
    text.contains("Т-Банк") || text.contains("Tinkoff") || text.contains("Тинькофф")
}

pub fn extract_amount(text: &str) -> Option<Decimal> {
    extract_amount_from_text(text, Decimal::new(0, 0)).ok()
}

pub fn validate_amount(ocr_amount: Decimal, expected_amount: Decimal) -> Result<bool> {
    // Allow small tolerance for OCR errors (0.01%)
    let tolerance = expected_amount * Decimal::new(1, 4); // 0.0001 = 0.01%
    let min_allowed = expected_amount - tolerance;
    let max_allowed = expected_amount + tolerance;
    
    debug!("Validating amount: OCR={}, Expected={}, Tolerance={}", ocr_amount, expected_amount, tolerance);
    
    Ok(ocr_amount >= min_allowed && ocr_amount <= max_allowed)
}

pub fn extract_amount_from_text(text: &str, expected_amount: Decimal) -> Result<Decimal> {
    debug!("Extracting amount from text, expecting: {}", expected_amount);
    
    // Normalize text: remove spaces in numbers
    let normalized = text.replace(" ", "");
    
    // Patterns for amount extraction
    let patterns = vec![
        // Russian format with spaces: 10 000,00 руб
        r"([\d\s]+)[,\.]?(\d{0,2})\s*(?:руб|₽|RUB)",
        // Format: 10000.00 RUB
        r"([\d]+)\.?(\d{0,2})\s*(?:руб|₽|RUB)",
        // Format with currency symbol: ₽ 10,000.00
        r"(?:₽|RUB)\s*([\d\s,]+)\.?(\d{0,2})",
        // Just numbers with common separators
        r"([\d]{1,3}(?:[\s,]\d{3})*)\.?(\d{0,2})",
        // Simple number format
        r"([\d]+)\.?(\d{0,2})",
    ];
    
    let mut found_amounts = Vec::new();
    
    for pattern in patterns {
        let re = Regex::new(pattern).unwrap();
        for captures in re.captures_iter(&normalized) {
            if let Some(whole_match) = captures.get(1) {
                let mut amount_str = whole_match.as_str()
                    .replace(" ", "")
                    .replace(",", "");
                
                // Add decimal part if captured
                if let Some(decimal_match) = captures.get(2) {
                    let decimal_str = decimal_match.as_str();
                    if !decimal_str.is_empty() {
                        amount_str.push('.');
                        amount_str.push_str(decimal_str);
                    }
                }
                
                // Try to parse the amount
                if let Ok(amount) = amount_str.parse::<Decimal>() {
                    // Filter out unlikely amounts (too small or too large)
                    if amount >= Decimal::new(100, 0) && amount <= Decimal::new(10000000, 0) {
                        found_amounts.push(amount);
                        debug!("Found potential amount: {}", amount);
                    }
                }
            }
        }
    }
    
    // Find the amount closest to expected
    if found_amounts.is_empty() {
        return Err(AppError::OcrError("No valid amount found in receipt".to_string()));
    }
    
    // If expected_amount is 0, just return the first found amount
    if expected_amount.is_zero() {
        return Ok(found_amounts[0]);
    }
    
    let mut best_match = found_amounts[0];
    let mut best_diff = (best_match - expected_amount).abs();
    
    for amount in found_amounts.iter().skip(1) {
        let diff = (*amount - expected_amount).abs();
        if diff < best_diff {
            best_match = *amount;
            best_diff = diff;
        }
    }
    
    debug!("Selected amount: {} (expected: {})", best_match, expected_amount);
    
    // Validate the amount is within acceptable range (10% tolerance for OCR)
    let max_tolerance = expected_amount * Decimal::new(1, 1); // 0.1 = 10%
    if best_diff > max_tolerance {
        warn!("Amount {} differs too much from expected {} (diff: {})", best_match, expected_amount, best_diff);
        return Err(AppError::OcrError(format!(
            "Amount {} differs too much from expected {}",
            best_match, expected_amount
        )));
    }
    
    Ok(best_match)
}

pub fn validate_receipt_data(receipt: &ReceiptData, expected_amount: Decimal) -> Result<()> {
    // Validate amount
    if !validate_amount(receipt.amount, expected_amount)? {
        return Err(AppError::ValidationError(format!(
            "Receipt amount {} doesn't match expected {}",
            receipt.amount, expected_amount
        )));
    }
    
    // Validate bank name
    if receipt.bank == "Unknown Bank" {
        warn!("Could not identify bank from receipt");
    }
    
    // Validate reference
    if receipt.reference.starts_with("AUTO-") {
        warn!("Using auto-generated reference: {}", receipt.reference);
    }
    
    // Validate timestamp (should be recent)
    let now = chrono::Utc::now();
    let age = now.signed_duration_since(receipt.timestamp);
    
    if age.num_days() > 7 {
        return Err(AppError::ValidationError(
            "Receipt is older than 7 days".to_string()
        ));
    }
    
    if age.num_seconds() < 0 {
        return Err(AppError::ValidationError(
            "Receipt timestamp is in the future".to_string()
        ));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_amount_from_text() {
        let test_cases = vec![
            ("10 000,00 руб", Decimal::new(10000, 0), Decimal::new(10000, 0)),
            ("₽ 5,500.50", Decimal::new(5500, 0), Decimal::new(55005, 1)),
            ("Amount: 1234.56 RUB", Decimal::new(1234, 0), Decimal::new(123456, 2)),
            ("Сумма: 999 рублей", Decimal::new(1000, 0), Decimal::new(999, 0)),
        ];
        
        for (text, expected, result) in test_cases {
            match extract_amount_from_text(text, expected) {
                Ok(amount) => assert_eq!(amount, result),
                Err(e) => panic!("Failed to extract amount from '{}': {}", text, e),
            }
        }
    }
    
    #[test]
    fn test_validate_amount() {
        assert!(validate_amount(Decimal::new(10000, 0), Decimal::new(10000, 0)).unwrap());
        assert!(validate_amount(Decimal::new(10001, 0), Decimal::new(10000, 0)).unwrap());
        assert!(validate_amount(Decimal::new(9999, 0), Decimal::new(10000, 0)).unwrap());
    }
}