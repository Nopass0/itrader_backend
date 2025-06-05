use reqwest::Client;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\n=== Quick Bybit Rate Test ===\n");
    
    // Create client with short timeout
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
        
    let url = "https://api2.bybit.com/fiat/otc/item/online";
    let payload = serde_json::json!({
        "userId": 431812707,
        "tokenId": "USDT",
        "currencyId": "RUB",
        "payment": ["382", "75"],
        "side": "0",
        "size": "10",
        "page": "4",
        "amount": "",
        "vaMaker": false,
        "bulkMaker": false,
        "canTrade": false,
        "verificationFilter": 0,
        "sortType": "TRADE_PRICE",
        "paymentPeriod": [],
        "itemRegion": 1
    });
    
    println!("Fetching P2P trades from page 4 (small day amounts)...");
    
    let response = client
        .post(url)
        .json(&payload)
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .header("Accept", "application/json")
        .header("Accept-Language", "ru-RU")
        .header("Content-Type", "application/json;charset=UTF-8")
        .header("Origin", "https://www.bybit.com")
        .header("Referer", "https://www.bybit.com/")
        .send()
        .await?;
        
    let status = response.status();
    println!("Response status: {}", status);
    
    if status.is_success() {
        let text = response.text().await?;
        let json: serde_json::Value = serde_json::from_str(&text)?;
        
        if let Some(result) = json.get("result") {
            if let Some(items) = result.get("items").and_then(|v| v.as_array()) {
                println!("\nFound {} items", items.len());
                
                if items.len() >= 2 {
                    let penultimate = &items[items.len() - 2];
                    if let Some(price) = penultimate.get("price").and_then(|v| v.as_str()) {
                        println!("\nPenultimate item price: {} RUB/USDT", price);
                        if let Some(nickname) = penultimate.get("nickName").and_then(|v| v.as_str()) {
                            println!("Trader: {}", nickname);
                        }
                        if let Some(min) = penultimate.get("minAmount").and_then(|v| v.as_str()) {
                            if let Some(max) = penultimate.get("maxAmount").and_then(|v| v.as_str()) {
                                println!("Range: {} - {} RUB", min, max);
                            }
                        }
                    }
                }
                
                // Show first 3 prices
                println!("\nFirst 3 prices:");
                for (i, item) in items.iter().take(3).enumerate() {
                    if let Some(price) = item.get("price").and_then(|v| v.as_str()) {
                        println!("  #{}: {} RUB/USDT", i + 1, price);
                    }
                }
            }
        }
    }
    
    println!("\n=== Test Complete ===");
    Ok(())
}