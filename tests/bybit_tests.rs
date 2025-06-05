mod common;

use itrader_backend::bybit::{BybitP2PClient, models::*};
use serde_json::json;

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

#[tokio::test]
async fn test_bybit_auth() {
    common::setup();
    
    println!("=== Testing Bybit P2P Authentication ===");
    
    // Load test credentials
    let (api_key, api_secret) = load_bybit_credentials().await;
    
    // Create client
    let config = common::get_test_config();
    let rate_limiter = common::create_rate_limiter();
    
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
            println!("✓ Successfully authenticated with Bybit");
            println!("  Account ID: {}", account.id);
            println!("  Nickname: {}", account.nickname);
            println!("  Active ads: {}", account.active_ads);
            println!("  Status: {}", account.status);
        }
        Err(e) => {
            println!("✗ Failed to authenticate with Bybit: {}", e);
            let credentials_file = std::env::var("BYBIT_CREDENTIALS_FILE")
                .unwrap_or_else(|_| "test_data/bybit_creditials.json".to_string());
            println!("  Please check API credentials in {}", credentials_file);
            println!("\nNote: This error often occurs when:");
            println!("  1. Using testnet API keys with mainnet endpoint");
            println!("  2. Using mainnet API keys with testnet endpoint");
            println!("  3. System time is not synchronized (MOST COMMON)");
            println!("\nTo test with testnet, use: ./test.sh bybit-auth --testnet");
            println!("\nTo check time sync, run: cargo run --bin check_time_sync");
            
            // Check if it's a timestamp error
            if e.to_string().contains("timestamp") || e.to_string().contains("10002") {
                println!("\n⚠️  This appears to be a time synchronization issue!");
                println!("   Run 'cargo run --bin check_time_sync' to verify");
            }
        }
    }
}

#[tokio::test]
async fn test_bybit_get_advertisements() {
    common::setup();
    
    println!("\n=== Testing Bybit Get Advertisements ===");
    
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
    
    // Get my advertisements
    match client.get_my_advertisements().await {
        Ok(ads) => {
            println!("✓ Successfully fetched advertisements");
            println!("  Found {} advertisements", ads.len());
            
            for (i, ad) in ads.iter().take(3).enumerate() {
                println!("\n  Advertisement {}:", i + 1);
                println!("    ID: {}", ad.id);
                println!("    Asset: {}", ad.asset);
                println!("    Fiat: {}", ad.fiat);
                println!("    Price: {} {}/{}", ad.price, ad.fiat, ad.asset);
                println!("    Amount: {} {}", ad.amount, ad.asset);
                println!("    Min/Max: {}-{} {}", ad.min_amount, ad.max_amount, ad.fiat);
                println!("    Status: {}", ad.status);
                if let Some(remarks) = &ad.remarks {
                    println!("    Remarks: {}", remarks);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to get advertisements: {}", e);
        }
    }
}

#[tokio::test]
async fn test_bybit_check_availability() {
    common::setup();
    
    println!("\n=== Testing Bybit Account Availability ===");
    
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
    
    // Check if account is available for new ads
    match client.is_account_available().await {
        Ok(available) => {
            println!("✓ Successfully checked account availability");
            println!("  Account available for new ads: {}", available);
            
            if let Ok(count) = client.get_active_ads_count().await {
                println!("  Current active ads: {}/{}", count, config.bybit.max_ads_per_account);
            }
        }
        Err(e) => {
            println!("✗ Failed to check availability: {}", e);
        }
    }
}

#[tokio::test]
async fn test_bybit_get_orders() {
    common::setup();
    
    println!("\n=== Testing Bybit Get P2P Orders ===");
    
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
            println!("✓ Successfully fetched P2P orders");
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
            println!("✗ Failed to get orders: {}", e);
            println!("  This might be normal if the API endpoint requires active orders");
        }
    }
}

