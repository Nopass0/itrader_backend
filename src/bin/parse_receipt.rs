use itrader_backend::ocr::PdfReceiptParser;
use std::env;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <pdf_file_path>", args[0]);
        std::process::exit(1);
    }
    
    let pdf_path = PathBuf::from(&args[1]);
    
    if !pdf_path.exists() {
        eprintln!("Error: File not found: {}", pdf_path.display());
        std::process::exit(1);
    }
    
    let parser = PdfReceiptParser::new();
    
    println!("Parsing receipt: {}", pdf_path.display());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    match parser.parse_receipt(&pdf_path).await {
        Ok(info) => {
            println!("✅ Successfully parsed!");
            println!("📅 Date/Time:     {}", info.date_time.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("💰 Amount:        {} RUB", info.amount);
            println!("🏦 Bank:          {}", info.bank_name.clone().unwrap_or_else(|| "Not detected".to_string()));
            println!("🔢 Transaction:   {}", info.transaction_id.clone().unwrap_or_else(|| "Not detected".to_string()));
            println!("👤 Recipient:     {}", info.recipient.clone().unwrap_or_else(|| "Not detected".to_string()));
            println!("📤 Sender:        {}", info.sender.clone().unwrap_or_else(|| "Not detected".to_string()));
            println!("💳 Card:          {}", info.card_number.clone().unwrap_or_else(|| "Not detected".to_string()));
            
            // Output JSON format for programmatic use
            if args.len() > 2 && args[2] == "--json" {
                let json_output = serde_json::json!({
                    "date_time": info.date_time.to_rfc3339(),
                    "amount": info.amount.to_string(),
                    "bank_name": info.bank_name,
                    "transaction_id": info.transaction_id,
                    "recipient": info.recipient,
                    "sender": info.sender,
                    "card_number": info.card_number,
                });
                println!("\nJSON Output:");
                println!("{}", serde_json::to_string_pretty(&json_output)?);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to parse receipt: {}", e);
            eprintln!("   Error chain:");
            for cause in e.chain() {
                eprintln!("   - {}", cause);
            }
            std::process::exit(1);
        }
    }
    
    Ok(())
}