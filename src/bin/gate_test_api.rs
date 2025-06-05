use anyhow::Context;
use colored::Colorize;
use itrader_backend::core::config::Config;
use itrader_backend::core::rate_limiter::RateLimiter;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::sync::Arc;
use tracing::debug;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("{}", "=== Gate.io API Endpoint Test ===".bright_blue().bold());
    println!();

    // Load config
    let config = Config::load().context("Failed to load config")?;
    
    // Load cookies
    let cookie_file = ".gate_cookies.json";
    let cookies_str = std::fs::read_to_string(cookie_file)
        .context("Failed to read cookie file")?;
    let cookies: Vec<serde_json::Value> = serde_json::from_str(&cookies_str)
        .context("Failed to parse cookies")?;
    
    // Build cookie header
    let cookie_header = cookies.iter()
        .filter_map(|c| {
            if let (Some(name), Some(value)) = (c.get("name"), c.get("value")) {
                Some(format!("{}={}", name.as_str()?, value.as_str()?))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("; ");
    
    println!("Cookie header length: {}", cookie_header.len());
    
    // Create HTTP client
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    
    // Test different endpoints
    let endpoints = vec![
        ("/payments/payouts", "All payouts"),
        ("/payments/payouts?page=1", "Payouts page 1"),
        ("/payments/payouts?filters%5Bstatus%5D%5B%5D=4&filters%5Bstatus%5D%5B%5D=5&page=1", "Payouts with status 4,5"),
        ("/payments/payouts?search%5Bid%5D=2518434&page=1", "Search by ID"),
    ];
    
    for (path, desc) in endpoints {
        println!("\n{}", format!("Testing: {}", desc).yellow());
        let url = format!("{}{}", config.gate.base_url, path);
        println!("URL: {}", url);
        
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_static("application/json, text/plain, */*"));
        headers.insert("accept-language", HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert("cookie", HeaderValue::from_str(&cookie_header)?);
        headers.insert("referer", HeaderValue::from_static("https://panel.gate.cx/requests?page=1"));
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36"));
        
        match client.get(&url).headers(headers).send().await {
            Ok(response) => {
                let status = response.status();
                println!("Status: {}", status);
                
                if status.is_success() {
                    let text = response.text().await?;
                    println!("{}", "✓ Success!".green());
                    if text.starts_with("{") {
                        // It's JSON
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                            println!("Response: {}", serde_json::to_string_pretty(&json)?);
                        } else {
                            println!("Response (first 200 chars): {}", &text[..text.len().min(200)]);
                        }
                    } else {
                        println!("Response (first 200 chars): {}", &text[..text.len().min(200)]);
                    }
                } else {
                    let text = response.text().await.unwrap_or_default();
                    println!("{}", format!("✗ Failed with status {}", status).red());
                    println!("Error response: {}", text);
                }
            }
            Err(e) => {
                println!("{}", format!("✗ Request failed: {}", e).red());
            }
        }
    }

    Ok(())
}