use std::process::{Command, Stdio};
use std::io::Write;
use serde::{Deserialize, Serialize};
use serde_json;

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

#[test]
fn test_python_script_rates() {
    println!("Testing Python script for Bybit rates");
    
    // Prepare request
    let request = GetRatesRequest {
        amount_rub: 10000.0,
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
    drop(stdin);  // Close stdin
    
    // Wait for response
    let output = child.wait_with_output().expect("Failed to wait for Python script");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Python script failed: {}", stderr);
    }
    
    // Parse response
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Python response: {}", stdout);
    
    let response: RatesResponse = serde_json::from_str(&stdout)
        .expect("Failed to parse Python script response");
    
    // Check response
    assert!(response.success, "Rate fetch failed: {:?}", response.error);
    assert!(response.data.is_some(), "No data in response");
    
    let data = response.data.unwrap();
    println!("Buy rate: {} RUB/USDT", data.buy_rate);
    println!("Sell rate: {} RUB/USDT", data.sell_rate);
    println!("Spread: {} RUB", data.spread);
    
    // Validate rates
    assert!(data.buy_rate > 0.0, "Buy rate should be positive");
    assert!(data.sell_rate > 0.0, "Sell rate should be positive");
    assert!(data.buy_rate > data.sell_rate, "Buy rate should be higher than sell rate");
}

#[derive(Serialize)]
struct CreateAdRequest {
    api_key: String,
    api_secret: String,
    testnet: bool,
    ad_params: AdParams,
}

#[derive(Serialize)]
struct AdParams {
    side: String,
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

#[test] 
fn test_python_script_create_ad() {
    println!("Testing Python script for creating Bybit ad");
    
    // Prepare request
    let request = CreateAdRequest {
        api_key: "test_api_key".to_string(),
        api_secret: "test_api_secret".to_string(),
        testnet: true,
        ad_params: AdParams {
            side: "0".to_string(),  // Sell
            currency: "RUB".to_string(),
            price: "98.50".to_string(),
            quantity: "10".to_string(),
            min_amount: "1000".to_string(),
            max_amount: "5000".to_string(),
            payment_methods: vec!["582".to_string()],
            remarks: "Test ad from Rust".to_string(),
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
    drop(stdin);  // Close stdin
    
    // Wait for response
    let output = child.wait_with_output().expect("Failed to wait for Python script");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Python script failed: {}", stderr);
    }
    
    // Parse response
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Python response: {}", stdout);
    
    let response: BybitResponse = serde_json::from_str(&stdout)
        .expect("Failed to parse Python script response");
    
    // Check response
    assert_eq!(response.ret_code, 0, "API error: {}", response.ret_msg);
    assert!(response.result.is_some(), "No result in response");
    
    let result = response.result.unwrap();
    println!("Created ad with ID: {}", result.ad_id);
    assert!(!result.ad_id.is_empty(), "Ad ID should not be empty");
    assert_eq!(result.status, "online", "Ad status should be online");
}