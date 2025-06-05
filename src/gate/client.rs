use anyhow::Context;
use chrono::Utc;
use parking_lot::Mutex;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT},
    Client, StatusCode,
};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn, error};

use super::models::*;
use crate::core::rate_limiter::RateLimiter;
use crate::utils::error::{AppError, Result};

pub struct GateClient {
    client: Client,
    base_url: String,
    cookies: Arc<Mutex<Vec<Cookie>>>,
    rate_limiter: Arc<RateLimiter>,
}

impl GateClient {
    pub fn new(base_url: String, rate_limiter: Arc<RateLimiter>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .cookie_store(false) // Disable reqwest's cookie store, we manage cookies manually
            .gzip(false) // Disable gzip to avoid compression issues
            .build()
            .map_err(|e| AppError::Config(format!("Failed to build HTTP client: {}", e)))?;

        Ok(Self {
            client,
            base_url,
            cookies: Arc::new(Mutex::new(Vec::new())),
            rate_limiter,
        })
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<LoginResponse> {
        self.rate_limiter.check_and_wait("gate").await?;

        let request_body = json!({
            "login": email,
            "password": password
        });

        let login_url = format!("{}/auth/basic/login", self.base_url);
        info!("Attempting login to: {}", login_url);

        let response = self
            .client
            .post(login_url)
            .headers(self.build_headers())
            .json(&request_body)
            .send()
            .await
            .context("Failed to send login request")?;

        let status = response.status();
        debug!("Login response status: {}", status);

        if status == StatusCode::FORBIDDEN {
            warn!("Cloudflare challenge detected, implement bypass logic");
            return Err(AppError::CloudflareBlock);
        }

        // Debug headers
        debug!("Response headers: {:?}", response.headers());

        let cookies_headers = response.headers().get_all("set-cookie");
        if cookies_headers.iter().count() > 0 {
            let mut stored_cookies = self.cookies.lock();
            stored_cookies.clear();

            for cookie_header in cookies_headers {
                if let Ok(cookie_str) = cookie_header.to_str() {
                    debug!("Received cookie: {}", cookie_str);

                    // Parse cookie string to extract name and value
                    if let Some(cookie) = parse_cookie_string(cookie_str) {
                        stored_cookies.push(cookie);
                    }
                }
            }

            info!(
                "Stored {} cookies from login response",
                stored_cookies.len()
            );
        }

        // Try to parse the response, but if we got cookies, consider it successful
        let stored_cookies_count = self.cookies.lock().len();

        // First try to get the response text to see what we're dealing with
        let response_text = response
            .text()
            .await
            .context("Failed to read login response")?;

        debug!("Login response: {}", response_text);

        // If we got cookies, login was successful
        if stored_cookies_count > 0 {
            info!(
                "Successfully logged into Gate.io account: {} (got {} cookies)",
                email, stored_cookies_count
            );

            // Return a dummy response since we got cookies
            return Ok(LoginResponse {
                user_id: "unknown".to_string(),
                session_id: "from_cookies".to_string(),
                expires_at: Utc::now() + chrono::Duration::days(1),
            });
        }

        // Otherwise try to parse the error
        if let Ok(body) = serde_json::from_str::<GateResponse<LoginResponse>>(&response_text) {
            if !body.success {
                return Err(AppError::Authentication(
                    body.error.unwrap_or_else(|| "Unknown error".to_string()),
                ));
            }
            info!("Successfully logged into Gate.io account: {}", email);
            Ok(body.response.unwrap())
        } else {
            // Log the actual response for debugging
            error!("Failed to parse login response. Response text: {}", response_text);
            
            // Check if it's an error response in a different format
            if let Ok(error_response) = serde_json::from_str::<serde_json::Value>(&response_text) {
                if let Some(error_msg) = error_response.get("error").and_then(|e| e.as_str()) {
                    return Err(AppError::Authentication(error_msg.to_string()));
                }
                if let Some(message) = error_response.get("message").and_then(|m| m.as_str()) {
                    return Err(AppError::Authentication(message.to_string()));
                }
            }
            
            Err(AppError::Authentication(
                "Failed to parse login response - unexpected format".to_string(),
            ))
        }
    }

    pub async fn set_cookies(&self, cookies: Vec<Cookie>) -> Result<()> {
        let mut stored_cookies = self.cookies.lock();
        *stored_cookies = cookies;
        info!("Set {} cookies for Gate.io client", stored_cookies.len());
        Ok(())
    }

    pub async fn get_balance(&self, currency: &str) -> Result<BalanceResponse> {
        self.rate_limiter.check_and_wait("gate").await?;

        let response = self
            .client
            .get(format!("{}/auth/me", self.base_url))
            .headers(self.build_headers_with_cookies()?)
            .send()
            .await
            .context("Failed to get balance")?;

        self.check_response_status(&response)?;

        let response_text = response
            .text()
            .await
            .context("Failed to read response text")?;

        debug!("Auth/me response: {}", response_text);

        let body: GateResponse<AuthMeResponse> =
            serde_json::from_str(&response_text).context("Failed to parse auth/me response")?;

        if !body.success {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Failed to get user info: {}",
                body.error.unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        // Find the wallet with the requested currency
        let auth_response = body
            .response
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("No response data")))?;

        let wallet = auth_response
            .user
            .wallets
            .into_iter()
            .find(|w| {
                w.currency.code.to_uppercase() == currency.to_uppercase()
                    || w.currency.code == "643" && currency.to_uppercase() == "RUB"
            }) // Handle RUB as 643
            .ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!(
                    "Wallet for currency {} not found",
                    currency
                ))
            })?;

        // Parse balance from string
        let balance = Decimal::from_str(&wallet.balance).context("Failed to parse balance")?;

        Ok(BalanceResponse {
            currency: currency.to_string(),
            balance,
            available: balance, // Assuming all balance is available for now
            locked: Decimal::ZERO,
        })
    }

    pub async fn set_balance(&self, _email: &str, amount: f64) -> Result<f64> {
        self.rate_limiter.check_and_wait("gate").await?;

        let request_body = json!({
            "amount": amount.to_string()
        });

        let response = self
            .client
            .post(format!("{}/payments/payouts/balance", self.base_url))
            .headers(self.build_headers_with_cookies()?)
            .json(&request_body)
            .send()
            .await
            .context("Failed to set balance")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response text")?;

        debug!(
            "Set balance response (status {}): {}",
            status, response_text
        );

        if !status.is_success() {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Failed to set balance: HTTP {} - {}",
                status,
                response_text
            )));
        }

        let body: GateResponse<serde_json::Value> =
            serde_json::from_str(&response_text).context("Failed to parse set balance response")?;

        if !body.success {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Failed to set balance: {}",
                body.error.unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        info!("Successfully set balance to {}", amount);
        Ok(amount)
    }

    pub async fn get_transactions(&self) -> Result<Vec<GateTransaction>> {
        self.get_transactions_with_filter(TransactionFilter {
            status: None,
            currency: None,
            page: None,
            limit: None,
        })
        .await
    }

    pub async fn get_transactions_with_filter(
        &self,
        filter: TransactionFilter,
    ) -> Result<Vec<GateTransaction>> {
        self.rate_limiter.check_and_wait("gate").await?;

        let mut query_params = vec![];
        if let Some(page) = filter.page {
            query_params.push(("page", page.to_string()));
        }
        if let Some(limit) = filter.limit {
            query_params.push(("per_page", limit.to_string()));
        } else {
            query_params.push(("per_page", "30".to_string()));
        }

        let response = self
            .client
            .get(format!("{}/payments/payouts", self.base_url))
            .headers(self.build_headers_with_cookies()?)
            .query(&query_params)
            .send()
            .await
            .context("Failed to get transactions")?;

        self.check_response_status(&response)?;

        let response_text = response
            .text()
            .await
            .context("Failed to read response text")?;

        debug!("Transactions response: {}", response_text);

        let body: GateResponse<PayoutsResponse> = serde_json::from_str(&response_text)
            .context("Failed to parse transactions response")?;

        if !body.success {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Failed to get transactions: {}",
                body.error.unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        // Convert payouts to GateTransaction format
        let payouts_response = body
            .response
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("No response data")))?;

        let transactions: Vec<GateTransaction> = payouts_response
            .payouts
            .data
            .into_iter()
            .filter_map(|payout| {
                // Skip transactions with empty amounts (they might be in progress)
                if payout.amount.trader.is_empty() || payout.total.trader.is_empty() {
                    debug!("Skipping transaction {} with empty amounts", payout.id);
                    return None;
                }

                // Extract RUB amount from the HashMap
                let rub_amount = payout
                    .amount
                    .trader
                    .get("643")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let rub_total = payout
                    .total
                    .trader
                    .get("643")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                Some(GateTransaction {
                    id: payout.id.to_string(),
                    order_id: payout.id.to_string(),
                    amount: Decimal::from_f64(rub_amount).unwrap_or_default(),
                    currency: "RUB".to_string(),
                    fiat_currency: "RUB".to_string(),
                    fiat_amount: Decimal::from_f64(rub_total).unwrap_or_default(),
                    rate: Decimal::ONE, // No rate info in payouts
                    status: payout.status,
                    buyer_name: "Unknown".to_string(),
                    payment_method: payout.method.label,
                    created_at: Utc::now(), // Parse from string if needed
                    updated_at: Utc::now(), // Parse from string if needed
                })
            })
            .collect();

        Ok(transactions)
    }

    pub async fn accept_transaction(&self, transaction_id: &str) -> Result<()> {
        self.rate_limiter.check_and_wait("gate").await?;

        // According to the API documentation, accepting a transaction is done with POST to /show endpoint
        // The /show endpoint actually shows transaction details and accepts it
        info!("Accepting transaction {} via /show endpoint", transaction_id);
        
        let response = self
            .client
            .post(format!(
                "{}/payments/payouts/{}/show",
                self.base_url, transaction_id
            ))
            .headers(self.build_headers_with_cookies()?)
            .send()
            .await
            .context("Failed to accept transaction")?;

        let status = response.status();
        let response_text = response.text().await.context("Failed to read response text")?;
        
        debug!("Accept transaction response (status {}): {}", status, response_text);

        // Check various response formats
        if !status.is_success() {
            // Sometimes the transaction is already accepted or in a different state
            if status == StatusCode::CONFLICT || status == StatusCode::UNPROCESSABLE_ENTITY || status == StatusCode::BAD_REQUEST {
                // Try to parse error message
                if let Ok(error_resp) = serde_json::from_str::<serde_json::Value>(&response_text) {
                    // Check for error_description in response
                    if let Some(response_obj) = error_resp.get("response") {
                        if let Some(error_desc) = response_obj.get("error_description").and_then(|e| e.as_str()) {
                            // If it's incorrect_status error, the transaction is already in a different state
                            if error_desc.contains("incorrect_status") {
                                warn!("Transaction {} already processed or in incorrect state: {}", transaction_id, error_desc);
                                return Ok(()); // Consider it success - transaction is already accepted
                            }
                        }
                    }
                    
                    if let Some(msg) = error_resp.get("message").and_then(|m| m.as_str()) {
                        warn!("Transaction {} state issue: {}", transaction_id, msg);
                        // If already accepted or processing, consider it success
                        if msg.contains("already") || msg.contains("processing") {
                            return Ok(());
                        }
                    }
                }
            }
            
            return Err(AppError::Internal(anyhow::anyhow!(
                "Failed to accept transaction {}: HTTP {} - {}",
                transaction_id, status, response_text
            )));
        }

        // Try to parse response
        if let Ok(body) = serde_json::from_str::<GateResponse<serde_json::Value>>(&response_text) {
            if !body.success {
                let error_msg = body.error.unwrap_or_else(|| "Unknown error".to_string());
                // Check if it's a state issue
                if error_msg.contains("already") || error_msg.contains("state") {
                    warn!("Transaction {} already in expected state: {}", transaction_id, error_msg);
                    return Ok(());
                }
                return Err(AppError::Internal(anyhow::anyhow!(
                    "Failed to accept transaction: {}",
                    error_msg
                )));
            }
        }

        info!("Successfully accepted transaction: {}", transaction_id);
        Ok(())
    }

    pub async fn get_available_transactions(&self) -> Result<Vec<Payout>> {
        self.rate_limiter.check_and_wait("gate").await?;

        // Get transactions with status 4 or 5
        let url = format!(
            "{}/payments/payouts?filters%5Bstatus%5D%5B%5D=4&filters%5Bstatus%5D%5B%5D=5&page=1",
            self.base_url
        );

        debug!("Getting available transactions from: {}", url);
        
        // Build headers and log cookies
        let headers = self.build_headers_with_cookies()?;
        if let Some(cookie_header) = headers.get("cookie") {
            debug!("Sending cookies: {:?}", cookie_header);
        } else {
            warn!("No cookies in request headers!");
        }
        
        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .context("Failed to get transactions")?;

        let status = response.status();
        debug!("Response status: {}", status);
        debug!("Response headers: {:?}", response.headers());
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            warn!("Failed to get transactions from {}: {} - {}", url, status, error_text);
            return Err(AppError::Internal(anyhow::anyhow!(
                "Failed to get transactions from {}: {} - {}",
                url,
                status,
                error_text
            )));
        }

        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        debug!("Available transactions response: {}", response_text);

        let body: GateResponse<PayoutsResponse> = serde_json::from_str(&response_text)
            .context("Failed to parse transactions response")?;

        if !body.success {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Failed to get transactions: {}",
                body.error.unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        let payouts_response = body
            .response
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("No response data")))?;

        // Return all transactions (already filtered by status 4 and 5 in the URL)
        let available_transactions: Vec<Payout> = payouts_response.payouts.data;

        info!(
            "Found {} transactions with status 4 or 5",
            available_transactions.len()
        );
        Ok(available_transactions)
    }

    pub async fn get_pending_transactions(&self, _email: &str) -> Result<Vec<Payout>> {
        // Get transactions with status 4 (pending)
        self.get_available_transactions().await
    }

    pub async fn get_in_progress_transactions(&self) -> Result<Vec<Payout>> {
        self.rate_limiter.check_and_wait("gate").await?;

        let response = self
            .client
            .get(format!("{}/payments/payouts", self.base_url))
            .headers(self.build_headers_with_cookies()?)
            .send()
            .await
            .context("Failed to get transactions")?;

        self.check_response_status(&response)?;

        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        debug!("In progress transactions response: {}", response_text);

        let body: GateResponse<PayoutsResponse> = serde_json::from_str(&response_text)
            .context("Failed to parse transactions response")?;

        if !body.success {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Failed to get transactions: {}",
                body.error.unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        let payouts_response = body
            .response
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("No response data")))?;

        // Filter transactions with status 5
        let in_progress_transactions: Vec<Payout> = payouts_response
            .payouts
            .data
            .into_iter()
            .filter(|payout| payout.status == 5)
            .collect();

        info!(
            "Found {} in progress transactions (status 5)",
            in_progress_transactions.len()
        );
        Ok(in_progress_transactions)
    }

    pub async fn get_history_transactions(&self, page: Option<u32>) -> Result<Vec<Payout>> {
        self.rate_limiter.check_and_wait("gate").await?;

        let mut query_params = vec![];
        query_params.push(("page", page.unwrap_or(1).to_string()));
        query_params.push(("per_page", "30".to_string()));
        
        let response = self
            .client
            .get(format!("{}/payments/payouts", self.base_url))
            .headers(self.build_headers_with_cookies()?)
            .query(&query_params)
            .send()
            .await
            .context("Failed to get history transactions")?;

        self.check_response_status(&response)?;

        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        debug!("History transactions response: {}", response_text);

        let body: GateResponse<PayoutsResponse> = serde_json::from_str(&response_text)
            .context("Failed to parse history transactions response")?;

        if !body.success {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Failed to get history transactions: {}",
                body.error.unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        let payouts_response = body
            .response
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("No response data")))?;

        // History transactions have status 7 (completed with approved_at) or 9 (history)
        let history_transactions: Vec<Payout> = payouts_response
            .payouts
            .data
            .into_iter()
            .filter(|payout| {
                // Status 7 is completed with receipt
                (payout.status == 7 && payout.approved_at.is_some()) || 
                payout.status == 9
            })
            .collect();

        info!(
            "Found {} history/completed transactions on page {}",
            history_transactions.len(),
            page.unwrap_or(1)
        );
        Ok(history_transactions)
    }

    pub async fn get_transaction_details(&self, transaction_id: &str) -> Result<Payout> {
        self.rate_limiter.check_and_wait("gate").await?;

        // Try the /show endpoint which might have more details
        let response = self
            .client
            .get(format!(
                "{}/payments/payouts/{}/",
                self.base_url, transaction_id
            ))
            .headers(self.build_headers_with_cookies()?)
            .send()
            .await
            .context("Failed to get transaction details")?;

        // If that fails, try without /show
        let (status, response_text) = if response.status().is_success() {
            let text = response
                .text()
                .await
                .context("Failed to read response text")?;
            (StatusCode::OK, text)
        } else {
            // Try alternative endpoint
            let response2 = self
                .client
                .get(format!(
                    "{}/payments/payouts/{}",
                    self.base_url, transaction_id
                ))
                .headers(self.build_headers_with_cookies()?)
                .send()
                .await
                .context("Failed to get transaction details (alternative)")?;

            let status = response2.status();
            let text = response2
                .text()
                .await
                .context("Failed to read response text")?;
            (status, text)
        };

        if !status.is_success() {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Failed to get transaction details: {} - {}",
                status,
                response_text
            )));
        }

        debug!(
            "Transaction {} details response: {}",
            transaction_id, response_text
        );

        // First try to parse as a response with payout field
        #[derive(Deserialize)]
        struct TransactionDetailsResponse {
            payout: Payout,
        }

        // Try parsing with wrapper
        if let Ok(body) =
            serde_json::from_str::<GateResponse<TransactionDetailsResponse>>(&response_text)
        {
            if body.success {
                if let Some(details_response) = body.response {
                    info!(
                        "Successfully retrieved details for transaction {} (wrapped)",
                        transaction_id
                    );
                    return Ok(details_response.payout);
                }
            }
        }

        // Try parsing directly as Payout in response
        if let Ok(body) = serde_json::from_str::<GateResponse<Payout>>(&response_text) {
            if body.success {
                if let Some(payout) = body.response {
                    info!(
                        "Successfully retrieved details for transaction {} (direct)",
                        transaction_id
                    );
                    return Ok(payout);
                }
            }
        }

        Err(AppError::Internal(anyhow::anyhow!(
            "Failed to parse transaction details response: {}",
            response_text
        )))
    }

    pub async fn search_transaction_by_id(&self, transaction_id: &str) -> Result<Option<Payout>> {
        self.rate_limiter.check_and_wait("gate").await?;

        let url = format!(
            "{}/payments/payouts?search%5Bid%5D={}&filters%5Bstatus%5D%5B%5D=4&filters%5Bstatus%5D%5B%5D=5&page=1",
            self.base_url, transaction_id
        );

        let response = self
            .client
            .get(&url)
            .headers(self.build_headers_with_cookies()?)
            .send()
            .await
            .context("Failed to search transaction by ID")?;

        let response_text = response
            .text()
            .await
            .context("Failed to read response text")?;

        debug!("Search transaction response: {}", response_text);

        // Parse the response with payouts structure
        #[derive(Deserialize)]
        struct PayoutsData {
            data: Vec<Payout>,
            meta: serde_json::Value,
        }

        #[derive(Deserialize)]
        struct PayoutsResponse {
            payouts: PayoutsData,
        }

        let body: GateResponse<PayoutsResponse> = serde_json::from_str(&response_text)
            .context("Failed to parse search transaction response")?;

        if body.success {
            if let Some(response) = body.response {
                if !response.payouts.data.is_empty() {
                    let payout = response.payouts.data.into_iter().next().unwrap();
                    info!("Found transaction {} via search", transaction_id);
                    return Ok(Some(payout));
                }
            }
        }

        info!("Transaction {} not found via search", transaction_id);
        Ok(None)
    }

    pub async fn complete_transaction(&self, transaction_id: &str) -> Result<()> {
        self.rate_limiter.check_and_wait("gate").await?;

        let request_body = json!({
            "transaction_id": transaction_id,
            "action": "complete"
        });

        // Complete transaction with receipt (approve endpoint)
        let response = self
            .client
            .post(format!(
                "{}/payments/payouts/{}/approve",
                self.base_url, transaction_id
            ))
            .headers(self.build_headers_with_cookies()?)
            .json(&request_body)
            .send()
            .await
            .context("Failed to complete transaction")?;

        self.check_response_status(&response)?;

        let body: GateResponse<serde_json::Value> = response
            .json()
            .await
            .context("Failed to parse complete response")?;

        if !body.success {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Failed to complete transaction: {}",
                body.error.unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        info!("Successfully completed transaction: {}", transaction_id);
        Ok(())
    }

    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            ),
        );
        headers.insert(
            "referer",
            HeaderValue::from_static("https://panel.gate.cx/"),
        );
        headers.insert("origin", HeaderValue::from_static("https://panel.gate.cx"));
        headers.insert(
            "accept",
            HeaderValue::from_static("application/json, text/plain, */*"),
        );
        headers.insert(
            "accept-language",
            HeaderValue::from_static("en-US,en;q=0.9"),
        );
        // Don't accept compressed responses
        headers.insert("accept-encoding", HeaderValue::from_static("identity"));
        headers.insert("dnt", HeaderValue::from_static("1"));
        headers
    }

    fn build_headers_with_cookies(&self) -> Result<HeaderMap> {
        let mut headers = self.build_headers();

        let cookies = self.cookies.lock();
        let cookie_string = cookies
            .iter()
            .map(|c| format!("{}={}", c.name, c.value))
            .collect::<Vec<_>>()
            .join("; ");

        if !cookie_string.is_empty() {
            headers.insert(
                "cookie",
                HeaderValue::from_str(&cookie_string)
                    .map_err(|_| AppError::Config("Invalid cookie value".to_string()))?,
            );
        }

        Ok(headers)
    }

    pub async fn get_cookies(&self) -> Vec<Cookie> {
        let cookies = self.cookies.lock();
        cookies.clone()
    }

    pub async fn load_cookies(&self, file_path: &str) -> Result<()> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| AppError::FileSystem(format!("Failed to read cookies file: {}", e)))?;

        let cookies: Vec<Cookie> = serde_json::from_str(&content)
            .map_err(|e| AppError::Serialization(format!("Failed to parse cookies: {}", e)))?;

        self.set_cookies(cookies).await?;
        Ok(())
    }

    fn check_response_status(&self, response: &reqwest::Response) -> Result<()> {
        match response.status() {
            StatusCode::OK => Ok(()),
            StatusCode::UNAUTHORIZED => Err(AppError::SessionExpired),
            StatusCode::FORBIDDEN => Err(AppError::CloudflareBlock),
            StatusCode::TOO_MANY_REQUESTS => Err(AppError::RateLimit { retry_after: 60 }),
            status => Err(AppError::Internal(anyhow::anyhow!(
                "Unexpected status code: {}",
                status
            ))),
        }
    }

    pub async fn test_requests_endpoint(&self) -> Result<String> {
        let url = format!("{}/requests?page=1", self.base_url.replace("/api/v1", ""));
        
        debug!("Testing requests endpoint: {}", url);
        
        let response = self
            .client
            .get(&url)
            .headers(self.build_headers_with_cookies()?)
            .send()
            .await?;
            
        let status = response.status();
        let text = response.text().await?;
        
        if !status.is_success() {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Request failed: {} - {}",
                status,
                text
            )));
        }
        
        Ok(text)
    }

    pub async fn get_transaction_for_approval(&self, transaction_id: &str) -> Result<Option<Payout>> {
        // Try to get from available transactions first
        let available = self.get_available_transactions().await?;
        
        for tx in available {
            if tx.id.to_string() == transaction_id {
                return Ok(Some(tx));
            }
        }
        
        // Not found in available transactions
        Ok(None)
    }

    pub async fn approve_transaction_with_receipt(&self, transaction_id: &str, pdf_path: &str) -> Result<Payout> {
        use reqwest::multipart;
        use std::fs;
        use std::path::Path;
        
        info!("Approving transaction {} with receipt: {}", transaction_id, pdf_path);

        // Read the PDF file
        let pdf_data = fs::read(pdf_path)
            .context(format!("Failed to read PDF file: {}", pdf_path))?;
        
        let file_name = Path::new(pdf_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("receipt.pdf");

        // Create multipart form
        let part = multipart::Part::bytes(pdf_data)
            .file_name(file_name.to_string())
            .mime_str("application/pdf")?;
            
        let form = multipart::Form::new()
            .part("attachments[]", part);

        // Build headers for multipart request
        let mut headers = self.build_headers_with_cookies()?;
        // Remove content-type as it will be set automatically for multipart
        headers.remove(CONTENT_TYPE);

        // Send request
        let response = self
            .client
            .post(format!(
                "{}/payments/payouts/{}/approve",
                self.base_url, transaction_id
            ))
            .headers(headers)
            .multipart(form)
            .send()
            .await
            .context("Failed to approve transaction")?;

        self.check_response_status(&response)?;

        let body: GateResponse<PayoutResponse> = response
            .json()
            .await
            .context("Failed to parse approve response")?;

        if !body.success {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Failed to approve transaction: {}",
                body.error.unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        info!("Successfully approved transaction {} with receipt", transaction_id);
        
        // body.response is Option<PayoutResponse>
        match body.response {
            Some(payout_response) => Ok(payout_response.payout),
            None => Err(AppError::Internal(anyhow::anyhow!("No payout data in response")))
        }
    }
    
    pub async fn approve_transaction(&self, transaction_id: &str) -> Result<Payout> {
        info!("Approving transaction {} without receipt", transaction_id);
        
        let response = self
            .client
            .post(format!(
                "{}/payments/payouts/{}/approve",
                self.base_url, transaction_id
            ))
            .headers(self.build_headers_with_cookies()?)
            .send()
            .await
            .context("Failed to approve transaction")?;

        self.check_response_status(&response)?;

        let body: GateResponse<PayoutResponse> = response
            .json()
            .await
            .context("Failed to parse approve response")?;

        if !body.success {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Failed to approve transaction: {}",
                body.error.unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        match body.response {
            Some(payout_response) => Ok(payout_response.payout),
            None => Err(AppError::Internal(anyhow::anyhow!("No payout data in response")))
        }
    }
    
    pub async fn cancel_order(&self, transaction_id: &str) -> Result<Payout> {
        info!("Cancelling order {}", transaction_id);
        
        let response = self
            .client
            .post(format!(
                "{}/payments/payouts/{}/cancel",
                self.base_url, transaction_id
            ))
            .headers(self.build_headers_with_cookies()?)
            .send()
            .await
            .context("Failed to cancel order")?;

        self.check_response_status(&response)?;

        let body: GateResponse<PayoutResponse> = response
            .json()
            .await
            .context("Failed to parse cancel response")?;

        if !body.success {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Failed to cancel order: {}",
                body.error.unwrap_or_else(|| "Unknown error".to_string())
            )));
        }

        match body.response {
            Some(payout_response) => Ok(payout_response.payout),
            None => Err(AppError::Internal(anyhow::anyhow!("No payout data in response")))
        }
    }
    
    pub async fn update_balance(&self, amount: f64) -> Result<serde_json::Value> {
        info!("Updating balance to {}", amount);
        
        let body = json!({
            "amount": amount.to_string(),
            "currency": "RUB"
        });
        
        let response = self
            .client
            .post(format!("{}/account/balance/update", self.base_url))
            .headers(self.build_headers_with_cookies()?)
            .json(&body)
            .send()
            .await
            .context("Failed to update balance")?;

        self.check_response_status(&response)?;

        let body: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse balance update response")?;

        Ok(body)
    }
    
    pub async fn is_authenticated(&self) -> bool {
        // Check if we have valid cookies and can make authenticated requests
        if self.cookies.lock().is_empty() {
            return false;
        }
        
        // Try to get balance as authentication check
        match self.get_balance("RUB").await {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

// Helper function to parse cookie string
fn parse_cookie_string(cookie_str: &str) -> Option<Cookie> {
    // Split by semicolon to get cookie parts
    let parts: Vec<&str> = cookie_str.split(';').collect();
    if parts.is_empty() {
        return None;
    }

    // First part is name=value
    let name_value: Vec<&str> = parts[0].trim().splitn(2, '=').collect();
    if name_value.len() != 2 {
        return None;
    }

    let name = name_value[0].to_string();
    let value = name_value[1].to_string();

    let mut cookie = Cookie {
        name,
        value,
        domain: ".panel.gate.cx".to_string(),
        path: "/".to_string(),
        secure: true,
        http_only: true,
        same_site: None,
        session: false,
        host_only: false,
        store_id: None,
        expiration_date: None,
    };

    // Parse other attributes
    for part in parts.iter().skip(1) {
        let part = part.trim();
        if let Some((key, val)) = part.split_once('=') {
            match key.to_lowercase().as_str() {
                "domain" => cookie.domain = val.to_string(),
                "path" => cookie.path = val.to_string(),
                "max-age" => {
                    if let Ok(seconds) = val.parse::<i64>() {
                        let now = chrono::Utc::now().timestamp();
                        cookie.expiration_date = Some((now + seconds) as f64);
                    }
                }
                "expires" => {
                    // Parse expires date - this is simplified
                    // In production you'd want proper date parsing
                }
                _ => {}
            }
        } else {
            match part.to_lowercase().as_str() {
                "secure" => cookie.secure = true,
                "httponly" => cookie.http_only = true,
                _ => {}
            }
        }
    }

    // If no expiration date set, assume it expires in 30 days
    if cookie.expiration_date.is_none() {
        let thirty_days = chrono::Utc::now().timestamp() + (30 * 24 * 60 * 60);
        cookie.expiration_date = Some(thirty_days as f64);
    }

    Some(cookie)
}
