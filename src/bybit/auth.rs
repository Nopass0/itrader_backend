use hmac::{Hmac, Mac};
use sha2::Sha256;
use chrono::Utc;
use serde::Serialize;
use std::collections::BTreeMap;

use crate::utils::error::{AppError, Result};

type HmacSha256 = Hmac<Sha256>;

pub struct BybitAuth {
    api_key: String,
    api_secret: String,
}

impl BybitAuth {
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            api_key,
            api_secret,
        }
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn generate_signature(&self, params: &impl Serialize, timestamp: &str) -> Result<String> {
        // Convert params to sorted map for consistent ordering
        let json_value = serde_json::to_value(params)
            .map_err(|e| AppError::Config(format!("Failed to serialize params: {}", e)))?;
        
        let mut sorted_params = BTreeMap::new();
        if let serde_json::Value::Object(map) = json_value {
            for (k, v) in map {
                match v {
                    serde_json::Value::String(s) => {
                        sorted_params.insert(k, s);
                    }
                    _ => {
                        sorted_params.insert(k, v.to_string());
                    }
                }
            }
        }

        // Build query string
        let query_string = sorted_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        // Create signature payload: timestamp + api_key + query_string
        let payload = format!("{}{}{}", timestamp, self.api_key, query_string);

        // Generate HMAC-SHA256
        let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes())
            .map_err(|e| AppError::Config(format!("Invalid API secret: {}", e)))?;
        mac.update(payload.as_bytes());
        
        let result = mac.finalize();
        let signature = hex::encode(result.into_bytes());

        Ok(signature)
    }

    pub fn generate_ws_signature(&self) -> Result<(String, String)> {
        let expires = (Utc::now().timestamp_millis() + 10000).to_string();
        let payload = format!("GET/realtime{}", expires);

        let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes())
            .map_err(|e| AppError::Config(format!("Invalid API secret: {}", e)))?;
        mac.update(payload.as_bytes());
        
        let result = mac.finalize();
        let signature = hex::encode(result.into_bytes());

        Ok((expires, signature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_signature_generation() {
        let auth = BybitAuth::new("test_key".to_string(), "test_secret".to_string());
        let params = json!({
            "symbol": "BTCUSDT",
            "side": "Buy",
            "orderType": "Limit",
            "qty": "0.01",
            "price": "50000"
        });
        
        let timestamp = "1234567890123";
        let signature = auth.generate_signature(&params, timestamp).unwrap();
        
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64); // SHA256 produces 64 hex characters
    }
}