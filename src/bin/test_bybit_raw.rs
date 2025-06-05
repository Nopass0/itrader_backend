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
        "page": "5",
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
    println!("Payload: {}", serde_json::to_string_pretty(&payload)?);
    
    let response = client
        .post(url)
        .json(&payload)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await?;
    
    let status = response.status();
    println!("\nStatus: {}", status);
    
    let text = response.text().await?;
    println!("\nRaw Response:\n{}", text);
    
    // Try to parse as JSON
    if let Ok(json) = serde_json::from_str::<Value>(&text) {
        if let Some(result) = json.get("result") {
            if let Some(items) = result.get("items") {
                if let Some(items_array) = items.as_array() {
                    if let Some(first_item) = items_array.first() {
                        println!("\nFirst item structure:");
                        println!("{}", serde_json::to_string_pretty(first_item)?);
                    }
                }
            }
        }
    }
    
    Ok(())
}