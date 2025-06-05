use anyhow::Context;
use colored::Colorize;
use itrader_backend::core::config::Config;
use itrader_backend::core::rate_limiter::RateLimiter;
use reqwest::{Client, Proxy};
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Enable debug logging
    tracing_subscriber::fmt()
        .with_env_filter("itrader_backend=debug")
        .init();
        
    println!("{}", "=== Gate.io API Test with Proxy ===".bright_blue().bold());
    println!();

    // Load config
    let config = Config::load().context("Failed to load config")?;
    
    // Check if proxy is set in environment
    let proxy_url = std::env::var("HTTP_PROXY")
        .or_else(|_| std::env::var("http_proxy"))
        .unwrap_or_else(|_| "http://127.0.0.1:2080".to_string());
    
    println!("Testing with proxy: {}", proxy_url);
    
    // Build client with proxy
    let mut client_builder = Client::builder()
        .timeout(Duration::from_secs(30))
        .cookie_store(false)
        .gzip(false);
        
    // Try to set proxy
    match Proxy::all(&proxy_url) {
        Ok(proxy) => {
            client_builder = client_builder.proxy(proxy);
            println!("{}", "✓ Proxy configured".green());
        }
        Err(e) => {
            println!("{}", format!("⚠ Could not configure proxy: {}", e).yellow());
            println!("Continuing without proxy...");
        }
    }
    
    let client = client_builder.build()?;
    
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
    
    // Test URL
    let url = format!(
        "{}/payments/payouts?filters%5Bstatus%5D%5B%5D=4&filters%5Bstatus%5D%5B%5D=5&page=1",
        config.gate.base_url
    );
    
    println!("\nTesting URL: {}", url);
    
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
        .await?;
        
    let status = response.status();
    println!("\nResponse status: {}", status);
    
    let text = response.text().await?;
    println!("\nResponse body:");
    
    // Try to pretty print JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        println!("{}", text);
    }
    
    Ok(())
}