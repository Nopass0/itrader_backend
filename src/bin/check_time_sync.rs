use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BybitTimeResponse {
    #[serde(rename = "retCode")]
    ret_code: i32,
    #[serde(rename = "retMsg")]
    ret_msg: String,
    result: Option<TimeResult>,
}

#[derive(Debug, Deserialize)]
struct TimeResult {
    #[serde(rename = "timeSecond")]
    time_second: String,
    #[serde(rename = "timeNano")]
    time_nano: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Checking Time Synchronization ===\n");
    
    // Get local time
    let local_time = Local::now();
    let utc_time = Utc::now();
    
    println!("Local system time: {}", local_time.format("%Y-%m-%d %H:%M:%S %Z"));
    println!("UTC system time:   {}", utc_time.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("Local timestamp:   {} ms", local_time.timestamp_millis());
    
    // Get Bybit server time
    let client = Client::new();
    
    // Check mainnet
    println!("\n--- Bybit Mainnet ---");
    match get_bybit_time(&client, "https://api.bybit.com").await {
        Ok(server_time) => {
            let diff = (utc_time.timestamp_millis() - server_time) / 1000;
            println!("Server timestamp:  {} ms", server_time);
            println!("Time difference:   {} seconds", diff);
            if diff.abs() > 5 {
                println!("⚠️  WARNING: Time difference is too large! Please sync your system time.");
            } else {
                println!("✓  Time sync is OK");
            }
        }
        Err(e) => println!("Failed to get server time: {}", e),
    }
    
    // Check testnet
    println!("\n--- Bybit Testnet ---");
    match get_bybit_time(&client, "https://api-testnet.bybit.com").await {
        Ok(server_time) => {
            let diff = (utc_time.timestamp_millis() - server_time) / 1000;
            println!("Server timestamp:  {} ms", server_time);
            println!("Time difference:   {} seconds", diff);
            if diff.abs() > 5 {
                println!("⚠️  WARNING: Time difference is too large! Please sync your system time.");
            } else {
                println!("✓  Time sync is OK");
            }
        }
        Err(e) => println!("Failed to get server time: {}", e),
    }
    
    // Show how to fix
    println!("\n--- How to Fix Time Sync Issues ---");
    println!("On Linux:");
    println!("  sudo timedatectl set-ntp true");
    println!("  sudo systemctl restart systemd-timesyncd");
    println!("\nOn macOS:");
    println!("  sudo sntp -sS time.apple.com");
    println!("\nOn Windows:");
    println!("  w32tm /resync");
    
    Ok(())
}

async fn get_bybit_time(client: &Client, base_url: &str) -> Result<i64> {
    let url = format!("{}/v5/market/time", base_url);
    let response = client.get(&url).send().await?;
    let data: BybitTimeResponse = response.json().await?;
    
    if data.ret_code != 0 {
        return Err(anyhow::anyhow!("API error: {}", data.ret_msg));
    }
    
    if let Some(result) = data.result {
        let timestamp = result.time_second.parse::<i64>()? * 1000;
        Ok(timestamp)
    } else {
        Err(anyhow::anyhow!("No time data in response"))
    }
}