// Stub for Python bridge - disabled when python-sdk feature is not enabled
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::{Result};

use super::models::{AccountInfo, Advertisement, P2POrder};

/// Stub for Python Bybit client
pub struct PyBybitClient {
    _client: Arc<Mutex<String>>,
}

impl PyBybitClient {
    pub async fn new(_api_key: String, _api_secret: String) -> Result<Self> {
        Ok(Self {
            _client: Arc::new(Mutex::new("stub".to_string())),
        })
    }

    pub async fn get_account_info(&self) -> Result<AccountInfo> {
        unimplemented!("Python bridge is disabled")
    }

    pub async fn create_ad(&self, _ad_params: serde_json::Value) -> Result<Advertisement> {
        unimplemented!("Python bridge is disabled")
    }

    pub async fn get_active_ads(&self) -> Result<Vec<Advertisement>> {
        unimplemented!("Python bridge is disabled")
    }

    pub async fn delete_ad(&self, _ad_id: &str) -> Result<()> {
        unimplemented!("Python bridge is disabled")
    }

    pub async fn get_order(&self, _order_id: &str) -> Result<P2POrder> {
        unimplemented!("Python bridge is disabled")
    }

    pub async fn get_active_orders(&self) -> Result<Vec<P2POrder>> {
        unimplemented!("Python bridge is disabled")
    }

    pub async fn send_message(&self, _order_id: &str, _message: &str) -> Result<()> {
        unimplemented!("Python bridge is disabled")
    }

    pub async fn release_order(&self, _order_id: &str) -> Result<()> {
        unimplemented!("Python bridge is disabled")
    }
}