use anyhow::Result;
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Load credentials
    let creds_data = tokio::fs::read_to_string("test_data/gate_creditials.json").await?;
    let creds: serde_json::Value = serde_json::from_str(&creds_data)?;
    
    let login = creds["login"].as_str().unwrap();
    let password = creds["password"].as_str().unwrap();
    
    println!("Testing login with email: {}", login);
    
    let client = Client::builder()
        .cookie_store(true)
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;
    
    let request_body = json!({
        "login": login,
        "password": password
    });
    
    let url = "https://panel.gate.cx/api/v1/auth/basic/login";
    println!("Sending request to: {}", url);
    
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;
    
    let status = response.status();
    println!("Response status: {}", status);
    
    // Print headers
    println!("\nResponse headers:");
    for (key, value) in response.headers() {
        println!("  {}: {:?}", key, value);
    }
    
    // Get response text
    let response_text = response.text().await?;
    println!("\nResponse body length: {} bytes", response_text.len());
    
    if response_text.len() < 1000 {
        println!("Response body:\n{}", response_text);
    } else {
        println!("Response body (first 1000 chars):\n{}", &response_text[..1000]);
    }
    
    // Try to parse as JSON
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response_text) {
        println!("\nParsed as JSON:");
        println!("{}", serde_json::to_string_pretty(&json_value)?);
    } else {
        println!("\nResponse is not valid JSON");
    }
    
    Ok(())
}