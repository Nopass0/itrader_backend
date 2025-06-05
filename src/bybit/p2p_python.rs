use std::sync::Arc;
use anyhow::{Result, Context};
use rust_decimal::Decimal;
use tracing::{info, debug, warn, error};
use serde_json::json;

use crate::core::rate_limiter::RateLimiter;
use crate::utils::error::AppError;
use super::models::*;
use super::python_bridge::PyBybitClient;

/// Bybit P2P client implementation using Python SDK bridge
pub struct BybitP2PClient {
    api_key: String,
    api_secret: String,
    base_url: String,
    rate_limiter: Arc<RateLimiter>,
    max_ads_per_account: u32,
    py_client: Option<PyBybitClient>,
}

impl BybitP2PClient {
    /// Create a new Bybit P2P client with Python SDK integration
    pub async fn new(
        base_url: String,
        api_key: String,
        api_secret: String,
        rate_limiter: Arc<RateLimiter>,
        max_ads_per_account: u32,
    ) -> Result<Self> {
        info!("Initializing Bybit P2P client with Python SDK bridge");
        
        // Try to create Python client
        let py_client = match PyBybitClient::new(api_key.clone(), api_secret.clone()).await {
            Ok(client) => {
                info!("✅ Successfully initialized Python Bybit client");
                Some(client)
            }
            Err(e) => {
                error!("Failed to initialize Python client: {}", e);
                warn!("⚠️ Bybit P2P client will operate in limited mode");
                None
            }
        };
        
        Ok(Self {
            api_key,
            api_secret,
            base_url,
            rate_limiter,
            max_ads_per_account,
            py_client,
        })
    }

    /// Get server time
    pub async fn get_server_time(&self) -> Result<i64> {
        // For now, use local time as server time is not critical
        Ok(chrono::Utc::now().timestamp_millis())
    }

    /// Get account information
    pub async fn get_account_info(&self) -> Result<AccountInfo> {
        self.rate_limiter.acquire("bybit_api").await?;
        
        match &self.py_client {
            Some(client) => {
                debug!("Getting account info via Python SDK");
                client.get_account_info().await
                    .context("Failed to get account info from Python SDK")
            }
            None => Err(AppError::NotImplemented("Python SDK not available".to_string()).into())
        }
    }

    /// Get count of active advertisements
    pub async fn get_active_ads_count(&self) -> Result<u32> {
        let ads = self.get_active_advertisements().await?;
        Ok(ads.len() as u32)
    }

    /// Check if account is available for creating ads
    pub async fn is_account_available(&self) -> Result<bool> {
        let count = self.get_active_ads_count().await?;
        Ok(count < self.max_ads_per_account)
    }

    /// Create a new advertisement
    pub async fn create_advertisement(&self, params: AdParams) -> Result<Advertisement> {
        self.rate_limiter.acquire("bybit_api").await?;
        
        match &self.py_client {
            Some(client) => {
                info!("Creating advertisement via Python SDK");
                debug!("Ad params: {:?}", params);
                
                // Convert AdParams to JSON for Python
                let py_params = json!({
                    "asset": params.asset,
                    "fiat": params.fiat,
                    "price": params.price.to_string(),
                    "amount": params.amount.to_string(),
                    "min_amount": params.min_amount.to_string(),
                    "max_amount": params.max_amount.to_string(),
                    "payment_methods": params.payment_methods,
                    "remarks": params.remarks,
                });
                
                client.create_ad(py_params).await
                    .context("Failed to create advertisement via Python SDK")
            }
            None => Err(AppError::NotImplemented("Python SDK not available".to_string()).into())
        }
    }

