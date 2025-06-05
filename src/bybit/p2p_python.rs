// Stub for Python P2P client - disabled when python-sdk feature is not enabled
use std::sync::Arc;
use anyhow::Result;
use rust_decimal::Decimal;
use tracing::{info, debug, warn};
use uuid::Uuid;
use serde_json;

use crate::core::rate_limiter::RateLimiter;
use crate::utils::error::AppError;
use super::models::*;

/// Stub implementation of Bybit P2P client
pub struct BybitP2PClient {
    api_key: String,
    _api_secret: String,
    _base_url: String,
    _rate_limiter: Arc<RateLimiter>,
    _max_ads_per_account: u32,
}

impl BybitP2PClient {
    pub async fn new(
        base_url: String,
        api_key: String,
        api_secret: String,
        rate_limiter: Arc<RateLimiter>,
        max_ads_per_account: u32,
    ) -> Result<Self> {
        warn!("Using stub Bybit P2P client - Python SDK disabled");
        
        Ok(Self {
            api_key,
            _api_secret: api_secret,
            _base_url: base_url,
            _rate_limiter: rate_limiter,
            _max_ads_per_account: max_ads_per_account,
        })
    }

    pub async fn get_server_time(&self) -> Result<i64> {
        Ok(chrono::Utc::now().timestamp_millis())
    }

    pub async fn get_account_info(&self) -> Result<AccountInfo> {
        Err(AppError::NotImplemented("Python SDK disabled".to_string()).into())
    }

    pub async fn get_active_ads_count(&self) -> Result<u32> {
        Ok(0)
    }

    pub async fn is_account_available(&self) -> Result<bool> {
        Ok(false)
    }

    pub async fn create_advertisement(&self, _params: AdParams) -> Result<Advertisement> {
        Err(AppError::NotImplemented("Python SDK disabled".to_string()).into())
    }

    pub async fn create_sell_ad_from_transaction(
        &self,
        transaction: &crate::gate::models::GateTransaction,
        rate: Decimal,
    ) -> Result<Advertisement> {
        // Mock implementation - simulate successful ad creation
        info!("🎯 MOCK: Creating Bybit advertisement for transaction {}", transaction.id);
        info!("💰 Amount: {} USDT", transaction.amount);
        info!("💱 Rate: {} RUB/USDT", rate);
        info!("💵 Total: {} RUB", transaction.amount * rate);
        
        // Generate mock ad ID
        let mock_ad_id = format!("MOCK_AD_{}", uuid::Uuid::new_v4());
        
        info!("✅ MOCK: Successfully created advertisement with ID: {}", mock_ad_id);
        info!("📢 This is a simulated response - actual Bybit integration disabled");
        
        // Return mock advertisement
        Ok(Advertisement {
            id: mock_ad_id,
            user_id: "mock_user_123".to_string(),
            asset: "USDT".to_string(),
            fiat: "RUB".to_string(),
            price: rate,
            amount: transaction.amount,
            min_amount: Decimal::from(100),
            max_amount: transaction.amount,
            status: "active".to_string(),
            payment_methods: vec![PaymentMethod {
                id: "tbank".to_string(),
                name: "T-Bank".to_string(),
                account_info: None,
            }],
            remarks: Some(format!("Mock ad for Gate transaction {}", transaction.id)),
            created_at: chrono::Utc::now(),
        })
    }

    pub async fn get_my_advertisements(&self) -> Result<Vec<Advertisement>> {
        Ok(vec![])
    }

    pub async fn get_all_my_advertisements(&self) -> Result<Vec<Advertisement>> {
        Ok(vec![])
    }

    pub async fn get_active_advertisements(&self) -> Result<Vec<Advertisement>> {
        Ok(vec![])
    }

    pub async fn get_advertisement_orders(&self, _ad_id: &str) -> Result<Vec<P2POrder>> {
        Ok(vec![])
    }

    pub async fn get_all_order_chats(&self, _ad_id: &str) -> Result<Vec<OrderChat>> {
        Ok(vec![])
    }

    pub async fn delete_advertisement(&self, _ad_id: &str) -> Result<()> {
        Err(AppError::NotImplemented("Python SDK disabled".to_string()).into())
    }

    pub async fn get_order(&self, _order_id: &str) -> Result<P2POrder> {
        Err(AppError::NotImplemented("Python SDK disabled".to_string()).into())
    }

    pub async fn get_active_orders(&self) -> Result<Vec<P2POrder>> {
        Ok(vec![])
    }

    pub async fn send_chat_message(&self, _order_id: &str, _message: &str) -> Result<()> {
        Err(AppError::NotImplemented("Python SDK disabled".to_string()).into())
    }

    pub async fn send_message(&self, _order_id: &str, _message: &str) -> Result<()> {
        Err(AppError::NotImplemented("Python SDK disabled".to_string()).into())
    }

    pub async fn release_order(&self, _order_id: &str) -> Result<()> {
        Err(AppError::NotImplemented("Python SDK disabled".to_string()).into())
    }

    pub async fn get_chat_messages(&self, _order_id: &str) -> Result<Vec<ChatMessage>> {
        Ok(vec![])
    }

    pub async fn confirm_payment_received(&self, _order_id: &str) -> Result<()> {
        Err(AppError::NotImplemented("Python SDK disabled".to_string()).into())
    }

    pub async fn cancel_order(&self, _order_id: &str) -> Result<()> {
        Err(AppError::NotImplemented("Python SDK disabled".to_string()).into())
    }

    pub async fn create_order(&self, _ad_id: &str, _amount: &str) -> Result<P2POrder> {
        Err(AppError::NotImplemented("Python SDK disabled".to_string()).into())
    }

    pub async fn get_payment_methods(&self) -> Result<Vec<PaymentMethod>> {
        Ok(vec![])
    }

    pub async fn monitor_order_status(&self, _order_id: &str) -> Result<P2POrder> {
        Err(AppError::NotImplemented("Python SDK disabled".to_string()).into())
    }

    pub async fn get_order_count(&self) -> Result<u32> {
        Ok(0)
    }

    pub async fn is_authenticated(&self) -> bool {
        false
    }
}