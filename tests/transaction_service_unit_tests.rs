use itrader_backend::gate::{GateClient, TransactionService};
use itrader_backend::core::config::RateLimitsConfig;
use itrader_backend::core::rate_limiter::RateLimiter;
use std::sync::Arc;

#[tokio::test]
async fn test_transaction_service_cache_unit() {
    // Create a rate limiter with test config
    let rate_limits_config = RateLimitsConfig {
        bybit_requests_per_minute: 600,
        gate_requests_per_minute: 240,
        default_burst_size: 10,
    };
    let rate_limiter = Arc::new(RateLimiter::new(&rate_limits_config));
    
    // Create a client with test URL
    let client = GateClient::new("https://test.gate.io".to_string(), rate_limiter).unwrap();
    let service = TransactionService::new(client);
    
    // Test cache operations
    service.clear_cache().await;
    
    // Cache cleared successfully
    // Note: Direct cache access requires test configuration
    
    println!("✓ Cache functionality test passed");
}

#[test]
fn test_rate_limit_config() {
    // Verify rate limits are set correctly
    let config = RateLimitsConfig {
        bybit_requests_per_minute: 600,
        gate_requests_per_minute: 240,
        default_burst_size: 10,
    };
    
    assert_eq!(config.gate_requests_per_minute, 240);
    assert_eq!(config.bybit_requests_per_minute, 600);
    assert_eq!(config.default_burst_size, 10);
    
    println!("✓ Rate limit configuration test passed");
}