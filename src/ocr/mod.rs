pub mod processor;
pub mod validators;
pub mod pdf;

pub use processor::{ReceiptProcessor, ReceiptData};
pub use pdf::{PdfReceiptParser, ReceiptInfo};

use crate::utils::error::Result;
use rust_decimal::Decimal;

pub struct OcrProcessor {
    processor: ReceiptProcessor,
}

impl OcrProcessor {
    pub fn new() -> Self {
        Self {
            processor: ReceiptProcessor::new(),
        }
    }

    pub async fn process_receipt(&self, receipt_path: &str) -> Result<ReceiptData> {
        let receipt_data = tokio::fs::read(receipt_path).await?;
        let expected_amount = Decimal::new(0, 0); // Will be provided by caller
        self.processor.process_receipt(&receipt_data, expected_amount).await
    }

    pub async fn process_receipt_with_amount(&self, receipt_path: &str, expected_amount: Decimal) -> Result<ReceiptData> {
        let receipt_data = tokio::fs::read(receipt_path).await?;
        self.processor.process_receipt(&receipt_data, expected_amount).await
    }

    pub async fn compare_with_transaction(
        &self,
        receipt: &ReceiptData,
        wallet: &str,
        bank_name: Option<&str>,
        amount: Decimal,
    ) -> Result<bool> {
        // Check if receipt was successful
        if !receipt.is_successful() {
            return Ok(false);
        }

        // Compare amount
        if !validators::validate_amount(receipt.amount, amount)? {
            return Ok(false);
        }

        // Compare phone or card number
        let wallet_matches = if let Some(phone) = &receipt.phone {
            // Normalize phone numbers for comparison
            let normalized_phone = phone.chars().filter(|c| c.is_numeric()).collect::<String>();
            let normalized_wallet = wallet.chars().filter(|c| c.is_numeric()).collect::<String>();
            
            // Check if wallet is a phone number (10-11 digits)
            if normalized_wallet.len() >= 10 && normalized_wallet.len() <= 11 {
                normalized_phone.ends_with(&normalized_wallet) || normalized_wallet.ends_with(&normalized_phone)
            } else {
                false
            }
        } else if let Some(card) = &receipt.card_number {
            // Check if wallet contains the last 4 digits
            wallet.ends_with(card)
        } else {
            false
        };

        if !wallet_matches {
            return Ok(false);
        }

        // Compare bank if provided
        if let Some(expected_bank) = bank_name {
            if !receipt.bank.to_lowercase().contains(&expected_bank.to_lowercase()) &&
               !expected_bank.to_lowercase().contains(&receipt.bank.to_lowercase()) {
                return Ok(false);
            }
        }

        Ok(true)
    }
}