use std::process::{Command, Stdio};
use std::io::Write;
use serde::{Deserialize, Serialize};
use serde_json;
use tokio;
use tracing::info;
use sqlx::PgPool;
use itrader_backend::db::models::BybitAccount;
use itrader_backend::utils::error::Result;

mod common;
use common::init_test_env;

#[derive(Serialize)]
struct CreateAdRequest {
    api_key: String,
    api_secret: String,
    testnet: bool,
    ad_params: AdParams,
}

#[derive(Serialize)]
struct AdParams {
    side: String,  // "0" for sell, "1" for buy
    currency: String,
    price: String,
    quantity: String,
    min_amount: String,
    max_amount: String,
    payment_methods: Vec<String>,
    remarks: String,
}

#[derive(Deserialize)]
struct BybitResponse {
    #[serde(rename = "retCode")]
    ret_code: i32,
    #[serde(rename = "retMsg")]
    ret_msg: String,
    result: Option<AdResult>,
}

#[derive(Deserialize)]
struct AdResult {
    #[serde(rename = "adId")]
    ad_id: String,
    status: String,
}

async fn get_test_account(pool: &PgPool) -> Result<BybitAccount> {
    // Get first available Bybit account from test database
    let account = sqlx::query_as!(
        BybitAccount,
        r#"
        SELECT id, account_name, api_key, api_secret, 
               active_ads, status, testnet, last_used, created_at, updated_at
        FROM bybit_accounts 
        WHERE status = 'available' 
        LIMIT 1
        "#
    )
    .fetch_optional(pool)
    .await?;
    
    account.ok_or_else(|| anyhow::anyhow!("No available Bybit account found in test database"))
}

