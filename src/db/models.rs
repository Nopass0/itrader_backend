use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GateAccount {
    pub id: i32,
    pub email: String,
    pub password_encrypted: String,
    pub cookies: Option<serde_json::Value>,
    pub last_auth: Option<DateTime<Utc>>,
    pub balance: Decimal,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BybitAccount {
    pub id: i32,
    pub account_name: String,
    pub api_key: String,
    pub api_secret_encrypted: String,
    pub active_ads: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub gate_transaction_id: String,
    pub bybit_order_id: Option<String>,
    pub gate_account_id: Option<i32>,
    pub bybit_account_id: Option<i32>,
    pub amount: Decimal,
    pub currency: String,
    pub fiat_currency: String,
    pub rate: Decimal,
    pub total_fiat: Decimal,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderPool {
    pub id: i32,
    pub pool_type: String,
    pub order_id: Uuid,
    pub data: serde_json::Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AIConversation {
    pub id: i32,
    pub order_id: Uuid,
    pub messages: serde_json::Value,
    pub customer_language: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailReceipt {
    pub id: i32,
    pub order_id: Uuid,
    pub email_from: String,
    pub email_subject: Option<String>,
    pub receipt_data: serde_json::Value,
    pub ocr_result: Option<serde_json::Value>,
    pub is_valid: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatus {
    pub status: String,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

impl Order {
    pub fn new(
        gate_transaction_id: String,
        amount: Decimal,
        currency: String,
        fiat_currency: String,
        rate: Decimal,
        total_fiat: Decimal,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            gate_transaction_id,
            bybit_order_id: None,
            gate_account_id: None,
            bybit_account_id: None,
            amount,
            currency,
            fiat_currency,
            rate,
            total_fiat,
            status: "pending".to_string(),
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
        }
    }
}