#[tokio::test]
async fn test_bybit_get_all_advertisements() {
    println!("\n=== Testing Bybit Get All Advertisements ===");
    
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
    
    // Get all advertisements (active and inactive)
    match client.get_all_my_advertisements().await {
        Ok(ads) => {
            println!("✓ Successfully fetched all advertisements");
            println!("  Total advertisements: {}", ads.len());
            
            let active_count = ads.iter().filter(|ad| ad.status == "1").count();
            let inactive_count = ads.iter().filter(|ad| ad.status == "2").count();
            let hidden_count = ads.iter().filter(|ad| ad.status == "3").count();
            
            println!("  Active: {}", active_count);
            println!("  Inactive: {}", inactive_count);
            println!("  Hidden: {}", hidden_count);
            
            for (i, ad) in ads.iter().take(5).enumerate() {
                println!("\n  Advertisement {} (Status: {}):", i + 1, match ad.status.as_str() {
                    "1" => "Active",
                    "2" => "Inactive",
                    "3" => "Hidden",
                    _ => "Unknown"
                });
                println!("    ID: {}", ad.id);
                println!("    Price: {} {}/{}", ad.price, ad.fiat, ad.asset);
                println!("    Amount: {} {}", ad.amount, ad.asset);
                println!("    Created: {}", ad.created_at);
            }
        }
        Err(e) => {
            println!("✗ Failed to get all advertisements: {:?}", e);
            println!("  This might be due to:");
            println!("  - Account needs VA (Verified Advertiser) status or above");
            println!("  - P2P API permissions not enabled in API key settings");
            println!("  - The account hasn't completed P2P merchant verification");
            println!("\n  Note: Bybit P2P API is only available for verified advertisers");
            println!("  To enable P2P API access:");
            println!("  1. Become a P2P advertiser on Bybit");
            println!("  2. Reach VA (Verified Advertiser) level");
            println!("  3. Enable P2P permissions in API Key Management");
        }
    }
}

#[tokio::test]
async fn test_bybit_get_active_advertisements() {
    println!("\n=== Testing Bybit Get Active Advertisements ===");
    
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
    
    // Get only active advertisements
    match client.get_active_advertisements().await {
        Ok(ads) => {
            println!("✓ Successfully fetched active advertisements");
            println!("  Active advertisements: {}", ads.len());
            
            for (i, ad) in ads.iter().take(3).enumerate() {
                println!("\n  Active Advertisement {}:", i + 1);
                println!("    ID: {}", ad.id);
                println!("    Price: {} {}/{}", ad.price, ad.fiat, ad.asset);
                println!("    Amount: {} {}", ad.amount, ad.asset);
                println!("    Min-Max: {}-{} {}", ad.min_amount, ad.max_amount, ad.fiat);
                println!("    Payment Methods: {:?}", ad.payment_methods.iter().map(|pm| &pm.name).collect::<Vec<_>>());
            }
        }
        Err(e) => {
            println!("✗ Failed to get active advertisements: {}", e);
        }
    }
}

#[tokio::test]
async fn test_bybit_get_advertisement_chats() {
    println!("\n=== Testing Bybit Get Advertisement Chats ===");
    
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
    
    // First get active advertisements
    match client.get_active_advertisements().await {
        Ok(ads) => {
            if ads.is_empty() {
                println!("No active advertisements found to get chats for");
                return;
            }
            
            // Get chats for the first advertisement
            let ad = &ads[0];
            println!("Getting chats for advertisement: {}", ad.id);
            
            match client.get_all_order_chats(&ad.id).await {
                Ok(chats) => {
                    println!("✓ Successfully fetched chats");
                    println!("  Total orders with chats: {}", chats.len());
                    
                    for (i, order_chat) in chats.iter().take(3).enumerate() {
                        println!("\n  Order {} Chat:", i + 1);
                        println!("    Order ID: {}", order_chat.order_id);
                        println!("    Status: {}", order_chat.order_status);
                        println!("    Buyer ID: {}", order_chat.buyer_id);
                        println!("    Seller ID: {}", order_chat.seller_id);
                        println!("    Messages: {}", order_chat.messages.len());
                        
                        for (j, msg) in order_chat.messages.iter().take(5).enumerate() {
                            println!("      Message {}: [{}] from {}: {}", 
                                j + 1, 
                                msg.created_at, 
                                msg.sender_id, 
                                msg.content
                            );
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Failed to get chats: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to get advertisements: {}", e);
        }
    }
}