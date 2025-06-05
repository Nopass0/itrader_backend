// Bybit P2P API Module
// 
// IMPORTANT: Bybit P2P API requires special permissions:
// 1. Account must be a verified P2P advertiser (VA level or above)
// 2. P2P API permissions must be enabled in API Key Management
// 3. Without these permissions, all P2P endpoints will return 404 errors
//
// To enable P2P API access:
// - Become a P2P advertiser on Bybit
// - Complete merchant verification
// - Reach VA (Verified Advertiser) status
// - Enable P2P permissions when creating API keys

use std::sync::Arc;
use rust_decimal::Decimal;
use serde_json::json;
use tracing::{info, debug, warn};

use crate::utils::error::Result;
use crate::core::rate_limiter::RateLimiter;
use crate::gate::models::GateTransaction;
use super::client::BybitClient;
use super::models::*;

/// Bybit P2P Client for managing P2P advertisements and orders
/// 
/// Note: The P2P API requires special permissions - advertisers must be at VA level or above.
/// Make sure your API key has the necessary P2P permissions before using this client.
pub struct BybitP2PClient {
    client: Arc<BybitClient>,
    max_ads_per_account: u32,
}

impl BybitP2PClient {
    pub async fn new(
        base_url: String,
        api_key: String,
        api_secret: String,
        rate_limiter: Arc<RateLimiter>,
        max_ads_per_account: u32,
    ) -> Result<Self> {
        let client = Arc::new(BybitClient::new(base_url, api_key, api_secret, rate_limiter)?);
        
        // Sync server time on initialization
        client.sync_server_time().await?;
        
        Ok(Self {
            client,
            max_ads_per_account,
        })
    }
    
    pub async fn get_server_time(&self) -> Result<i64> {
        debug!("Getting server time");
        let endpoint = "/v5/market/time";
        
        #[derive(serde::Deserialize)]
        struct TimeResponse {
            #[serde(rename = "timeSecond")]
            time_second: String,
            #[serde(rename = "timeNano")]
            time_nano: String,
        }
        
        let response: TimeResponse = self.client.request("GET", endpoint, None::<&()>).await?;
        Ok(response.time_second.parse().unwrap_or(0))
    }

    pub async fn get_account_info(&self) -> Result<AccountInfo> {
        debug!("Getting Bybit account info");
        let endpoint = "/v5/user/query-api";
        self.client.request::<(), AccountInfo>("GET", endpoint, None).await
    }

    pub async fn get_active_ads_count(&self) -> Result<u32> {
        let account_info = self.get_account_info().await?;
        Ok(account_info.active_ads)
    }

    pub async fn is_account_available(&self) -> Result<bool> {
        let active_ads = self.get_active_ads_count().await?;
        Ok(active_ads < self.max_ads_per_account)
    }

    pub async fn create_advertisement(&self, params: AdParams) -> Result<Advertisement> {
        info!("Creating P2P advertisement: {} {} at {} {}", 
            params.amount, params.asset, params.price, params.fiat);
        
        let endpoint = "/p2p/item/create";
        self.client.request("POST", endpoint, Some(&params)).await
    }

    pub async fn create_sell_ad_from_transaction(
        &self,
        transaction: &GateTransaction,
        rate: Decimal,
    ) -> Result<Advertisement> {
        let params = AdParams {
            asset: transaction.currency.clone(),
            fiat: transaction.fiat_currency.clone(),
            price: rate.to_string(),
            amount: transaction.amount.to_string(),
            payment_methods: vec!["1".to_string()], // Bank card
            remarks: Some("Fast release, T-Bank only. Быстрый релиз, только Т-Банк.".to_string()),
            min_amount: Some("1000".to_string()),
            max_amount: Some(transaction.fiat_amount.to_string()),
        };

        self.create_advertisement(params).await
    }

    pub async fn get_my_advertisements(&self) -> Result<Vec<Advertisement>> {
        debug!("Getting my P2P advertisements");
        let endpoint = "/p2p/item/list";
        let params = json!({
            "tokenId": "USDT",
            "currencyId": "RUB"
        });
        
        self.client.request("GET", endpoint, Some(&params)).await
    }
    
    pub async fn get_all_my_advertisements(&self) -> Result<Vec<Advertisement>> {
        debug!("Getting all my P2P advertisements (active and inactive)");
        let endpoint = "/p2p/item/list";
        let params = json!({
            "tokenId": "USDT",
            "currencyId": "RUB",
            "status": "1,2,3" // 1=active, 2=inactive, 3=hidden
        });
        
        let response: AdvertisementsResponse = self.client.request("GET", endpoint, Some(&params)).await?;
        Ok(response.items)
    }
    
    pub async fn get_active_advertisements(&self) -> Result<Vec<Advertisement>> {
        debug!("Getting active P2P advertisements");
        let endpoint = "/p2p/item/list";
        let params = json!({
            "tokenId": "USDT",
            "currencyId": "RUB",
            "status": "1" // Only active
        });
        
        let response: AdvertisementsResponse = self.client.request("GET", endpoint, Some(&params)).await?;
        Ok(response.items)
    }
    
