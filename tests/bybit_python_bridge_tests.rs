use itrader_backend::bybit::python_bridge::PythonBybitClient;
use itrader_backend::bybit::models::{AdParams, P2POrder};
use itrader_backend::core::config::Config;
use std::sync::Arc;
use tokio;
use tracing::info;
use rust_decimal::Decimal;
use std::str::FromStr;

mod common;
use common::init_test_env;

#[tokio::test]
async fn test_bybit_python_auth() {
    init_test_env();
    info!("Testing Bybit authentication through Python bridge");
    
    let config = Config::from_env().expect("Failed to load config");
    
    // Skip test if no Bybit credentials
    if std::env::var("BYBIT_API_KEY").is_err() {
        info!("Skipping test - no Bybit credentials in environment");
        return;
    }
    
    let client = PythonBybitClient::new(
        config.bybit.api_key.clone(),
        config.bybit.api_secret.clone(),
        config.bybit.testnet
    ).await.expect("Failed to create Python Bybit client");
    
    // Test getting account info
    let account_info = client.get_account_info().await;
    assert!(account_info.is_ok(), "Failed to get account info: {:?}", account_info.err());
    
    let info = account_info.unwrap();
    info!("Account info retrieved: {:?}", info);
    assert!(!info.uid.is_empty(), "UID should not be empty");
}

#[tokio::test]
async fn test_bybit_python_rate_fetcher() {
    init_test_env();
    info!("Testing Bybit rate fetching through Python bridge");
    
    let config = Config::from_env().expect("Failed to load config");
    
    // Skip test if no Bybit credentials
    if std::env::var("BYBIT_API_KEY").is_err() {
        info!("Skipping test - no Bybit credentials in environment");
        return;
    }
    
    let client = PythonBybitClient::new(
        config.bybit.api_key.clone(),
        config.bybit.api_secret.clone(),
        config.bybit.testnet
    ).await.expect("Failed to create Python Bybit client");
    
    // Test getting P2P rates
    let test_amount = std::env::var("TEST_AMOUNT")
        .unwrap_or_else(|_| "10000".to_string())
        .parse::<f64>()
        .unwrap();
    
    let buy_rate = client.get_p2p_buy_rate(test_amount).await;
    assert!(buy_rate.is_ok(), "Failed to get buy rate: {:?}", buy_rate.err());
    let buy_rate = buy_rate.unwrap();
    info!("Buy rate for {} RUB: {} USDT/RUB", test_amount, buy_rate);
    assert!(buy_rate > 0.0, "Buy rate should be positive");
    
    let sell_rate = client.get_p2p_sell_rate(test_amount).await;
    assert!(sell_rate.is_ok(), "Failed to get sell rate: {:?}", sell_rate.err());
    let sell_rate = sell_rate.unwrap();
    info!("Sell rate for {} RUB: {} USDT/RUB", test_amount, sell_rate);
    assert!(sell_rate > 0.0, "Sell rate should be positive");
    
    // Buy rate should be higher than sell rate (spread)
    assert!(buy_rate > sell_rate, "Buy rate should be higher than sell rate");
}

#[tokio::test]
async fn test_bybit_python_account_info() {
    init_test_env();
    info!("Testing Bybit account info through Python bridge");
    
    let config = Config::from_env().expect("Failed to load config");
    
    // Skip test if no Bybit credentials
    if std::env::var("BYBIT_API_KEY").is_err() {
        info!("Skipping test - no Bybit credentials in environment");
        return;
    }
    
    let client = PythonBybitClient::new(
        config.bybit.api_key.clone(),
        config.bybit.api_secret.clone(),
        config.bybit.testnet
    ).await.expect("Failed to create Python Bybit client");
    
    // Get account info
    let account_info = client.get_account_info().await;
    assert!(account_info.is_ok(), "Failed to get account info: {:?}", account_info.err());
    
    let info = account_info.unwrap();
    info!("Account info: {:?}", info);
    
    // Validate account info fields
    assert!(!info.uid.is_empty(), "UID should not be empty");
    assert!(!info.nickname.is_empty(), "Nickname should not be empty");
    assert!(info.kyc_level >= 0, "KYC level should be non-negative");
}

