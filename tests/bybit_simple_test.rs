use itrader_backend::bybit::BybitP2PClient;

mod common;

#[tokio::test]
async fn test_bybit_simple_request() {
    println!("\n=== Testing Bybit Simple Request ===");
    
    // Load credentials
    let cred_data = tokio::fs::read_to_string("test_data/bybit_creditials.json")
        .await
        .expect("Failed to read bybit_creditials.json");
    
    let creds: serde_json::Value = serde_json::from_str(&cred_data)
        .expect("Failed to parse credentials");
    
    let api_key = creds["api_key"].as_str().expect("Missing api_key");
    let api_secret = creds["api_secret"].as_str().expect("Missing api_secret");
    
    println!("  API Key: {}...", &api_key[..8.min(api_key.len())]);
    
    let config = common::get_test_config();
    let rate_limiter = common::create_rate_limiter();
    
    println!("  Bybit REST URL: {}", config.bybit.rest_url);
    
    let client = BybitP2PClient::new(
        config.bybit.rest_url.clone(),
        api_key.to_string(),
        api_secret.to_string(),
        rate_limiter,
        config.bybit.max_ads_per_account,
    ).expect("Failed to create Bybit client");
    
    // Try a simple endpoint that doesn't require P2P permissions
    println!("\n  Testing server time endpoint...");
    match client.get_server_time().await {
        Ok(time) => {
            println!("✓ Server time request successful: {}", time);
        }
        Err(e) => {
            println!("✗ Server time request failed: {:?}", e);
        }
    }
}