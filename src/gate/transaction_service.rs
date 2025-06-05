use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use crate::gate::{GateClient, Payout};
use crate::utils::error::AppError;

pub struct TransactionService {
    client: GateClient,
    cache: Arc<RwLock<HashMap<String, CachedTransaction>>>,
}

struct CachedTransaction {
    transaction: Payout,
    cached_at: DateTime<Utc>,
}

impl TransactionService {
    pub fn new(client: GateClient) -> Self {
        Self {
            client,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_transaction(&self, transaction_id: &str) -> Result<Option<Payout>, AppError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(transaction_id) {
                // Cache valid for 5 minutes
                if Utc::now().signed_duration_since(cached.cached_at).num_minutes() < 5 {
                    return Ok(Some(cached.transaction.clone()));
                }
            }
        }

        // Fetch from API
        let transaction = self.client.search_transaction_by_id(transaction_id).await?;
        
        // Update cache if found
        if let Some(ref tx) = transaction {
            let mut cache = self.cache.write().await;
            cache.insert(
                transaction_id.to_string(),
                CachedTransaction {
                    transaction: tx.clone(),
                    cached_at: Utc::now(),
                }
            );
        }

        Ok(transaction)
    }

    pub async fn get_multiple_transactions(&self, transaction_ids: &[&str]) -> Result<HashMap<String, Option<Payout>>, AppError> {
        let mut results = HashMap::new();
        
        for id in transaction_ids {
            let transaction = self.get_transaction(id).await?;
            results.insert(id.to_string(), transaction);
        }
        
        Ok(results)
    }

    pub async fn get_transaction_status(&self, transaction_id: &str) -> Result<Option<i32>, AppError> {
        let transaction = self.get_transaction(transaction_id).await?;
        Ok(transaction.map(|t| t.status))
    }

    pub async fn is_transaction_completed(&self, transaction_id: &str) -> Result<bool, AppError> {
        let status = self.get_transaction_status(transaction_id).await?;
        Ok(status.map(|s| s == 5).unwrap_or(false))
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
    
    #[cfg(test)]
    pub async fn cache_size(&self) -> usize {
        self.cache.read().await.len()
    }

    pub async fn remove_from_cache(&self, transaction_id: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(transaction_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::RateLimitsConfig;
    use crate::core::rate_limiter::RateLimiter;

    #[tokio::test]
    async fn test_cache_functionality() {
        let rate_limits_config = RateLimitsConfig {
            bybit_requests_per_minute: 600,
            gate_requests_per_minute: 240,
            default_burst_size: 10,
        };
        let rate_limiter = Arc::new(RateLimiter::new(&rate_limits_config));
        let client = GateClient::new("https://test.com".to_string(), rate_limiter).unwrap();
        let service = TransactionService::new(client);
        
        // Test cache operations
        service.clear_cache().await;
        
        // Cache should be empty
        let cache = service.cache.read().await;
        assert_eq!(cache.len(), 0);
    }
}