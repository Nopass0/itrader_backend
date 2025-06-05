use rust_decimal::Decimal;
use crate::utils::error::{Result, AppError};
use crate::ocr::pdf::extract_text_from_pdf;
use crate::ocr::validators::{validate_receipt_data, extract_amount_from_text};
use std::process::Command;
use std::fs;
use std::path::Path;
use tracing::{info, debug, warn, error};
use uuid::Uuid;
use regex::Regex;
use chrono::{DateTime, Utc, NaiveDateTime};
use tempfile::NamedTempFile;

pub struct ReceiptProcessor;

impl ReceiptProcessor {
    pub fn new() -> Self {
        Self
    }

    pub async fn process_receipt(&self, image_data: &[u8], expected_amount: Decimal) -> Result<ReceiptData> {
        info!("Processing receipt, expected amount: {}", expected_amount);
        
        // Check if it's a PDF
        if image_data.starts_with(b"%PDF") {
            return self.process_pdf_receipt(image_data, expected_amount).await;
        }
        
        // Process as image
        self.process_image_receipt(image_data, expected_amount).await
    }
    
    async fn process_pdf_receipt(&self, pdf_data: &[u8], expected_amount: Decimal) -> Result<ReceiptData> {
        debug!("Processing PDF receipt");
        
        // Write PDF data to temporary file
        let temp_file = std::env::temp_dir().join(format!("receipt_{}.pdf", uuid::Uuid::new_v4()));
        std::fs::write(&temp_file, pdf_data)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to write temp file: {}", e)))?;
        
        // Extract text from PDF file
        let text = extract_text_from_pdf(&temp_file)?;
        
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_file);
        debug!("Extracted text from PDF: {}", text);
        
