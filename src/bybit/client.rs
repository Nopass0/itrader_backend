use std::sync::Arc;
use std::time::Duration;
use reqwest::{Client, header::{HeaderMap, HeaderValue, CONTENT_TYPE}, StatusCode};
use tracing::{debug, warn, info, error};
use anyhow::Context;
use tokio::sync::RwLock;

use crate::core::rate_limiter::RateLimiter;
use crate::utils::error::{AppError, Result};
use super::auth::BybitAuth;
use super::models::*;

pub struct BybitClient {
    client: Client,
    base_url: String,
    auth: BybitAuth,
    rate_limiter: Arc<RateLimiter>,
    server_time_offset: Arc<RwLock<i64>>, // Offset in milliseconds
}

impl BybitClient {
    pub fn new(
        base_url: String,
        api_key: String,
        api_secret: String,
        rate_limiter: Arc<RateLimiter>,
    ) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::Config(format!("Failed to build HTTP client: {}", e)))?;

        let auth = BybitAuth::new(api_key, api_secret);

        Ok(Self {
            client,
            base_url,
            auth,
            rate_limiter,
            server_time_offset: Arc::new(RwLock::new(0)),
        })
    }

    pub async fn sync_server_time(&self) -> Result<()> {
        let url = format!("{}/v5/market/time", self.base_url);
        let response = self.client.get(&url).send().await
            .context("Failed to get server time")?;
        
        let body: BybitResponse<ServerTimeResponse> = response.json().await
            .context("Failed to parse server time response")?;
        
        if body.ret_code != 0 {
            return Err(AppError::Internal(
                anyhow::anyhow!("Failed to get server time: {}", body.ret_msg)
            ));
        }
        
        if let Some(time_result) = body.result {
            let server_time = time_result.time_second.parse::<i64>()
                .context("Failed to parse server time")? * 1000;
            let local_time = chrono::Utc::now().timestamp_millis();
            let offset = server_time - local_time;
            
            *self.server_time_offset.write().await = offset;
            info!("Server time synchronized. Offset: {} ms", offset);
        }
        
        Ok(())
    }
    
    async fn get_server_timestamp(&self) -> i64 {
        let offset = *self.server_time_offset.read().await;
        chrono::Utc::now().timestamp_millis() + offset
    }

    pub async fn request<T, R>(&self, method: &str, endpoint: &str, params: Option<&T>) -> Result<R>
    where
        T: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        self.rate_limiter.check_and_wait("bybit").await?;

        // Re-sync time if offset is too large (more than 5 seconds)
        let offset = *self.server_time_offset.read().await;
        if offset.abs() > 5000 {
            debug!("Time offset too large ({}ms), re-syncing...", offset);
            if let Err(e) = self.sync_server_time().await {
                warn!("Failed to re-sync server time: {}", e);
            }
        }

        let timestamp = self.get_server_timestamp().await.to_string();
        let url = format!("{}{}", self.base_url, endpoint);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("X-BAPI-API-KEY", HeaderValue::from_str(self.auth.api_key())
            .map_err(|_| AppError::Config("Invalid API key".to_string()))?);
        headers.insert("X-BAPI-TIMESTAMP", HeaderValue::from_str(&timestamp)
            .map_err(|_| AppError::Config("Invalid timestamp".to_string()))?);
        headers.insert("X-BAPI-RECV-WINDOW", HeaderValue::from_static("5000"));

        let request_builder = match method {
            "GET" => {
                if let Some(params) = params {
                    let signature = self.auth.generate_signature(params, &timestamp)?;
                    headers.insert("X-BAPI-SIGN", HeaderValue::from_str(&signature)
                        .map_err(|_| AppError::Config("Invalid signature".to_string()))?);
                    self.client.get(&url).headers(headers).query(params)
                } else {
                    self.client.get(&url).headers(headers)
                }
            }
            "POST" => {
                if let Some(params) = params {
                    let signature = self.auth.generate_signature(params, &timestamp)?;
                    headers.insert("X-BAPI-SIGN", HeaderValue::from_str(&signature)
                        .map_err(|_| AppError::Config("Invalid signature".to_string()))?);
                    self.client.post(&url).headers(headers).json(params)
                } else {
                    self.client.post(&url).headers(headers)
                }
            }
            _ => return Err(AppError::InvalidInput(format!("Unsupported method: {}", method))),
        };

        let response = request_builder
            .send()
            .await
            .context("Failed to send request to Bybit")?;

        self.check_response_status(&response)?;

        let body: BybitResponse<R> = response
            .json()
            .await
            .context("Failed to parse Bybit response")?;

        if body.ret_code != 0 {
            return Err(AppError::Internal(
                anyhow::anyhow!("Bybit API error {}: {}", body.ret_code, body.ret_msg)
            ));
        }

        body.result.ok_or_else(|| {
            AppError::Internal(
                anyhow::anyhow!("Empty response from Bybit API")
            )
        })
    }

    fn check_response_status(&self, response: &reqwest::Response) -> Result<()> {
        match response.status() {
            StatusCode::OK => Ok(()),
            StatusCode::UNAUTHORIZED => Err(AppError::Authentication("Invalid API credentials".to_string())),
            StatusCode::FORBIDDEN => Err(AppError::Authentication("Forbidden - check API permissions".to_string())),
            StatusCode::TOO_MANY_REQUESTS => Err(AppError::RateLimit { retry_after: 60 }),
            status => Err(AppError::Internal(
                anyhow::anyhow!("Unexpected status code: {}", status)
            )),
        }
    }
}