    /// Create a sell advertisement from a Gate transaction
    pub async fn create_sell_ad_from_transaction(
        &self,
        transaction: &crate::gate::models::GateTransaction,
        rate: Decimal,
    ) -> Result<Advertisement> {
        info!("🎯 Creating Bybit advertisement for transaction {}", transaction.id);
        info!("💰 Amount: {} USDT", transaction.amount);
        info!("💱 Rate: {} RUB/USDT", rate);
        info!("💵 Total: {} RUB", transaction.amount * rate);
        
        // Calculate amounts
        let total_rub = transaction.amount * rate;
        let min_amount = Decimal::from(100); // Minimum 100 RUB
        let max_amount = total_rub; // Maximum is the full amount
        
        // Create ad parameters
        let params = AdParams {
            asset: "USDT".to_string(),
            fiat: "RUB".to_string(),
            side: "1".to_string(), // 1 = Sell
            price: rate,
            amount: transaction.amount,
            min_amount,
            max_amount,
            payment_methods: vec!["75".to_string()], // Tinkoff Bank
            remarks: Some(format!("Order from Gate.io transaction {}", transaction.id)),
        };
        
        // Create the advertisement
        let ad = self.create_advertisement(params).await?;
        
        info!("✅ Successfully created advertisement with ID: {}", ad.id);
        
        Ok(ad)
    }

    /// Get user's advertisements
    pub async fn get_my_advertisements(&self) -> Result<Vec<Advertisement>> {
        self.rate_limiter.acquire("bybit_api").await?;
        
        match &self.py_client {
            Some(client) => {
                debug!("Getting advertisements via Python SDK");
                client.get_active_ads().await
                    .context("Failed to get advertisements from Python SDK")
            }
            None => Ok(vec![])
        }
    }

    /// Get all user's advertisements (alias for get_my_advertisements)
    pub async fn get_all_my_advertisements(&self) -> Result<Vec<Advertisement>> {
        self.get_my_advertisements().await
    }

    /// Get active advertisements
    pub async fn get_active_advertisements(&self) -> Result<Vec<Advertisement>> {
        let all_ads = self.get_my_advertisements().await?;
        Ok(all_ads.into_iter()
            .filter(|ad| ad.status == "1" || ad.status == "active")
            .collect())
    }

    /// Get orders for an advertisement
    pub async fn get_advertisement_orders(&self, ad_id: &str) -> Result<Vec<P2POrder>> {
        self.rate_limiter.acquire("bybit_api").await?;
        
        // Get all active orders and filter by ad_id
        let all_orders = self.get_active_orders().await?;
        Ok(all_orders.into_iter()
            .filter(|order| order.ad_id == ad_id)
            .collect())
    }

    /// Get all order chats for an advertisement
    pub async fn get_all_order_chats(&self, ad_id: &str) -> Result<Vec<OrderChat>> {
        let orders = self.get_advertisement_orders(ad_id).await?;
        
        let mut chats = Vec::new();
        for order in orders {
            if let Ok(messages) = self.get_chat_messages(&order.id).await {
                chats.push(OrderChat {
                    order_id: order.id.clone(),
                    messages,
                });
            }
        }
        
        Ok(chats)
    }

    /// Delete an advertisement
    pub async fn delete_advertisement(&self, ad_id: &str) -> Result<()> {
        self.rate_limiter.acquire("bybit_api").await?;
        
        match &self.py_client {
            Some(client) => {
                info!("Deleting advertisement {} via Python SDK", ad_id);
                client.delete_ad(ad_id).await
                    .context("Failed to delete advertisement via Python SDK")
            }
            None => Err(AppError::NotImplemented("Python SDK not available".to_string()).into())
        }
    }

    /// Get a specific order
    pub async fn get_order(&self, order_id: &str) -> Result<P2POrder> {
        self.rate_limiter.acquire("bybit_api").await?;
        
        match &self.py_client {
            Some(client) => {
                debug!("Getting order {} via Python SDK", order_id);
                client.get_order(order_id).await
                    .context("Failed to get order from Python SDK")
            }
            None => Err(AppError::NotImplemented("Python SDK not available".to_string()).into())
        }
    }

