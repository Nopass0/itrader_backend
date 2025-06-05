use anyhow::Result;
use itrader_backend::gate::models::Cookie;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Creating test cookies for Gate.io...");
    
    // Create sample cookies (these would normally come from a successful login)
    let cookies = vec![
        Cookie {
            domain: ".gate.cx".to_string(),
            expiration_date: Some(1764446400.0), // Some future date
            host_only: false,
            http_only: true,
            name: "session_id".to_string(),
            path: "/".to_string(),
            same_site: Some("Lax".to_string()),
            secure: true,
            session: false,
            store_id: None,
            value: "test_session_value".to_string(),
        },
        Cookie {
            domain: ".gate.cx".to_string(),
            expiration_date: Some(1764446400.0),
            host_only: false,
            http_only: true,
            name: "auth_token".to_string(),
            path: "/".to_string(),
            same_site: Some("Lax".to_string()),
            secure: true,
            session: false,
            store_id: None,
            value: "test_auth_value".to_string(),
        },
    ];
    
    // Save cookies to file
    let cookie_json = serde_json::to_string_pretty(&cookies)?;
    fs::write("test_data/gate_cookie.json", cookie_json).await?;
    
    println!("✓ Saved {} test cookies to test_data/gate_cookie.json", cookies.len());
    println!("Note: These are dummy cookies for testing. You'll need real cookies from a successful login.");
    
    Ok(())
}