    pub async fn get_advertisement_orders(&self, ad_id: &str) -> Result<Vec<P2POrder>> {
        debug!("Getting orders for advertisement: {}", ad_id);
        let endpoint = "/p2p/order/list";
        let params = json!({
            "itemId": ad_id,
            "limit": 50
        });
        
        let response: OrdersResponse = self.client.request("GET", endpoint, Some(&params)).await?;
        Ok(response.list)
    }
    
    pub async fn get_all_order_chats(&self, ad_id: &str) -> Result<Vec<OrderChat>> {
        debug!("Getting all chats for advertisement: {}", ad_id);
        
        // First get all orders for this advertisement
        let orders = self.get_advertisement_orders(ad_id).await?;
        
        let mut all_chats = Vec::new();
        
        // Then get chat messages for each order
        for order in orders {
            let messages = self.get_chat_messages(&order.id).await?;
            all_chats.push(OrderChat {
                order_id: order.id.clone(),
                order_status: order.status,
                buyer_id: order.buyer_id.clone(),
                seller_id: order.seller_id.clone(),
                messages,
            });
        }
        
        Ok(all_chats)
    }

    pub async fn delete_advertisement(&self, ad_id: &str) -> Result<()> {
        info!("Deleting P2P advertisement: {}", ad_id);
        let endpoint = "/p2p/item/delete";
        let params = json!({
            "itemId": ad_id
        });
        
        self.client.request::<_, serde_json::Value>("POST", endpoint, Some(&params)).await?;
        Ok(())
    }

    pub async fn get_order(&self, order_id: &str) -> Result<P2POrder> {
        debug!("Getting P2P order: {}", order_id);
        let endpoint = "/p2p/order/get";
        let params = json!({
            "orderId": order_id
        });
        
        self.client.request("GET", endpoint, Some(&params)).await
    }

    pub async fn get_active_orders(&self) -> Result<Vec<P2POrder>> {
        debug!("Getting active P2P orders");
        let endpoint = "/p2p/order/list";
        let params = json!({
            "orderStatus": "10,20,30" // PENDING, PAID, APPEAL
        });
        
        self.client.request("GET", endpoint, Some(&params)).await
    }

    pub async fn send_chat_message(&self, order_id: &str, message: &str) -> Result<()> {
        self.send_message(order_id, message).await
    }

    pub async fn send_message(&self, order_id: &str, message: &str) -> Result<()> {
        info!("Sending message to order {}: {}", order_id, message);
        let endpoint = "/p2p/chat/send";
        let params = SendMessageRequest {
            order_id: order_id.to_string(),
            content: message.to_string(),
            message_type: "TEXT".to_string(),
        };
        
        self.client.request::<_, serde_json::Value>("POST", endpoint, Some(&params)).await?;
        Ok(())
    }

    pub async fn release_order(&self, order_id: &str) -> Result<()> {
        info!("Releasing P2P order: {}", order_id);
        let endpoint = "/p2p/order/release";
        let params = json!({
            "orderId": order_id
        });
        
        self.client.request::<_, serde_json::Value>("POST", endpoint, Some(&params)).await?;
        Ok(())
    }


    pub async fn get_chat_messages(&self, order_id: &str) -> Result<Vec<ChatMessage>> {
        debug!("Getting chat messages for order: {}", order_id);
        let endpoint = "/p2p/chat/list";
        let params = json!({
            "orderId": order_id,
            "limit": 50
        });
        
        self.client.request("GET", endpoint, Some(&params)).await
    }

    pub async fn confirm_payment_received(&self, order_id: &str) -> Result<()> {
        info!("Confirming payment received for order: {}", order_id);
        let endpoint = "/p2p/order/confirm";
        let params = json!({
            "orderId": order_id
        });
        
        self.client.request::<_, serde_json::Value>("POST", endpoint, Some(&params)).await?;
        Ok(())
    }


    pub async fn appeal_order(&self, order_id: &str, reason: &str) -> Result<()> {
        warn!("Creating appeal for order {}: {}", order_id, reason);
        let endpoint = "/p2p/order/appeal";
        let params = json!({
            "orderId": order_id,
            "reason": reason
        });
        
        self.client.request::<_, serde_json::Value>("POST", endpoint, Some(&params)).await?;
        Ok(())
    }

    pub async fn monitor_order_status(&self, order_id: &str) -> Result<P2POrder> {
        loop {
            let order = self.get_order(order_id).await?;
            
            match order.status.as_str() {
                "PENDING" => {
                    debug!("Order {} is pending buyer payment", order_id);
                }
                "PAID" => {
                    info!("Order {} has been marked as paid by buyer", order_id);
                    return Ok(order);
                }
                "RELEASED" => {
                    info!("Order {} has been completed", order_id);
                    return Ok(order);
                }
                "CANCELLED" => {
                    warn!("Order {} has been cancelled", order_id);
                    return Ok(order);
                }
                "APPEAL" => {
                    warn!("Order {} is under appeal", order_id);
                    return Ok(order);
                }
                _ => {
                    warn!("Unknown order status: {}", order.status);
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}