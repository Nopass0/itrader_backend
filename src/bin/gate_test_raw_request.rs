use anyhow::Context;
use colored::Colorize;
use reqwest::{Client, header::{HeaderMap, HeaderValue}};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", "=== Gate.io Raw Request Test ===".bright_blue().bold());
    println!();

    // Build client without any special configuration
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .danger_accept_invalid_certs(true) // Accept any cert
        .build()?;
    
    // Load cookies
    let cookie_file = ".gate_cookies.json";
    let cookie_data = std::fs::read_to_string(cookie_file)
        .context("Failed to read cookie file")?;
    let cookies: Vec<serde_json::Value> = serde_json::from_str(&cookie_data)
        .context("Failed to parse cookies")?;
        
    // Build cookie string
    let cookie_string = cookies.iter()
        .filter_map(|c| {
            if let (Some(name), Some(value)) = (c.get("name"), c.get("value")) {
                Some(format!("{}={}", 
                    name.as_str().unwrap_or(""),
                    value.as_str().unwrap_or("")
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("; ");
    
    println!("Cookies loaded: {} chars", cookie_string.len());
    
    // Try the exact URL format from the browser (without encoding)
    let urls = vec![
        "https://panel.gate.cx/api/v1/payments/payouts?filters[status][]=4&filters[status][]=5&page=1",
        "https://panel.gate.cx/api/v1/payments/payouts?filters%5Bstatus%5D%5B%5D=4&filters%5Bstatus%5D%5B%5D=5&page=1",
    ];
    
    for url in urls {
        println!("\n{}", format!("Testing URL: {}", url).yellow());
        
        // Build headers exactly as browser
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_static("application/json, text/plain, */*"));
        headers.insert("accept-language", HeaderValue::from_static("ru,en;q=0.9,pl;q=0.8"));
        headers.insert("cookie", HeaderValue::from_str(&cookie_string)?);
        headers.insert("priority", HeaderValue::from_static("u=1, i"));
        headers.insert("referer", HeaderValue::from_static("https://panel.gate.cx/"));
        headers.insert("sec-ch-ua", HeaderValue::from_static("\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\""));
        headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
        headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Windows\""));
        headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
        headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
        headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
        headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"));
        
        // Try with raw request builder
        let request = client.get(url)
            .headers(headers)
            .build()?;
            
        let response = client.execute(request).await?;
        
        let status = response.status();
        println!("Response status: {}", status);
        
        let text = response.text().await?;
        
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            println!("Response: {}", serde_json::to_string_pretty(&json)?);
            
            // Check if we got actual data
            if let Some(success) = json.get("success").and_then(|s| s.as_bool()) {
                if success {
                    println!("{}", "✓ Success! Got valid response".green());
                    
                    if let Some(response) = json.get("response") {
                        if let Some(payouts) = response.get("payouts") {
                            if let Some(data) = payouts.get("data") {
                                if let Some(arr) = data.as_array() {
                                    println!("{}", format!("Found {} transactions", arr.len()).green());
                                    
                                    // Print first transaction details
                                    if let Some(first) = arr.first() {
                                        println!("\nFirst transaction:");
                                        println!("  ID: {}", first.get("id").and_then(|v| v.as_i64()).unwrap_or(0));
                                        println!("  Status: {}", first.get("status").and_then(|v| v.as_i64()).unwrap_or(0));
                                        if let Some(wallet) = first.get("wallet").and_then(|v| v.as_str()) {
                                            println!("  Wallet: {}", wallet);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            println!("Response text: {}", text);
        }
    }
    
    // Also try to check if cookies are valid by testing auth endpoint
    println!("\n{}", "Testing authentication status...".yellow());
    
    let auth_url = "https://panel.gate.cx/api/v1/auth/me";
    let response = client.get(auth_url)
        .header("cookie", &cookie_string)
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("accept", "application/json, text/plain, */*")
        .header("referer", "https://panel.gate.cx/")
        .send()
        .await?;
        
    let status = response.status();
    println!("Auth check status: {}", status);
    
    if status.is_success() {
        println!("{}", "✓ Authentication is valid".green());
    } else {
        println!("{}", "✗ Authentication failed - cookies might be expired".red());
    }
    
    Ok(())
}