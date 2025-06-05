use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Bybit API Connectivity Test ===\n");
    
    // Test URLs
    let urls = vec![
        ("https://api2.bybit.com", "Main API"),
        ("https://api.bybit.com", "Alternative API"),
        ("https://www.bybit.com", "Website"),
    ];
    
    println!("Testing connectivity to Bybit endpoints...\n");
    
    for (url, name) in urls {
        print!("Testing {} ({})... ", name, url);
        
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;
            
        match client.get(url).send().await {
            Ok(response) => {
                println!("✓ Connected (Status: {})", response.status());
            }
            Err(e) => {
                println!("✗ Failed: {}", e);
            }
        }
    }
    
    println!("\n=== P2P API Test ===\n");
    
    // Test the actual P2P endpoint
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
        
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
    
    println!("Testing P2P endpoint: {}", url);
    print!("Sending POST request... ");
    
    match client
        .post(url)
        .json(&payload)
        .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .header("Accept", "application/json")
        .send()
        .await
    {
        Ok(response) => {
            println!("✓ Success!");
            println!("Status: {}", response.status());
            
            if response.status().is_success() {
                match response.text().await {
                    Ok(text) => {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(ret_code) = json.get("ret_code") {
                                println!("API Response: ret_code={}", ret_code);
                            }
                            if let Some(result) = json.get("result") {
                                if let Some(count) = result.get("count") {
                                    println!("Found {} P2P trades", count);
                                }
                            }
                        }
                    }
                    Err(e) => println!("Failed to read response: {}", e),
                }
            }
        }
        Err(e) => {
            println!("✗ Failed!");
            println!("Error: {}", e);
            
            if e.is_timeout() {
                println!("\nThe request timed out. This could be due to:");
                println!("1. Network connectivity issues");
                println!("2. Firewall or proxy blocking the connection");
                println!("3. Bybit API being temporarily unavailable");
                println!("4. Rate limiting or IP blocking");
            } else if e.is_connect() {
                println!("\nConnection failed. This could be due to:");
                println!("1. No internet connection");
                println!("2. DNS resolution issues");
                println!("3. The API endpoint being down");
            }
        }
    }
    
    println!("\n=== Test Complete ===");
    
    Ok(())
}