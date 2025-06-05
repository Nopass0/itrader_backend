use itrader_backend::core::rate_limiter::RateLimiter;
use itrader_backend::core::config::RateLimitsConfig;
use itrader_backend::gate::client::GateClient;
use std::sync::Arc;
use anyhow::Result;

/// Create a Gate client with minimal configuration
/// This avoids loading the full Config which requires admin_token
pub fn create_gate_client() -> Result<GateClient> {
    let base_url = std::env::var("GATE_API_URL")
        .unwrap_or_else(|_| "https://panel.gate.cx/api/v1".to_string());
    
    let rate_limits = RateLimitsConfig {
        gate_requests_per_minute: 240,
        bybit_requests_per_minute: 120,
        default_burst_size: 10,
    };
    
    let rate_limiter = Arc::new(RateLimiter::new(&rate_limits));
    let client = GateClient::new(base_url, rate_limiter)?;
    
    Ok(client)
}

/// Get the default cookie file path
pub fn get_cookie_file() -> String {
    std::env::var("COOKIE_FILE").unwrap_or_else(|_| ".gate_cookies.json".to_string())
}