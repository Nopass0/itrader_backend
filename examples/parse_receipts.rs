use itrader_backend::ocr::PdfReceiptParser;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let parser = PdfReceiptParser::new();
    let test_data_dir = PathBuf::from("test_data");
    
    let receipt_files = vec![
        "Receipt (8).pdf",
        "Receipt (9).pdf",
        "Receipt (10).pdf",
        "receipt_27.05.2025.pdf",
    ];
    
    println!("\n🧪 Парсинг PDF чеков\n");
    println!("{}", "=".repeat(50));
    
    for file_name in receipt_files {
        let file_path = test_data_dir.join(file_name);
        println!("\n📄 Обработка: {}", file_name);
        println!("{}", "-".repeat(50));
        
        match parser.parse_receipt(&file_path).await {
            Ok(info) => {
                println!("✅ Успешно распознан!");
                println!("📅 Дата/Время:    {}", info.date_time.format("%Y-%m-%d %H:%M:%S UTC"));
                println!("💰 Сумма:         {} ₽", info.amount);
                
                if let Some(bank) = &info.bank_name {
                    println!("🏦 Банк:          {}", bank);
                }
                
                if let Some(tx_id) = &info.transaction_id {
                    println!("🔢 Транзакция:    {}", tx_id);
                }
                
                if let Some(recipient) = &info.recipient {
                    println!("👤 Получатель:    {}", recipient);
                }
                
                if let Some(sender) = &info.sender {
                    println!("📤 Отправитель:   {}", sender);
                }
                
                if let Some(card) = &info.card_number {
                    println!("💳 Карта:         {}", card);
                }
                
                if let Some(phone) = &info.phone_number {
                    println!("📱 Телефон:       {}", phone);
                }
                
                if let Some(status) = &info.status {
                    println!("✔️  Статус:        {}", status);
                }
            }
            Err(e) => {
                println!("❌ Ошибка парсинга: {}", e);
            }
        }
    }
    
    println!("\n{}", "=".repeat(50));
    println!("✨ Тест завершен");
}