        // Parse receipt data from text
        self.parse_receipt_text(&text, expected_amount)
    }
    
    async fn process_image_receipt(&self, image_data: &[u8], expected_amount: Decimal) -> Result<ReceiptData> {
        debug!("Processing image receipt with Tesseract");
        
        // Save image to temporary file
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| AppError::OcrError(format!("Failed to create temp file: {}", e)))?;
        
        std::io::Write::write_all(&mut temp_file, image_data)
            .map_err(|e| AppError::OcrError(format!("Failed to write temp file: {}", e)))?;
        
        let temp_path = temp_file.path();
        
        // Run Tesseract OCR
        let output = Command::new("tesseract")
            .arg(temp_path)
            .arg("stdout")
            .arg("-l")
            .arg("rus+eng") // Russian + English
            .arg("--psm")
            .arg("6") // Uniform block of text
            .output()
            .map_err(|e| AppError::OcrError(format!("Failed to run Tesseract: {}", e)))?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::OcrError(format!("Tesseract failed: {}", error)));
        }
        
        let text = String::from_utf8_lossy(&output.stdout);
        debug!("OCR extracted text: {}", text);
        
        // Parse receipt data from text
        self.parse_receipt_text(&text, expected_amount)
    }
    
    fn parse_receipt_text(&self, text: &str, expected_amount: Decimal) -> Result<ReceiptData> {
        // Extract amount
        let amount = extract_amount_from_text(text, expected_amount)?;
        
        // Extract bank name
        let bank = self.extract_bank_name(text);
        
        // Extract reference number
        let reference = self.extract_reference(text);
        
        // Extract timestamp
        let timestamp = self.extract_timestamp(text).unwrap_or_else(Utc::now);
        
        // Extract phone number
        let phone = self.extract_phone_number(text);
        
        // Extract card number (last 4 digits)
        let card_number = self.extract_card_number(text);
        
        // Extract status
        let status = self.extract_status(text);
        
        let receipt = ReceiptData {
            amount,
            bank,
            reference,
            timestamp,
            phone,
            card_number,
            status,
        };
        
        // Validate the receipt data
        validate_receipt_data(&receipt, expected_amount)?;
        
        Ok(receipt)
    }
    
    fn extract_bank_name(&self, text: &str) -> String {
        let text_lower = text.to_lowercase();
        
        // Known bank patterns
        let banks = vec![
            ("т-банк", "T-Bank"),
            ("тинькофф", "Tinkoff"),
            ("сбербанк", "Sberbank"),
            ("сбер", "Sberbank"),
            ("альфа-банк", "Alfa-Bank"),
            ("альфа банк", "Alfa-Bank"),
            ("райффайзен", "Raiffeisen"),
            ("втб", "VTB"),
            ("газпромбанк", "Gazprombank"),
            ("открытие", "Otkritie"),
            ("россельхозбанк", "Rosselkhozbank"),
            ("почта банк", "Pochta Bank"),
            ("qiwi", "QIWI"),
            ("юмани", "YooMoney"),
            ("yoomoney", "YooMoney"),
        ];
        
        for (pattern, name) in banks {
            if text_lower.contains(pattern) {
                return name.to_string();
            }
        }
        
        "Unknown Bank".to_string()
    }
    
    fn extract_reference(&self, text: &str) -> String {
        // Try various reference patterns
        let patterns = vec![
            r"(?:номер операции|операция|transaction|чек|квитанция|reference)[:\s]*([\d\-A-Za-z]+)",
            r"№\s*([\d\-A-Za-z]+)",
            r"ID[:\s]*([\d\-A-Za-z]+)",
            r"([\d]{6,})", // At least 6 digits
        ];
        
        for pattern in patterns {
            let re = Regex::new(pattern).unwrap();
            if let Some(captures) = re.captures(text) {
                if let Some(reference) = captures.get(1) {
                    let ref_str = reference.as_str().trim();
                    if ref_str.len() >= 6 {
                        return ref_str.to_string();
                    }
                }
            }
        }
        
        // Generate a reference if none found
        format!("AUTO-{}", chrono::Utc::now().timestamp())
    }
    
    fn extract_phone_number(&self, text: &str) -> Option<String> {
        // Russian phone patterns
        let patterns = vec![
            r"\+7\s*\(?\d{3}\)?\s*\d{3}[-\s]?\d{2}[-\s]?\d{2}",
            r"8\s*\(?\d{3}\)?\s*\d{3}[-\s]?\d{2}[-\s]?\d{2}",
            r"\+7\d{10}",
            r"8\d{10}",
        ];
        
        for pattern in patterns {
            let re = Regex::new(pattern).unwrap();
            if let Some(captures) = re.find(text) {
                let phone = captures.as_str()
                    .chars()
                    .filter(|c| c.is_numeric() || *c == '+')
                    .collect::<String>();
                    
                // Normalize to +7 format
                if phone.starts_with("8") && phone.len() == 11 {
                    return Some(format!("+7{}", &phone[1..]));
                }
                
                return Some(phone);
            }
        }
        
        None
    }
    
    fn extract_card_number(&self, text: &str) -> Option<String> {
        // Look for patterns like "*1234" or "****1234" or "ending with 1234"
        let patterns = vec![
            r"\*+\s*(\d{4})\b",
            r"(?:карта|card|счет|счёт).*?(\d{4})\b",
            r"(?:заканчивается на|ending with|оканчивается)\s*(\d{4})",
            r"\b(\d{4})\s*(?:карта|card)",
        ];
        
        for pattern in patterns {
            let re = Regex::new(pattern).unwrap();
            if let Some(captures) = re.captures(text) {
                if let Some(digits) = captures.get(1) {
                    return Some(digits.as_str().to_string());
                }
            }
        }
        
        None
    }
    
    fn extract_status(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();
        
        // Status patterns
        let status_patterns = vec![
            (vec!["успешно", "успешный", "успешная", "выполнено", "completed", "success"], "Успешно"),
            (vec!["отклонено", "отклонен", "declined", "rejected", "failed"], "Отклонено"),
            (vec!["в обработке", "обрабатывается", "processing", "pending"], "В обработке"),
            (vec!["отменено", "отменен", "cancelled", "canceled"], "Отменено"),
        ];
        
        for (patterns, status) in status_patterns {
            for pattern in patterns {
                if text_lower.contains(pattern) {
                    return Some(status.to_string());
                }
            }
        }
        
        None
    }
    
    fn extract_timestamp(&self, text: &str) -> Option<DateTime<Utc>> {
        // Try various date/time patterns
        let patterns = vec![
            (r"(\d{2})\.(\d{2})\.(\d{4})\s+(\d{2}):(\d{2}):(\d{2})", "%d.%m.%Y %H:%M:%S"),
            (r"(\d{2})\.(\d{2})\.(\d{4})\s+(\d{2}):(\d{2})", "%d.%m.%Y %H:%M"),
            (r"(\d{2})\.(\d{2})\.(\d{4})", "%d.%m.%Y"),
            (r"(\d{4})-(\d{2})-(\d{2})\s+(\d{2}):(\d{2}):(\d{2})", "%Y-%m-%d %H:%M:%S"),
            (r"(\d{4})-(\d{2})-(\d{2})", "%Y-%m-%d"),
        ];
        
        for (pattern, format) in patterns {
            let re = Regex::new(pattern).unwrap();
            if let Some(captures) = re.captures(text) {
                let date_str = captures.get(0)?.as_str();
                if let Ok(naive_dt) = NaiveDateTime::parse_from_str(date_str, format) {
                    return Some(DateTime::from_naive_utc_and_offset(naive_dt, Utc));
                }
                // Try without time component
                if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_str, format) {
                    let naive_dt = naive_date.and_hms_opt(0, 0, 0)?;
                    return Some(DateTime::from_naive_utc_and_offset(naive_dt, Utc));
                }
            }
        }
        
        None
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReceiptData {
    pub amount: Decimal,
    pub bank: String,
    pub reference: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub phone: Option<String>,
    pub card_number: Option<String>,
    pub status: Option<String>,
}

impl ReceiptData {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "amount": self.amount.to_string(),
            "bank": self.bank,
            "reference": self.reference,
            "timestamp": self.timestamp.to_rfc3339(),
            "phone": self.phone,
            "card_number": self.card_number,
            "status": self.status
        })
    }
    
    pub fn is_successful(&self) -> bool {
        self.status.as_ref().map(|s| s == "Успешно").unwrap_or(false)
    }
}