#[tokio::test]
#[ignore] // This test creates data, run manually
async fn test_bybit_python_create_ad() {
    init_test_env();
    info!("Testing Bybit ad creation through Python bridge");
    
    let config = Config::from_env().expect("Failed to load config");
    
    // Skip test if no Bybit credentials
    if std::env::var("BYBIT_API_KEY").is_err() {
        info!("Skipping test - no Bybit credentials in environment");
        return;
    }
    
    let client = PythonBybitClient::new(
        config.bybit.api_key.clone(),
        config.bybit.api_secret.clone(),
        config.bybit.testnet
    ).await.expect("Failed to create Python Bybit client");
    
    // Test ad parameters from environment or use defaults
    let test_ad_data = std::env::var("TEST_AD_DATA").ok();
    
    let ad_params = if let Some(data) = test_ad_data {
        // Parse JSON test data
        serde_json::from_str(&data).expect("Invalid TEST_AD_DATA JSON")
    } else {
        // Default test ad (small amount for safety)
        AdParams {
            side: "0".to_string(), // Sell
            currency: "RUB".to_string(),
            price: Decimal::from_str("98.50").unwrap(),
            amount: Decimal::from_str("10").unwrap(), // Small amount
            min_amount: Decimal::from_str("1000").unwrap(),
            max_amount: Some(Decimal::from_str("5000").unwrap()),
            payment_methods: vec!["582".to_string()], // Tinkoff
            remarks: Some("Test ad from Rust (Python bridge)".to_string()),
            auto_reply: Some("Test auto reply".to_string()),
        }
    };
    
    info!("Creating test ad with params: {:?}", ad_params);
    
    // Create ad
    let result = client.create_ad(ad_params).await;
    assert!(result.is_ok(), "Failed to create ad: {:?}", result.err());
    
    let ad_id = result.unwrap();
    info!("Successfully created ad with ID: {}", ad_id);
    
    // Clean up - delete the ad
    if !config.bybit.testnet {
        info!("Cleaning up - deleting test ad");
        let delete_result = client.delete_ad(&ad_id).await;
        assert!(delete_result.is_ok(), "Failed to delete test ad: {:?}", delete_result.err());
        info!("Test ad deleted successfully");
    }
}

#[tokio::test]
#[ignore] // This test deletes data, run manually
async fn test_bybit_python_delete_ad() {
    init_test_env();
    info!("Testing Bybit ad deletion through Python bridge");
    
    let config = Config::from_env().expect("Failed to load config");
    
    // Skip test if no Bybit credentials
    if std::env::var("BYBIT_API_KEY").is_err() {
        info!("Skipping test - no Bybit credentials in environment");
        return;
    }
    
    // Need an existing ad ID to delete
    let ad_id = std::env::var("TEST_AD_ID")
        .expect("TEST_AD_ID environment variable required for delete test");
    
    let client = PythonBybitClient::new(
        config.bybit.api_key.clone(),
        config.bybit.api_secret.clone(),
        config.bybit.testnet
    ).await.expect("Failed to create Python Bybit client");
    
    info!("Deleting ad with ID: {}", ad_id);
    
    let result = client.delete_ad(&ad_id).await;
    assert!(result.is_ok(), "Failed to delete ad: {:?}", result.err());
    
    info!("Successfully deleted ad");
}

#[tokio::test]
async fn test_bybit_python_get_orders() {
    init_test_env();
    info!("Testing Bybit order fetching through Python bridge");
    
    let config = Config::from_env().expect("Failed to load config");
    
    // Skip test if no Bybit credentials
    if std::env::var("BYBIT_API_KEY").is_err() {
        info!("Skipping test - no Bybit credentials in environment");
        return;
    }
    
    let client = PythonBybitClient::new(
        config.bybit.api_key.clone(),
        config.bybit.api_secret.clone(),
        config.bybit.testnet
    ).await.expect("Failed to create Python Bybit client");
    
    // Get recent orders
    let orders = client.get_orders(None, Some(10)).await;
    assert!(orders.is_ok(), "Failed to get orders: {:?}", orders.err());
    
    let orders = orders.unwrap();
    info!("Retrieved {} orders", orders.len());
    
    for order in orders.iter().take(3) {
        info!("Order: {} - Status: {}, Amount: {} USDT", 
            order.id, order.status, order.amount);
    }
}