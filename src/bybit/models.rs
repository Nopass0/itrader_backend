use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerTimeResponse {
    #[serde(rename = "timeSecond")]
    pub time_second: String,
    #[serde(rename = "timeNano")]
    pub time_nano: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BybitResponse<T> {
    #[serde(rename = "retCode")]
    pub ret_code: i32,
    #[serde(rename = "retMsg")]
    pub ret_msg: String,
    pub result: Option<T>,
    #[serde(rename = "retExtInfo")]
    pub ret_ext_info: Option<serde_json::Value>,
    pub time: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdParams {
    pub asset: String,
    pub fiat: String,
    pub side: String,  // "0" for buy, "1" for sell
    pub price: Decimal,
    pub amount: Decimal,
    #[serde(rename = "paymentMethods")]
    pub payment_methods: Vec<String>,
    pub remarks: Option<String>,
    #[serde(rename = "minAmount")]
    pub min_amount: Decimal,
    #[serde(rename = "maxAmount")]
    pub max_amount: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Advertisement {
    pub id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub asset: String,
    pub fiat: String,
    pub price: Decimal,
    pub amount: Decimal,
    #[serde(rename = "minAmount")]
    pub min_amount: Decimal,
    #[serde(rename = "maxAmount")]
    pub max_amount: Decimal,
    pub status: String,
    #[serde(rename = "paymentMethods")]
    pub payment_methods: Vec<PaymentMethod>,
    pub remarks: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentMethod {
    pub id: String,
    pub name: String,
    #[serde(rename = "accountInfo")]
    pub account_info: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct P2POrder {
    pub id: String,
    #[serde(rename = "adId")]
    pub ad_id: String,
    #[serde(rename = "buyerId")]
    pub buyer_id: String,
    #[serde(rename = "sellerId")]
    pub seller_id: String,
    pub asset: String,
    pub fiat: String,
    pub price: String,
    pub amount: String,
    pub status: String, // "10", "20", "30" etc or "PENDING", "PAID", "RELEASED"
    #[serde(rename = "paymentInfo")]
    pub payment_info: serde_json::Value,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "paidAt")]
    pub paid_at: Option<String>,
    #[serde(rename = "releasedAt")]
    pub released_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub id: String,
    #[serde(rename = "orderId")]
    pub order_id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub content: String,
    pub timestamp: String,
    #[serde(rename = "messageType")]
    pub message_type: String, // "text", "image", "system"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountInfo {
    pub id: String,
    pub nickname: String,
    #[serde(rename = "activeAds")]
    pub active_ads: u32,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateOrderRequest {
    #[serde(rename = "adId")]
    pub ad_id: String,
    pub amount: String,
    #[serde(rename = "paymentId")]
    pub payment_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SendMessageRequest {
    #[serde(rename = "orderId")]
    pub order_id: String,
    pub content: String,
    #[serde(rename = "messageType")]
    pub message_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReleaseOrderRequest {
    #[serde(rename = "orderId")]
    pub order_id: String,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdvertisementsResponse {
    pub items: Vec<Advertisement>,
    #[serde(rename = "totalCount")]
    pub total_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrdersResponse {
    pub list: Vec<P2POrder>,
    #[serde(rename = "totalCount")]
    pub total_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderChat {
    pub order_id: String,
    pub messages: Vec<ChatMessage>,
}