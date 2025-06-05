use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GateResponse<T> {
    pub success: bool,
    pub response: Option<T>,
    pub error: Option<String>,
    pub code: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginResponse {
    pub user_id: String,
    pub session_id: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GateTransaction {
    pub id: String,
    pub order_id: String,
    pub amount: Decimal,
    pub currency: String,
    pub fiat_currency: String,
    pub fiat_amount: Decimal,
    pub rate: Decimal,
    pub status: i32, // 1 = Pending, 2 = In Progress, 3 = Completed
    pub buyer_name: String,
    pub payment_method: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BalanceRequest {
    pub currency: String,
    pub amount: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BalanceResponse {
    pub currency: String,
    pub balance: Decimal,
    pub available: Decimal,
    pub locked: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AcceptTransactionRequest {
    pub transaction_id: String,
    pub action: String, // "accept"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompleteTransactionRequest {
    pub transaction_id: String,
    pub action: String, // "complete"
    pub confirmation_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cookie {
    pub domain: String,
    #[serde(rename = "expirationDate")]
    pub expiration_date: Option<f64>,
    #[serde(rename = "hostOnly")]
    pub host_only: bool,
    #[serde(rename = "httpOnly")]
    pub http_only: bool,
    pub name: String,
    pub path: String,
    #[serde(rename = "sameSite")]
    pub same_site: Option<String>,
    pub secure: bool,
    pub session: bool,
    #[serde(rename = "storeId")]
    pub store_id: Option<String>,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionFilter {
    pub status: Option<i32>,
    pub currency: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserWallet {
    pub balance: String,
    pub currency: Currency,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Currency {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub id: i64,
    pub name: String,
    pub login: Option<String>,  // This is the email field, but it can be null
    pub wallets: Vec<UserWallet>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthMeResponse {
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PayoutsResponse {
    pub payouts: PayoutsData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PayoutsData {
    pub data: Vec<Payout>,
    pub meta: Option<PayoutsMeta>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PayoutsMeta {
    pub current_page: Option<u32>,
    pub last_page: Option<u32>,
    pub per_page: Option<u32>,
    pub total: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Payout {
    pub id: i64,
    pub payment_method_id: Option<i32>,
    pub status: i32,
    pub wallet: String,
    pub amount: PayoutAmount,
    pub total: PayoutAmount,
    pub method: PaymentMethod,
    pub meta: Option<PayoutMeta>,
    pub approved_at: Option<String>,
    pub expired_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub attachments: Option<Vec<PayoutAttachment>>,
    pub trader: Option<Trader>,
    pub bank: Option<Bank>,
    pub tooltip: Option<Tooltip>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PayoutAmount {
    pub trader: std::collections::HashMap<String, serde_json::Value>,
}

// Custom deserializer for PayoutAmount that handles both objects and empty arrays
impl<'de> Deserialize<'de> for PayoutAmount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        
        match value {
            // If it's an object with "trader" field
            serde_json::Value::Object(mut map) => {
                if let Some(trader_value) = map.remove("trader") {
                    if let serde_json::Value::Object(trader_map) = trader_value {
                        let mut result = std::collections::HashMap::new();
                        for (k, v) in trader_map {
                            result.insert(k, v);
                        }
                        return Ok(PayoutAmount { trader: result });
                    }
                }
                // If no trader field or it's not an object, return empty
                Ok(PayoutAmount { trader: std::collections::HashMap::new() })
            }
            // If it's an empty array, return empty HashMap
            serde_json::Value::Array(_) => {
                Ok(PayoutAmount { trader: std::collections::HashMap::new() })
            }
            // For any other case, return empty
            _ => Ok(PayoutAmount { trader: std::collections::HashMap::new() })
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentMethod {
    pub id: Option<i32>,
    #[serde(rename = "type")]
    pub method_type: Option<i32>,
    pub name: Option<i32>,
    pub label: String,
    pub status: Option<i32>,
    pub payment_provider_id: Option<i32>,
    pub wallet_currency_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PayoutMeta {
    pub bank: Option<String>,
    pub card_number: Option<String>,
    pub courses: Option<serde_json::Value>,
    pub reason: Option<PayoutReason>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PayoutReason {
    pub trader: Option<String>,
    pub support: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PayoutAttachment {
    pub name: String,
    pub file_name: String,
    pub original_url: String,
    pub extension: String,
    pub size: i64,
    pub created_at: String,
    pub custom_properties: Option<AttachmentProperties>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttachmentProperties {
    pub fake: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trader {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bank {
    pub id: Option<i32>,
    pub name: String,
    pub code: String,
    pub label: String,
    #[serde(default)]
    pub active: bool,
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tooltip {
    pub payments: Option<TooltipPayments>,
    pub reasons: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TooltipPayments {
    pub success: Option<i32>,
    pub rejected: Option<i32>,
    pub percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutResponse {
    pub payout: Payout,
}