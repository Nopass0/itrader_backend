use anyhow::Context;
use colored::Colorize;
use itrader_backend::core::config::Config;
use reqwest::Client;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", "=== Gate.io API Endpoint Test ===".bright_blue().bold());
    println!();

    // Load config
    let config = Config::load().context("Failed to load config")?;
    
    // Build client
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .cookie_store(false)
        .gzip(false)
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
    
    println!("Using cookies: {} chars", cookie_string.len());
    
    // Test different endpoint variations
    let base_urls = vec![
        &config.gate.base_url,
        "https://panel.gate.cx/api/v1",
        "https://panel.gate.cx/api",
        "https://panel.gate.cx",
    ];
    
    let endpoints = vec![
        "/payments/payouts",
        "/payments/payouts?page=1",
        "/payments/payouts?filters[status][]=4&filters[status][]=5&page=1",
        "/payments/payouts?filters%5Bstatus%5D%5B%5D=4&filters%5Bstatus%5D%5B%5D=5&page=1",
        "/payments/payouts/available",
        "/payments/payouts/pending",
        "/payouts",
        "/api/v1/payments/payouts",
    ];
    
    for base_url in &base_urls {
        println!("\n{}", format!("Testing base URL: {}", base_url).yellow());
        println!("{}", "-".repeat(60));
        
        for endpoint in &endpoints {
            let url = if endpoint.starts_with('/') {
                format!("{}{}", base_url, endpoint)
            } else {
                format!("{}/{}", base_url, endpoint)
            };
            
            print!("Testing: {} ... ", url);
            
            // Make request
            let response = client
                .get(&url)
                .header("cookie", &cookie_string)
                .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .header("accept", "application/json, text/plain, */*")
                .header("accept-language", "en-US,en;q=0.9")
                .header("referer", "https://panel.gate.cx/")
                .header("origin", "https://panel.gate.cx")
                .send()
                .await;
                
            match response {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        println!("{}", format!("✓ {} - Success!", status).green());
                        let text = resp.text().await?;
                        // Check if it's JSON and has data
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                            if json.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
                                println!("  Response contains success=true");
                                if let Some(response) = json.get("response") {
                                    if let Some(payouts) = response.get("payouts") {
                                        if let Some(data) = payouts.get("data") {
                                            if let Some(arr) = data.as_array() {
                                                println!("  Found {} payouts", arr.len());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        println!("{}", format!("✗ {}", status).red());
                    }
                }
                Err(e) => {
                    println!("{}", format!("✗ Error: {}", e).red());
                }
            }
        }
    }
    
    // Also test the exact URL from browser logs
    println!("\n{}", "Testing exact browser URL format:".yellow());
    println!("{}", "-".repeat(60));
    
    let browser_url = "https://panel.gate.cx/api/v1/payments/payouts?filters[status][]=4&filters[status][]=5&page=1";
    print!("Testing: {} ... ", browser_url);
    
    let response = client
        .get(browser_url)
        .header("cookie", &cookie_string)
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("accept", "application/json, text/plain, */*")
        .header("accept-language", "en-US,en;q=0.9")
        .header("referer", "https://panel.gate.cx/")
        .header("origin", "https://panel.gate.cx")
        .send()
        .await?;
        
    let status = response.status();
    println!("{}", format!("{}", status).yellow());
    
    let text = response.text().await?;
    println!("\nResponse:");
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        println!("{}", text);
    }
    
    Ok(())
}