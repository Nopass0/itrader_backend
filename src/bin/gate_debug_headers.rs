use anyhow::Context;
use colored::Colorize;
use itrader_backend::core::config::Config;
use reqwest::header::{HeaderMap, HeaderValue};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{}", "=== Gate.io Headers Debug ===".bright_blue().bold());
    println!();

    // Load config
    let config = Config::load().context("Failed to load config")?;
    
    // Load cookies
    let cookie_file = ".gate_cookies.json";
    let cookies_str = std::fs::read_to_string(cookie_file)
        .context("Failed to read cookie file")?;
    let cookies: Vec<serde_json::Value> = serde_json::from_str(&cookies_str)
        .context("Failed to parse cookies")?;
    
    // Build cookie header exactly as browser
    let cookie_parts: Vec<String> = cookies.iter()
        .filter_map(|c| {
            if let (Some(name), Some(value)) = (c.get("name"), c.get("value")) {
                Some(format!("{}={}", name.as_str()?, value.as_str()?))
            } else {
                None
            }
        })
        .collect();
    
    let cookie_header = cookie_parts.join("; ");
    
    println!("Cookies found: {}", cookie_parts.len());
    for part in &cookie_parts {
        let name = part.split('=').next().unwrap_or("unknown");
        println!("  - {}", name);
    }
    
    // Create client with exact browser configuration
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .gzip(true) // Enable gzip like browser
        .build()?;
    
    // Test with exact browser headers
    let url = "https://panel.gate.cx/api/v1/payments/payouts?filters%5Bstatus%5D%5B%5D=4&filters%5Bstatus%5D%5B%5D=5&page=1";
    
    println!("\nTesting URL: {}", url);
    
    let mut headers = HeaderMap::new();
    // Exact headers from browser (skip pseudo headers)
    headers.insert("accept", HeaderValue::from_static("application/json, text/plain, */*"));
    headers.insert("accept-encoding", HeaderValue::from_static("gzip, deflate, br, zstd"));
    headers.insert("accept-language", HeaderValue::from_static("ru,en;q=0.9"));
    headers.insert("cookie", HeaderValue::from_str(&cookie_header)?);
    headers.insert("priority", HeaderValue::from_static("u=1, i"));
    headers.insert("referer", HeaderValue::from_static("https://panel.gate.cx/requests?page=1"));
    headers.insert("sec-ch-ua", HeaderValue::from_static("\"Not A(Brand\";v=\"8\", \"Chromium\";v=\"132\", \"YaBrowser\";v=\"25.2\", \"Yowser\";v=\"2.5\""));
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Linux\""));
    headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));
    headers.insert("user-agent", HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 YaBrowser/25.2.0.0 Safari/537.36"));
    
    println!("\nSending request with {} headers...", headers.len());
    
    match client.get(url).headers(headers).send().await {
        Ok(response) => {
            let status = response.status();
            println!("\nResponse Status: {}", status);
            
            // Print response headers
            println!("\nResponse Headers:");
            for (key, value) in response.headers().iter() {
                println!("  {}: {:?}", key, value);
            }
            
            let text = response.text().await?;
            
            if status.is_success() {
                println!("{}", "\n✓ Success!".green());
                // Try to parse as JSON
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    println!("\nResponse JSON:");
                    println!("{}", serde_json::to_string_pretty(&json)?);
                } else {
                    println!("\nResponse Text (first 500 chars):");
                    println!("{}", &text[..text.len().min(500)]);
                }
            } else {
                println!("{}", format!("\n✗ Failed with status {}", status).red());
                println!("\nError Response:");
                println!("{}", text);
            }
        }
        Err(e) => {
            println!("{}", format!("\n✗ Request failed: {}", e).red());
        }
    }

    Ok(())
}