async fn update_account_ads(pool: &PgPool, account_id: i32, increment: i32) -> Result<()> {
    sqlx::query!(
        "UPDATE bybit_accounts SET active_ads = active_ads + $1, last_used = NOW() WHERE id = $2",
        increment,
        account_id
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

#[tokio::test]
async fn test_bybit_p2p_create_ad_via_python() {
    init_test_env();
    info!("Testing Bybit P2P ad creation via Python script");
    
    // Connect to test database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:root@localhost/itrader_test".to_string());
    
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");
    
    // Get test account from database
    let account = match get_test_account(&pool).await {
        Ok(acc) => acc,
        Err(e) => {
            info!("Skipping test - no test account available: {}", e);
            return;
        }
    };
    
    info!("Using test account: {}", account.account_name);
    
    // Prepare ad parameters
    let request = CreateAdRequest {
        api_key: account.api_key.clone(),
        api_secret: account.api_secret.clone(),
        testnet: account.testnet,
        ad_params: AdParams {
            side: "0".to_string(),  // Sell
            currency: "RUB".to_string(),
            price: "98.50".to_string(),
            quantity: "10".to_string(),  // Small test amount
            min_amount: "1000".to_string(),
            max_amount: "5000".to_string(),
            payment_methods: vec!["582".to_string()],  // Tinkoff
            remarks: "Test ad from Rust via Python".to_string(),
        },
    };
    
    // Call Python script
    let mut child = Command::new("python3")
        .arg("scripts/bybit_create_ad.py")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start Python script");
    
    // Send JSON to stdin
    let stdin = child.stdin.as_mut().expect("Failed to get stdin");
    let request_json = serde_json::to_string(&request).expect("Failed to serialize request");
    stdin.write_all(request_json.as_bytes()).expect("Failed to write to stdin");
    stdin.flush().expect("Failed to flush stdin");
    
    // Wait for response
    let output = child.wait_with_output().expect("Failed to wait for Python script");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Python script failed: {}", stderr);
    }
    
    // Parse response
    let stdout = String::from_utf8_lossy(&output.stdout);
    info!("Python script response: {}", stdout);
    
    let response: BybitResponse = serde_json::from_str(&stdout)
        .expect("Failed to parse Python script response");
    
    // Check response
    assert_eq!(response.ret_code, 0, "API error: {}", response.ret_msg);
    assert!(response.result.is_some(), "No result in response");
    
    let result = response.result.unwrap();
    info!("Created ad with ID: {}", result.ad_id);
    assert!(!result.ad_id.is_empty(), "Ad ID should not be empty");
    assert_eq!(result.status, "online", "Ad status should be online");
    
    // Update database
    update_account_ads(&pool, account.id, 1).await
        .expect("Failed to update account ads count");
    
    info!("Successfully created P2P ad and updated database");
}

#[tokio::test]
async fn test_bybit_p2p_python_integration() {
    init_test_env();
    info!("Testing full Bybit P2P integration with database");
    
    // Connect to test database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:root@localhost/itrader_test".to_string());
    
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");
    
    // Check if we have any Bybit accounts
    let account_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM bybit_accounts"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count accounts");
    
    info!("Found {} Bybit accounts in test database", account_count.unwrap_or(0));
    
    if account_count.unwrap_or(0) == 0 {
        // Insert a test account
        let test_account = sqlx::query!(
            r#"
            INSERT INTO bybit_accounts (account_name, api_key, api_secret, active_ads, status, testnet)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
            "test_account",
            "test_api_key",
            "test_api_secret",
            0,
            "available",
            true
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to insert test account");
        
        info!("Created test account with ID: {}", test_account.id);
    }
    
    // Get account statistics
    let stats = sqlx::query!(
        r#"
        SELECT 
            COUNT(*) as total_accounts,
            COUNT(*) FILTER (WHERE status = 'available') as available_accounts,
            COALESCE(SUM(active_ads), 0) as total_ads
        FROM bybit_accounts
        "#
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to get statistics");
    
    info!("Account statistics:");
    info!("  Total accounts: {}", stats.total_accounts.unwrap_or(0));
    info!("  Available accounts: {}", stats.available_accounts.unwrap_or(0));
    info!("  Total active ads: {}", stats.total_ads.unwrap_or(0));
}

#[derive(Serialize)]
struct GetRatesRequest {
    amount_rub: f64,
    testnet: bool,
}

#[derive(Deserialize)]
struct RatesResponse {
    success: bool,
    error: Option<String>,
    data: Option<RatesData>,
}

#[derive(Deserialize)]
struct RatesData {
    buy_rate: f64,
    sell_rate: f64,
    amount_rub: f64,
    spread: f64,
    timestamp: String,
}

#[tokio::test]
async fn test_bybit_p2p_get_rates_via_python() {
    init_test_env();
    info!("Testing Bybit P2P rate fetching via Python script");
    
    // Test amounts
    let test_amounts = vec![1000.0, 10000.0, 100000.0];
    
    for amount in test_amounts {
        info!("Getting rates for {} RUB", amount);
        
        // Prepare request
        let request = GetRatesRequest {
            amount_rub: amount,
            testnet: true,
        };
        
        // Call Python script
        let mut child = Command::new("python3")
            .arg("scripts/bybit_get_rates.py")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start Python script");
        
        // Send JSON to stdin
        let stdin = child.stdin.as_mut().expect("Failed to get stdin");
        let request_json = serde_json::to_string(&request).expect("Failed to serialize request");
        stdin.write_all(request_json.as_bytes()).expect("Failed to write to stdin");
        stdin.flush().expect("Failed to flush stdin");
        
        // Wait for response
        let output = child.wait_with_output().expect("Failed to wait for Python script");
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!("Python script failed: {}", stderr);
        }
        
        // Parse response
        let stdout = String::from_utf8_lossy(&output.stdout);
        let response: RatesResponse = serde_json::from_str(&stdout)
            .expect("Failed to parse Python script response");
        
        // Check response
        assert!(response.success, "Rate fetch failed: {:?}", response.error);
        assert!(response.data.is_some(), "No data in response");
        
        let data = response.data.unwrap();
        info!("  Buy rate: {} RUB/USDT", data.buy_rate);
        info!("  Sell rate: {} RUB/USDT", data.sell_rate);
        info!("  Spread: {} RUB", data.spread);
        
        // Validate rates
        assert!(data.buy_rate > 0.0, "Buy rate should be positive");
        assert!(data.sell_rate > 0.0, "Sell rate should be positive");
        assert!(data.buy_rate > data.sell_rate, "Buy rate should be higher than sell rate");
        assert_eq!(data.amount_rub, amount, "Amount should match request");
    }
}