    /// Get active orders
    pub async fn get_active_orders(&self) -> Result<Vec<P2POrder>> {
        self.rate_limiter.acquire("bybit_api").await?;
        
        match &self.py_client {
            Some(client) => {
                debug!("Getting active orders via Python SDK");
                client.get_active_orders().await
                    .context("Failed to get active orders from Python SDK")
            }
            None => Ok(vec![])
        }
    }

    /// Send a chat message (alias for send_message)
    pub async fn send_chat_message(&self, order_id: &str, message: &str) -> Result<()> {
        self.send_message(order_id, message).await
    }

    /// Send a message in order chat
    pub async fn send_message(&self, order_id: &str, message: &str) -> Result<()> {
        self.rate_limiter.acquire("bybit_api").await?;
        
        match &self.py_client {
            Some(client) => {
                info!("Sending message to order {} via Python SDK", order_id);
                client.send_message(order_id, message).await
                    .context("Failed to send message via Python SDK")
            }
            None => Err(AppError::NotImplemented("Python SDK not available".to_string()).into())
        }
    }

    /// Release an order
    pub async fn release_order(&self, order_id: &str) -> Result<()> {
        self.rate_limiter.acquire("bybit_api").await?;
        
        match &self.py_client {
            Some(client) => {
                info!("Releasing order {} via Python SDK", order_id);
                client.release_order(order_id).await
                    .context("Failed to release order via Python SDK")
            }
            None => Err(AppError::NotImplemented("Python SDK not available".to_string()).into())
        }
    }

    /// Get chat messages for an order
    pub async fn get_chat_messages(&self, order_id: &str) -> Result<Vec<ChatMessage>> {
        self.rate_limiter.acquire("bybit_api").await?;
        
        match &self.py_client {
            Some(client) => {
                debug!("Getting chat messages for order {} via Python SDK", order_id);
                client.get_chat_messages(order_id).await
                    .context("Failed to get chat messages from Python SDK")
            }
            None => Ok(vec![])
        }
    }

    /// Confirm payment received (not implemented in Python SDK yet)
    pub async fn confirm_payment_received(&self, _order_id: &str) -> Result<()> {
        Err(AppError::NotImplemented("Confirm payment not implemented in Python SDK".to_string()).into())
    }

    /// Cancel an order (not implemented in Python SDK yet)
    pub async fn cancel_order(&self, _order_id: &str) -> Result<()> {
        Err(AppError::NotImplemented("Cancel order not implemented in Python SDK".to_string()).into())
    }

    /// Create an order (buyer side - not implemented)
    pub async fn create_order(&self, _ad_id: &str, _amount: &str) -> Result<P2POrder> {
        Err(AppError::NotImplemented("Create order not implemented in Python SDK".to_string()).into())
    }

    /// Get payment methods
    pub async fn get_payment_methods(&self) -> Result<Vec<PaymentMethod>> {
        // Return common payment methods
        Ok(vec![
            PaymentMethod {
                id: "75".to_string(),
                name: "Tinkoff".to_string(),
                account_info: None,
            },
            PaymentMethod {
                id: "382".to_string(),
                name: "СБП (SBP)".to_string(),
                account_info: None,
            },
        ])
    }

    /// Monitor order status
    pub async fn monitor_order_status(&self, order_id: &str) -> Result<P2POrder> {
        self.get_order(order_id).await
    }

    /// Get total order count
    pub async fn get_order_count(&self) -> Result<u32> {
        let orders = self.get_active_orders().await?;
        Ok(orders.len() as u32)
    }

    /// Check if client is authenticated
    pub async fn is_authenticated(&self) -> bool {
        self.py_client.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Requires Python environment
    async fn test_python_p2p_client() {
        let rate_limiter = Arc::new(RateLimiter::new(240, 60));
        
        let client = BybitP2PClient::new(
            "https://api.bybit.com".to_string(),
            "test_key".to_string(),
            "test_secret".to_string(),
            rate_limiter,
            5,
        ).await;
        
        assert!(client.is_ok());
        let client = client.unwrap();
        assert!(client.is_authenticated().await);
    }
}