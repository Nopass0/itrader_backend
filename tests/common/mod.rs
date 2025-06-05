use std::sync::Arc;
use itrader_backend::core::config::Config;
use itrader_backend::core::rate_limiter::RateLimiter;

pub fn setup() {
    // Set up test environment
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUN_MODE", "test");
    let _ = tracing_subscriber::fmt::try_init();
}

pub fn get_test_config() -> Config {
    // Load test configuration
    dotenv::from_filename(".env.test").ok();
    Config::load().expect("Failed to load test config")
}

pub fn create_rate_limiter() -> Arc<RateLimiter> {
    let config = get_test_config();
    Arc::new(RateLimiter::new(&config.rate_limits))
}