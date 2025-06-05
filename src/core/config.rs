use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub gate: GateConfig,
    pub bybit: BybitConfig,
    pub ai: AIConfig,
    pub rate_limits: RateLimitsConfig,
    pub email: EmailConfig,
    pub ocr: OCRConfig,
    pub monitoring: MonitoringConfig,
    pub auto_trader: AutoTraderConfig,
    pub admin_token: String,
    #[serde(default = "default_use_db_storage")]
    pub use_db_storage: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GateConfig {
    pub base_url: String,
    #[serde(default = "default_p2p_url")]
    pub p2p_url: String,
    #[serde(default = "default_panel_url")]
    pub panel_url: String,
    pub session_refresh_interval: u64,
    pub balance_check_interval: u64,
    pub target_balance: f64,
    pub min_balance: f64,
    pub request_timeout: u64,
    pub shutdown_balance: f64,
}

fn default_p2p_url() -> String {
    "https://panel.gate.cx/api/v1".to_string()
}

fn default_panel_url() -> String {
    "https://panel.gate.cx/api/v1".to_string()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BybitConfig {
    pub rest_url: String,
    pub ws_url: String,
    pub p2p_api_version: String,
    pub max_ads_per_account: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AIConfig {
    pub openrouter_api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub response_delay_min: u64,
    pub response_delay_max: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RateLimitsConfig {
    pub gate_requests_per_minute: u32,
    pub bybit_requests_per_minute: u32,
    pub default_burst_size: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmailConfig {
    pub imap_server: String,
    pub imap_port: u16,
    pub email: String,
    pub password: String,
    pub check_interval: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OCRConfig {
    pub tesseract_lang: String,
    pub confidence_threshold: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MonitoringConfig {
    pub metrics_port: u16,
    pub health_check_interval: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AutoTraderConfig {
    pub enabled: bool,
    pub check_interval_secs: u64,
    pub balance_check_interval_hours: u64,
    pub target_balance_rub: f64,
    pub min_order_amount: f64,
    pub max_order_amount: f64,
    pub auto_confirm: bool,
    pub max_concurrent_orders: usize,
    #[serde(default = "default_interactive_mode")]
    pub interactive_mode: bool,
}

fn default_use_db_storage() -> bool {
    true // Default to using database storage
}

fn default_interactive_mode() -> bool {
    true  // Default to interactive mode for safety
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".to_string());

        let s = ConfigBuilder::builder()
            // Start with the default configuration file
            .add_source(File::with_name("config/default"))
            // Add environment-specific configuration
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Add in settings from environment variables (with prefix "APP")
            .add_source(Environment::with_prefix("APP").separator("__"))
            // Override specific fields from environment variables
            .set_override_option("database.url", env::var("DATABASE_URL").ok())?
            .set_override_option("redis.url", env::var("REDIS_URL").ok())?
            .set_override_option("ai.openrouter_api_key", env::var("OPENROUTER_API_KEY").ok())?
            .set_override_option("email.email", env::var("EMAIL_ADDRESS").ok())?
            .set_override_option("email.password", env::var("EMAIL_PASSWORD").ok())?
            .set_override_option("admin_token", env::var("ADMIN_TOKEN").ok())?
            .build()?;

        s.try_deserialize()
    }
}