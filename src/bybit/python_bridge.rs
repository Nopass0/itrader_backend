use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::{Result, Context};
use pyo3::prelude::*;
use pyo3::types::{PyModule, PyDict};
use serde_json::Value;

use super::models::{AccountInfo, Advertisement, P2POrder, ChatMessage, PaymentMethod};

/// Python Bybit client wrapper using PyO3
pub struct PyBybitClient {
    client: Arc<Mutex<Py<PyAny>>>,
}

impl PyBybitClient {
    /// Create a new Python Bybit client
    pub async fn new(api_key: String, api_secret: String) -> Result<Self> {
        let client = Python::with_gil(|py| -> PyResult<Py<PyAny>> {
            // Import the bybit_wrapper module
            let sys = py.import("sys")?;
            let path = sys.getattr("path")?;
            path.call_method1("append", ("python_modules",))?;
            
            let module = py.import("bybit_wrapper")?;
            
            // Create client instance
            let client_class = module.getattr("BybitP2PWrapper")?;
            let client = client_class.call1((api_key, api_secret, false))?;
            
            Ok(client.into())
        }).context("Failed to create Python Bybit client")?;
        
        Ok(Self {
            client: Arc::new(Mutex::new(client)),
        })
    }
    
    /// Get account information
    pub async fn get_account_info(&self) -> Result<AccountInfo> {
        let client = self.client.lock().await;
        
        let result = Python::with_gil(|py| -> PyResult<AccountInfo> {
            let result = client.call_method0(py, "get_account_info")?;
            let json_str = result.to_string();
            
            // Parse the Python dict as JSON
            let value: Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Failed to parse JSON: {}", e)
                ))?;
            
            // Convert to AccountInfo
            let account = AccountInfo {
                id: value["id"].as_str().unwrap_or("").to_string(),
                email: value["email"].as_str().map(|s| s.to_string()),
                nickname: value["nickname"].as_str().unwrap_or("").to_string(),
                status: value["status"].as_str().unwrap_or("unknown").to_string(),
                active_ads: value["activeAds"].as_u64().unwrap_or(0) as u32,
            };
            
            Ok(account)
        }).context("Failed to get account info from Python")?;
        
        Ok(result)
    }
    
    /// Create a new P2P advertisement
    pub async fn create_ad(&self, ad_params: Value) -> Result<Advertisement> {
        let client = self.client.lock().await;
        let params_str = ad_params.to_string();
        
        let result = Python::with_gil(|py| -> PyResult<Advertisement> {
            // Call create_advertisement with JSON params
            let result = client.call_method1(py, "create_advertisement", (params_str,))?;
            let json_str = result.to_string();
            
            // Parse result
            let value: Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Failed to parse JSON: {}", e)
                ))?;
            
            // Convert to Advertisement
            let ad = Advertisement {
                id: value["id"].as_str().unwrap_or("").to_string(),
                asset: value["asset"].as_str().unwrap_or("").to_string(),
                fiat: value["fiat"].as_str().unwrap_or("").to_string(),
                price: value["price"].as_str().unwrap_or("0").to_string(),
                amount: value["amount"].as_str().unwrap_or("0").to_string(),
                min_amount: value["min_amount"].as_str().unwrap_or("0").to_string(),
                max_amount: value["max_amount"].as_str().unwrap_or("0").to_string(),
                status: value["status"].as_str().unwrap_or("0").to_string(),
                payment_methods: value["payment_methods"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|pm| {
                                Some(PaymentMethod {
                                    id: pm["id"].as_str()?.to_string(),
                                    name: pm["name"].as_str()?.to_string(),
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                remarks: value["remarks"].as_str().map(|s| s.to_string()),
                created_at: value["created_at"].as_str().unwrap_or("").to_string(),
            };
            
            Ok(ad)
        }).context("Failed to create advertisement")?;
        
        Ok(result)
    }
    
    /// Get active advertisements
    pub async fn get_active_ads(&self) -> Result<Vec<Advertisement>> {
        let client = self.client.lock().await;
        
        let result = Python::with_gil(|py| -> PyResult<Vec<Advertisement>> {
            let result = client.call_method0(py, "get_my_advertisements")?;
            let json_str = result.to_string();
            
            // Parse result array
            let values: Vec<Value> = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Failed to parse JSON: {}", e)
                ))?;
            
            // Convert to Vec<Advertisement>
            let ads = values.into_iter()
                .map(|value| Advertisement {
                    id: value["adId"].as_str().unwrap_or("").to_string(),
                    asset: value["tokenId"].as_str().unwrap_or("").to_string(),
                    fiat: value["currencyId"].as_str().unwrap_or("").to_string(),
                    price: value["price"].as_str().unwrap_or("0").to_string(),
                    amount: value["quantity"].as_str().unwrap_or("0").to_string(),
                    min_amount: value["minAmount"].as_str().unwrap_or("0").to_string(),
                    max_amount: value["maxAmount"].as_str().unwrap_or("0").to_string(),
                    status: value["status"].as_str().unwrap_or("0").to_string(),
                    payment_methods: vec![],  // Not included in list response
                    remarks: value["remarks"].as_str().map(|s| s.to_string()),
                    created_at: value["createdAt"].as_str().unwrap_or("").to_string(),
                })
                .collect();
            
            Ok(ads)
        }).context("Failed to get active advertisements")?;
        
        Ok(result)
    }
    
    /// Delete an advertisement
    pub async fn delete_ad(&self, ad_id: &str) -> Result<()> {
        let client = self.client.lock().await;
        
        Python::with_gil(|py| -> PyResult<()> {
            // Note: The delete_ad method is not implemented in the Python wrapper
            // This would need to be added to bybit_wrapper.py
            Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
                "delete_ad not implemented in Python wrapper"
            ))
        }).context("Failed to delete advertisement")?;
        
        Ok(())
    }
    
    /// Get a specific order
    pub async fn get_order(&self, order_id: &str) -> Result<P2POrder> {
        let client = self.client.lock().await;
        
        let result = Python::with_gil(|py| -> PyResult<P2POrder> {
            let result = client.call_method1(py, "get_order_details", (order_id,))?;
            let json_str = result.to_string();
            
            // Parse result
            let value: Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Failed to parse JSON: {}", e)
                ))?;
            
            // Convert to P2POrder
            let order = P2POrder {
                id: value["id"].as_str().unwrap_or("").to_string(),
                ad_id: value["adId"].as_str().unwrap_or("").to_string(),
                asset: value["tokenId"].as_str().unwrap_or("").to_string(),
                fiat: value["currencyId"].as_str().unwrap_or("").to_string(),
                price: value["price"].as_str().unwrap_or("0").to_string(),
                amount: value["amount"].as_str().unwrap_or("0").to_string(),
                status: value["status"].as_str().unwrap_or("10").to_string(),
                buyer_id: value["buyerUserId"].as_str().unwrap_or("").to_string(),
                seller_id: value["sellerUserId"].as_str().unwrap_or("").to_string(),
                payment_info: value["paymentInfo"].clone(),
                created_at: value["createdAt"].as_str().unwrap_or("").to_string(),
                paid_at: None,
                released_at: None,
            };
            
            Ok(order)
        }).context("Failed to get order details")?;
        
        Ok(result)
    }
    
    /// Get active orders
    pub async fn get_active_orders(&self) -> Result<Vec<P2POrder>> {
        let client = self.client.lock().await;
        
        let result = Python::with_gil(|py| -> PyResult<Vec<P2POrder>> {
            let result = client.call_method0(py, "get_active_orders")?;
            let json_str = result.to_string();
            
            // Parse result array
            let values: Vec<Value> = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Failed to parse JSON: {}", e)
                ))?;
            
            // Convert to Vec<P2POrder>
            let orders = values.into_iter()
                .map(|value| P2POrder {
                    id: value["orderId"].as_str().unwrap_or("").to_string(),
                    ad_id: value["adId"].as_str().unwrap_or("").to_string(),
                    asset: value["tokenId"].as_str().unwrap_or("").to_string(),
                    fiat: value["currencyId"].as_str().unwrap_or("").to_string(),
                    price: value["price"].as_str().unwrap_or("0").to_string(),
                    amount: value["amount"].as_str().unwrap_or("0").to_string(),
                    status: value["orderStatus"].as_str().unwrap_or("10").to_string(),
                    buyer_id: value["buyerUserId"].as_str().unwrap_or("").to_string(),
                    seller_id: value["sellerUserId"].as_str().unwrap_or("").to_string(),
                    payment_info: value["paymentInfo"].clone(),
                    created_at: value["createdAt"].as_str().unwrap_or("").to_string(),
                    paid_at: value["paidAt"].as_str().map(|s| s.to_string()),
                    released_at: value["releasedAt"].as_str().map(|s| s.to_string()),
                })
                .collect();
            
            Ok(orders)
        }).context("Failed to get active orders")?;
        
        Ok(result)
    }
    
    /// Send a message in order chat
    pub async fn send_message(&self, order_id: &str, message: &str) -> Result<()> {
        let client = self.client.lock().await;
        
        Python::with_gil(|py| -> PyResult<()> {
            let result = client.call_method1(py, "send_chat_message", (order_id, message))?;
            let success = result.extract::<bool>(py)?;
            
            if success {
                Ok(())
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    "Failed to send chat message"
                ))
            }
        }).context("Failed to send message")?;
        
        Ok(())
    }
    
    /// Release an order
    pub async fn release_order(&self, order_id: &str) -> Result<()> {
        let client = self.client.lock().await;
        
        Python::with_gil(|py| -> PyResult<()> {
            let result = client.call_method1(py, "release_order", (order_id,))?;
            let success = result.extract::<bool>(py)?;
            
            if success {
                Ok(())
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    "Failed to release order"
                ))
            }
        }).context("Failed to release order")?;
        
        Ok(())
    }
    
    /// Get chat messages for an order
    pub async fn get_chat_messages(&self, order_id: &str) -> Result<Vec<ChatMessage>> {
        let client = self.client.lock().await;
        
        let result = Python::with_gil(|py| -> PyResult<Vec<ChatMessage>> {
            let result = client.call_method1(py, "get_chat_messages", (order_id,))?;
            let json_str = result.to_string();
            
            // Parse result array
            let values: Vec<Value> = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Failed to parse JSON: {}", e)
                ))?;
            
            // Convert to Vec<ChatMessage>
            let messages = values.into_iter()
                .map(|value| ChatMessage {
                    id: value["id"].as_str().unwrap_or("").to_string(),
                    order_id: value["orderId"].as_str().unwrap_or("").to_string(),
                    user_id: value["userId"].as_str().unwrap_or("").to_string(),
                    content: value["content"].as_str().unwrap_or("").to_string(),
                    timestamp: value["timestamp"].as_str().unwrap_or("").to_string(),
                    message_type: value["type"].as_str().unwrap_or("text").to_string(),
                })
                .collect();
            
            Ok(messages)
        }).context("Failed to get chat messages")?;
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Requires Python environment
    async fn test_python_client_creation() {
        let client = PyBybitClient::new(
            "test_key".to_string(),
            "test_secret".to_string()
        ).await;
        
        assert!(client.is_ok());
    }
}