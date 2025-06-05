use crate::utils::error::{Result, AppError};
use chrono::{DateTime, Utc, FixedOffset, TimeZone, Timelike};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, debug};
use anyhow::Error;

#[derive(Debug, Clone)]
pub struct BybitRateFetcher {
    client: Client,
    base_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct P2PTradeResponse {
    pub ret_code: i32,
    pub ret_msg: String,
    pub result: Option<P2PTradeResult>,
    pub ext_code: Option<String>,
    pub ext_info: Option<serde_json::Value>,
    pub time_now: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct P2PTradeResult {
    pub count: i32,
    pub items: Vec<P2PTradeItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct P2PTradeItem {
    pub id: String,
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "userId")]  
    pub user_id: String,
    #[serde(rename = "nickName")]
    pub nickName: String,
    #[serde(rename = "tokenId")]
    pub token_id: String,
    #[serde(rename = "tokenName")]
    pub token_name: String,
    #[serde(rename = "currencyId")]
    pub currency_id: String,
    pub side: i32,
    #[serde(rename = "priceType")]
    pub price_type: i32,
    pub price: String,
    pub premium: String,
    #[serde(rename = "lastQuantity")]
    pub last_quantity: String,
    pub quantity: String,
    #[serde(rename = "frozenQuantity")]
    pub frozen_quantity: String,
    #[serde(rename = "executedQuantity")]
    pub executed_quantity: String,
    #[serde(rename = "minAmount")]
    pub min_amount: String,
    #[serde(rename = "maxAmount")]
    pub max_amount: String,
    pub remark: String,
    pub status: i32,
    #[serde(rename = "createDate")]
    pub create_date: String,
    pub payments: Vec<String>,
    #[serde(rename = "orderNum")]
    pub order_num: i32,
    #[serde(rename = "finishNum")]
    pub finish_num: i32,
    #[serde(rename = "recentOrderNum")]
    pub recent_order_num: i32,
    #[serde(rename = "recentExecuteRate")]
    pub recent_execute_rate: f64,
    #[serde(rename = "isOnline")]
    pub is_online: bool,
    #[serde(rename = "lastLogoutTime")]
    pub last_logout_time: Option<String>,
    pub blocked: Option<String>,
    #[serde(rename = "makerContact")]
    pub maker_contact: Option<bool>,
    #[serde(rename = "symbolInfo")]
    pub symbol_info: Option<serde_json::Value>,
    #[serde(rename = "tradingPreferenceSet")]
    pub trading_preference_set: Option<serde_json::Value>,
    pub version: Option<i32>,
    #[serde(rename = "authStatus")]
    pub auth_status: Option<i32>,
    pub recommend: Option<bool>,
    #[serde(rename = "recommendTag")]
    pub recommend_tag: Option<String>,
    #[serde(rename = "authTag")]
    pub auth_tag: Option<Vec<String>>,
    #[serde(rename = "userType")]
    pub user_type: Option<String>,
    #[serde(rename = "itemType")]
    pub item_type: Option<String>,
    #[serde(rename = "paymentPeriod")]
    pub payment_period: Option<i32>,
    #[serde(rename = "userMaskId")]
    pub user_mask_id: Option<String>,
    #[serde(rename = "verificationOrderSwitch")]
    pub verification_order_switch: Option<bool>,
    #[serde(rename = "verificationOrderLabels")]
    pub verification_order_labels: Option<Vec<String>>,
    #[serde(rename = "verificationOrderAmount")]
    pub verification_order_amount: Option<String>,
    pub ban: Option<bool>,
    pub baned: Option<bool>,
    pub fee: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum RateScenario {
    SmallAmountDay,   // ≤50k RUB, 7:00-1:00 MSK
    SmallAmountNight, // ≤50k RUB, 1:00-7:00 MSK
    LargeAmountDay,   // >50k RUB, 7:00-1:00 MSK
    LargeAmountNight, // >50k RUB, 1:00-7:00 MSK
}

impl RateScenario {
    pub fn determine(amount_rub: f64, moscow_time: DateTime<FixedOffset>) -> Self {
        let hour = moscow_time.hour();
        let is_night = hour >= 1 && hour < 7;
        let is_small_amount = amount_rub <= 50_000.0;
        
        match (is_small_amount, is_night) {
            (true, false) => RateScenario::SmallAmountDay,
            (true, true) => RateScenario::SmallAmountNight,
            (false, false) => RateScenario::LargeAmountDay,
            (false, true) => RateScenario::LargeAmountNight,
        }
    }
    
    pub fn get_page_number(&self) -> u32 {
        match self {
            RateScenario::SmallAmountDay => 4,
            RateScenario::SmallAmountNight => 2,
            RateScenario::LargeAmountDay => 5,
            RateScenario::LargeAmountNight => 3,
        }
    }
}

impl BybitRateFetcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");
            
        Self {
            client,
            base_url: "https://api2.bybit.com".to_string(),
        }
    }
    
