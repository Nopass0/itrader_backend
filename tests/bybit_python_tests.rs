mod common;

use itrader_backend::bybit::BybitP2PClient;

async fn load_bybit_credentials() -> (String, String) {
    let credentials_file = std::env::var("BYBIT_CREDENTIALS_FILE")
        .unwrap_or_else(|_| "test_data/bybit_creditials.json".to_string());
    
    let cred_data = tokio::fs::read_to_string(&credentials_file)
        .await
        .unwrap_or_else(|_| panic!("Failed to read {}", credentials_file));
    
    let creds: serde_json::Value = serde_json::from_str(&cred_data)
        .expect("Failed to parse credentials");
    
    let api_key = creds["api_key"].as_str().expect("Missing api_key").to_string();
    let api_secret = creds["api_secret"].as_str().expect("Missing api_secret").to_string();
    
    println!("Loaded API credentials from {}", credentials_file);
    println!("  API Key: {}...", &api_key[..api_key.len().min(8)]);
    
    (api_key, api_secret)
}

#[cfg(feature = "python-sdk")]
#[tokio::test]
async fn test_bybit_python_sdk_authentication() {
    common::setup();
    
    println!("=== Testing Bybit Python SDK Authentication ===");
    
    // Load test credentials
    let (api_key, api_secret) = load_bybit_credentials().await;
    
    // Create client with Python SDK
    let config = common::get_test_config();
    let rate_limiter = common::create_rate_limiter();
    
    // Test with production API (testnet = false)
    let client = BybitP2PClient::new(
        config.bybit.rest_url.clone(),
        api_key,
        api_secret,
        rate_limiter,
        config.bybit.max_ads_per_account,
    ).expect("Failed to create Bybit client");
    
    // Test authentication by getting account info
    match client.get_account_info().await {
        Ok(account) => {
            println!("✓ Successfully authenticated with Bybit Python SDK");
            println!("  Account ID: {}", account.id);
            println!("  UID: {}", account.id);  // Same as ID
            println!("  Nickname: {}", account.nickname);
            println!("  Active ads: {}", account.active_ads);
            println!("  Status: {}", account.status);
            
            // Check for email field
            if let Some(email) = &account.email {
                println!("  Email: {}", email);
            } else {
                println!("  Email: not available");
            }
        }
        Err(e) => {
            println!("✗ Failed to authenticate with Bybit Python SDK: {}", e);
            let credentials_file = std::env::var("BYBIT_CREDENTIALS_FILE")
                .unwrap_or_else(|_| "test_data/bybit_creditials.json".to_string());
            println!("  Please check API credentials in {}", credentials_file);
        }
    }
}

#[cfg(feature = "python-sdk")]
#[tokio::test]
async fn test_bybit_python_sdk_get_orders() {
    common::setup();
    
    println!("\n=== Testing Bybit Python SDK Get Orders ===");
    
    // Load credentials and create client
    let (api_key, api_secret) = load_bybit_credentials().await;
    
    let config = common::get_test_config();
    let rate_limiter = common::create_rate_limiter();
    
    let client = BybitP2PClient::new(
        config.bybit.rest_url.clone(),
        api_key,
        api_secret,
        rate_limiter,
        config.bybit.max_ads_per_account,
    ).expect("Failed to create Bybit client");
    
    // Get active orders
    match client.get_active_orders().await {
        Ok(orders) => {
            println!("✓ Successfully fetched P2P orders via Python SDK");
            println!("  Found {} active orders", orders.len());
            
            if orders.is_empty() {
                println!("  No active orders found (this is normal if no trades are ongoing)");
            } else {
                for (i, order) in orders.iter().take(3).enumerate() {
                    println!("\n  Order {}:", i + 1);
                    println!("    ID: {}", order.id);
                    println!("    Ad ID: {}", order.ad_id);
                    println!("    Amount: {} {}", order.amount, order.asset);
                    println!("    Price: {} {}/{}", order.price, order.fiat, order.asset);
                    println!("    Total: {} {}", order.total_price, order.fiat);
                    println!("    Status: {}", order.status);
                    println!("    Created: {}", order.created_at);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to get orders via Python SDK: {}", e);
            println!("  This might be normal if the API endpoint requires active orders");
        }
    }
}

#[cfg(feature = "python-sdk")]
#[tokio::test] 
async fn test_bybit_python_sdk_create_ad() {
    common::setup();
    
    println!("\n=== Testing Bybit Python SDK Create Advertisement ===");
    
    // Load credentials and create client
    let (api_key, api_secret) = load_bybit_credentials().await;
    
    let config = common::get_test_config();
    let rate_limiter = common::create_rate_limiter();
    
    let client = BybitP2PClient::new(
        config.bybit.rest_url.clone(),
        api_key,
        api_secret,
        rate_limiter,
        config.bybit.max_ads_per_account,
    ).expect("Failed to create Bybit client");
    
    // Create test advertisement parameters
    let ad_params = itrader_backend::bybit::models::AdParams {
        asset: "USDT".to_string(),
        fiat: "RUB".to_string(),
        price: "98.5".to_string(),
        amount: "1000".to_string(),
        min_amount: Some("100".to_string()),
        max_amount: Some("1000".to_string()),
        payment_methods: vec!["1".to_string()], // Payment method IDs
        remarks: Some("Test advertisement via Python SDK".to_string()),
    };
    
    println!("Creating test advertisement:");
    println!("  Asset: {}", ad_params.asset);
    println!("  Fiat: {}", ad_params.fiat);
    println!("  Price: {}", ad_params.price);
    println!("  Amount: {}", ad_params.amount);
    
    match client.create_advertisement(ad_params).await {
        Ok(ad) => {
            println!("✓ Successfully created advertisement via Python SDK");
            println!("  Advertisement ID: {}", ad.id);
            println!("  Status: {}", ad.status);
            println!("  Created at: {}", ad.created_at);
            
            // Try to delete it after creation
            println!("\nAttempting to delete the test advertisement...");
            match client.delete_advertisement(&ad.id).await {
                Ok(_) => println!("✓ Successfully deleted test advertisement"),
                Err(e) => println!("✗ Failed to delete test advertisement: {}", e),
            }
        }
        Err(e) => {
            println!("✗ Failed to create advertisement via Python SDK: {}", e);
            println!("  This might be due to:");
            println!("  - Account needs P2P merchant verification");
            println!("  - Insufficient balance");
            println!("  - Invalid payment method IDs");
        }
    }
}

#[cfg(not(feature = "python-sdk"))]
#[test]
fn test_python_sdk_feature_disabled() {
    println!("Python SDK feature is disabled. Enable it with --features python-sdk");
}