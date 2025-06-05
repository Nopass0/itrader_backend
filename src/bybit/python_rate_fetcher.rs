use std::sync::Arc;
use anyhow::{Result, Context};
use rust_decimal::Decimal;
use std::str::FromStr;
use tracing::{info, debug, warn, error};
use tokio::sync::{OnceCell, Mutex};
use pyo3::prelude::*;
use serde_json::{json, Value};

static PYTHON_INITIALIZED: OnceCell<()> = OnceCell::const_new();

/// Python-based rate fetcher using Bybit SDK
#[derive(Clone)]
pub struct PythonRateFetcher {
    api_key: String,
    api_secret: String,
    py_client: Arc<Mutex<Option<Py<PyAny>>>>,
}

impl PythonRateFetcher {
    /// Create a new Python rate fetcher
    pub async fn new(api_key: String, api_secret: String) -> Result<Self> {
        // Ensure Python is initialized only once
        PYTHON_INITIALIZED.get_or_init(|| async {
            info!("Initializing Python environment for rate fetcher");
        }).await;
        
        let py_client = Arc::new(Mutex::new(None));
        let fetcher = Self {
            api_key: api_key.clone(),
            api_secret: api_secret.clone(),
            py_client: py_client.clone(),
        };
        
        // Try to initialize Python client
        match fetcher.init_python_client().await {
            Ok(_) => info!("✅ Python rate fetcher initialized successfully"),
            Err(e) => {
                error!("Failed to initialize Python rate fetcher: {}", e);
                warn!("Rate fetcher will operate in fallback mode");
            }
        }
        
        Ok(fetcher)
    }
    
    /// Initialize the Python client
    async fn init_python_client(&self) -> Result<()> {
        let api_key = self.api_key.clone();
        let api_secret = self.api_secret.clone();
        let py_client = self.py_client.clone();
        
        Python::with_gil(|py| -> PyResult<()> {
            // Import required modules
            let sys = py.import("sys")?;
            let path = sys.getattr("path")?;
            path.call_method1("append", ("python_modules",))?;
            
            let module = py.import("bybit_wrapper")?;
            
            // Create client instance
            let client_class = module.getattr("BybitP2PWrapper")?;
            let client = client_class.call1((api_key, api_secret, false))?;
            
            // Store the client
            let mut guard = py_client.blocking_lock();
            *guard = Some(client.into());
            
            Ok(())
        }).context("Failed to initialize Python client")
    }
    
    /// Get current P2P rate
    pub async fn get_current_rate(
        &self,
        fiat: &str,
        trade_type: &str,
        payment_methods: Vec<String>
    ) -> Result<Decimal> {
        debug!("Fetching P2P rate for {} {} with payment methods: {:?}", 
               fiat, trade_type, payment_methods);
        
        let py_client = self.py_client.lock().await;
        
        match py_client.as_ref() {
            Some(client) => {
                let result = Python::with_gil(|py| -> PyResult<Decimal> {
                    // Prepare parameters
                    let params = json!({
                        "token_id": "USDT",
                        "currency_id": fiat,
                        "side": if trade_type == "BUY" { 0 } else { 1 },
                        "payment": payment_methods,
                        "page": 1,
                        "size": 10
                    });
                    
                    // Call fetch_p2p_rates
                    let result = client.call_method1(py, "fetch_p2p_rates", (params.to_string(),))?;
                    let json_str = result.to_string();
                    
                    // Parse response
                    let response: Value = serde_json::from_str(&json_str)
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            format!("Failed to parse JSON: {}", e)
                        ))?;
                    
                    // Extract rates from response
                    if response["success"].as_bool().unwrap_or(false) {
                        if let Some(items) = response["result"]["items"].as_array() {
                            if !items.is_empty() {
                                // Get the best rate (first item)
                                if let Some(price_str) = items[0]["price"].as_str() {
                                    let rate = Decimal::from_str(price_str)
                                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                                            format!("Failed to parse rate: {}", e)
                                        ))?;
                                    
                                    info!("Found P2P rate: {} {}/USDT", rate, fiat);
                                    return Ok(rate);
                                }
                            }
                        }
                    }
                    
                    // No rates found, use fallback
                    warn!("No P2P rates found, using fallback rate");
                    Ok(Decimal::from_str("100.0").unwrap())
                }).context("Failed to fetch rate from Python SDK")?;
                
                Ok(result)
            }
            None => {
                warn!("Python client not available, using fallback rate");
                Ok(self.get_fallback_rate(fiat))
            }
        }
    }
    
    /// Get fallback rate when Python SDK is not available
    fn get_fallback_rate(&self, fiat: &str) -> Decimal {
        match fiat {
            "RUB" => Decimal::from_str("103.50").unwrap(),
            "USD" => Decimal::from_str("1.01").unwrap(),
            "EUR" => Decimal::from_str("0.93").unwrap(),
            _ => Decimal::from_str("100.0").unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Requires Python environment
    async fn test_python_rate_fetcher() {
        let fetcher = PythonRateFetcher::new(
            "test_key".to_string(),
            "test_secret".to_string()
        ).await.unwrap();
        
        let rate = fetcher.get_current_rate(
            "RUB",
            "BUY",
            vec!["75".to_string(), "382".to_string()]
        ).await;
        
        assert!(rate.is_ok());
        let rate = rate.unwrap();
        assert!(rate > Decimal::ZERO);
    }
}