    pub async fn fetch_p2p_trades(&self, page: u32) -> Result<P2PTradeResponse> {
        let url = format!("{}/fiat/otc/item/online", self.base_url);
        
        // Prepare request body as JSON - matching exact structure from user
        let payload = serde_json::json!({
            "userId": 431812707,
            "tokenId": "USDT",
            "currencyId": "RUB",
            "payment": ["382", "75"], // СБП (382) and Тинькофф (75) payment methods
            "side": "0", // 0 = Buy (from user perspective, so we're looking at sell orders)
            "size": "10",
            "page": page.to_string(),
            "amount": "",
            "vaMaker": false,
            "bulkMaker": false,
            "canTrade": false,
            "verificationFilter": 0,
            "sortType": "TRADE_PRICE",
            "paymentPeriod": [],
            "itemRegion": 1
        });
        
        debug!("Fetching P2P trades from page {}", page);
        
        let response = self.client
            .post(&url)
            .json(&payload)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 YaBrowser/25.2.0.0 Safari/537.36")
            .header("Accept", "application/json")
            .header("Accept-Language", "ru-RU")
            .header("Content-Type", "application/json;charset=UTF-8")
            .header("Origin", "https://www.bybit.com")
            .header("Referer", "https://www.bybit.com/")
            .send()
            .await
            .map_err(|e| e)?; // Network error will be converted automatically
            
        if !response.status().is_success() {
            return Err(AppError::Internal(anyhow::anyhow!("API returned status: {}", response.status())));
        }
        
        let response_text = response.text().await
            .map_err(|e| AppError::Network(e))?;
            
        debug!("Raw response: {}", response_text);
        
        let trade_response: P2PTradeResponse = serde_json::from_str(&response_text)
            .map_err(|e| AppError::Serialization(format!("Failed to parse P2P response: {}", e)))?;
            
        Ok(trade_response)
    }
    
    pub async fn get_rate_for_scenario(&self, scenario: RateScenario) -> Result<f64> {
        let page = scenario.get_page_number();
        let response = self.fetch_p2p_trades(page).await?;
        
        if let Some(result) = response.result {
            // Since we already filter by payment methods in the request (СБП and Тинькофф),
            // all items should have the required payment methods
            info!("Found {} items on page {}", result.items.len(), page);
            
            // Get penultimate (second to last) price from items
            if result.items.len() >= 2 {
                let penultimate_index = result.items.len() - 2;
                let penultimate_item = &result.items[penultimate_index];
                let price_str = &penultimate_item.price;
                let price = price_str.parse::<f64>()
                    .map_err(|e| AppError::Serialization(format!("Failed to parse price: {}", e)))?;
                    
                info!("Got rate {} for scenario {:?} from page {} (trader: {})", 
                    price, scenario, page, penultimate_item.nickName);
                return Ok(price);
            } else {
                return Err(AppError::Internal(anyhow::anyhow!(
                    "Not enough items on page {} (found {} items)", 
                    page, result.items.len()
                )));
            }
        }
        
        Err(AppError::Internal(anyhow::anyhow!("No result in P2P response")))
    }
    
    pub async fn get_current_rate(&self, amount_rub: f64) -> Result<f64> {
        let moscow_time = self.get_moscow_time();
        let scenario = RateScenario::determine(amount_rub, moscow_time);
        
        info!("Determining rate for amount {} RUB at {} MSK", amount_rub, moscow_time.format("%H:%M:%S"));
        info!("Using scenario: {:?}", scenario);
        
        self.get_rate_for_scenario(scenario).await
    }
    
    pub async fn get_all_rates(&self) -> Result<HashMap<&'static str, f64>> {
        let mut rates = HashMap::new();
        
        // Fetch rates for all scenarios
        let scenarios = vec![
            ("small_amount_day", RateScenario::SmallAmountDay),
            ("small_amount_night", RateScenario::SmallAmountNight),
            ("large_amount_day", RateScenario::LargeAmountDay),
            ("large_amount_night", RateScenario::LargeAmountNight),
        ];
        
        for (name, scenario) in scenarios {
            match self.get_rate_for_scenario(scenario).await {
                Ok(rate) => {
                    rates.insert(name, rate);
                }
                Err(e) => {
                    info!("Failed to get rate for {}: {}", name, e);
                }
            }
        }
        
        Ok(rates)
    }
    
    fn get_moscow_time(&self) -> DateTime<FixedOffset> {
        let utc_now = Utc::now();
        let moscow_offset = FixedOffset::east_opt(3 * 3600).unwrap(); // UTC+3
        moscow_offset.from_utc_datetime(&utc_now.naive_utc())
    }
}