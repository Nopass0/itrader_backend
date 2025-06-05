use anyhow::Result;
use reqwest::Client;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new();
    let url = "https://api2.bybit.com/fiat/otc/item/online";
    
    let payload = serde_json::json!({
        "userId": 431812707,
        "tokenId": "USDT",
        "currencyId": "RUB",
        "payment": ["382", "75"],
        "side": "0",
        "size": "10",
        "page": "2",
        "amount": "",
        "vaMaker": false,
        "bulkMaker": false,
        "canTrade": false,
        "verificationFilter": 0,
        "sortType": "TRADE_PRICE",
        "paymentPeriod": [],
        "itemRegion": 1
    });
    
    println!("Sending request to: {}", url);
    let response = client
        .post(url)
        .json(&payload)
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 YaBrowser/25.2.0.0 Safari/537.36")
        .header("Accept", "application/json")
        .header("Accept-Language", "ru-RU")
        .header("Content-Type", "application/json;charset=UTF-8")
        .header("Origin", "https://www.bybit.com")
        .header("Referer", "https://www.bybit.com/")
        .send()
        .await?;
        
    println!("Status: {}", response.status());
    
    let text = response.text().await?;
    
    // Save to file for analysis
    std::fs::write("bybit_response.json", &text)?;
    println!("Response saved to bybit_response.json");
    
    // Try to parse
    let json: Value = serde_json::from_str(&text)?;
    
    // Pretty print just the first item structure
    if let Some(items) = json.get("result").and_then(|r| r.get("items")).and_then(|i| i.as_array()) {
        if let Some(first_item) = items.first() {
            println!("\nFirst item structure:");
            println!("{}", serde_json::to_string_pretty(first_item)?);
        }
    }
    
    Ok(())
}