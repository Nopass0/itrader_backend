use itrader_backend::ocr::PdfReceiptParser;
use std::path::PathBuf;

#[tokio::test]
async fn test_all_receipt_files() {
    let parser = PdfReceiptParser::new();
    let test_data_dir = PathBuf::from("/home/user/projects/itrader_backend/test_data");
    
    let receipt_files = vec![
        "Receipt (8).pdf",
        "Receipt (9).pdf",
        "Receipt (10).pdf",
        "receipt_27.05.2025.pdf",
    ];
    
    println!("\n========== PDF Receipt Parser Test Results ==========\n");
    
    for file_name in receipt_files {
        let file_path = test_data_dir.join(file_name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📄 Processing: {}", file_name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        match parser.parse_receipt(&file_path).await {
            Ok(info) => {
                println!("✅ Successfully parsed!");
                println!("📅 Date/Time:     {}", info.date_time.format("%Y-%m-%d %H:%M:%S UTC"));
                println!("💰 Amount:        {} RUB", info.amount);
                println!("🏦 Bank:          {}", info.bank_name.unwrap_or_else(|| "Not detected".to_string()));
                println!("🔢 Transaction:   {}", info.transaction_id.unwrap_or_else(|| "Not detected".to_string()));
                println!("👤 Recipient:     {}", info.recipient.unwrap_or_else(|| "Not detected".to_string()));
                println!("📤 Sender:        {}", info.sender.unwrap_or_else(|| "Not detected".to_string()));
                println!("💳 Card:          {}", info.card_number.unwrap_or_else(|| "Not detected".to_string()));
                println!("📱 Phone:         {}", info.phone_number.unwrap_or_else(|| "Not detected".to_string()));
                println!("✔️  Status:        {}", info.status.unwrap_or_else(|| "Not detected".to_string()));
                
                // Print first 200 chars of raw text for debugging
                if !info.raw_text.is_empty() {
                    let preview = if info.raw_text.len() > 200 {
                        format!("{}...", &info.raw_text[..200])
                    } else {
                        info.raw_text.clone()
                    };
                    println!("\n📝 Text preview:\n{}", preview);
                }
            }
            Err(e) => {
                println!("❌ Failed to parse: {}", e);
                println!("   Error chain:");
                for cause in e.chain() {
                    println!("   - {}", cause);
                }
            }
        }
        println!();
    }
    
    println!("========== Test Complete ==========");
}