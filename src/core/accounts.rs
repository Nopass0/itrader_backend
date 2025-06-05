use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tokio::sync::RwLock;
use std::sync::Arc;

use crate::utils::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateAccount {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub cookies: Option<serde_json::Value>,
    pub last_auth: Option<DateTime<Utc>>,
    pub balance: f64,
    pub status: AccountStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BybitAccount {
    pub id: i32,
    pub account_name: String,
    pub api_key: String,
    pub api_secret: String,
    pub active_ads: i32,
    pub status: BybitAccountStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    Active,
    Inactive,
    Suspended,
    AuthRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BybitAccountStatus {
    Available,
    Busy,
    Suspended,
    MaxAdsReached,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountsData {
    pub gate_accounts: Vec<GateAccount>,
    pub bybit_accounts: Vec<BybitAccount>,
    pub last_updated: DateTime<Utc>,
}

pub struct AccountManager {
    data: Arc<RwLock<AccountsData>>,
    file_path: String,
}

impl AccountManager {
    pub async fn new(file_path: impl AsRef<Path>) -> Result<Self> {
        let file_path = file_path.as_ref().to_string_lossy().to_string();
        
        // Load existing data or create new
        let data = if Path::new(&file_path).exists() {
            let content = fs::read_to_string(&file_path).await?;
            serde_json::from_str::<AccountsData>(&content)?
        } else {
            AccountsData {
                gate_accounts: Vec::new(),
                bybit_accounts: Vec::new(),
                last_updated: Utc::now(),
            }
        };

        let manager = Self {
            data: Arc::new(RwLock::new(data)),
            file_path,
        };

        // Save initial state
        manager.save_to_file().await?;
        
        Ok(manager)
    }

    // Gate account management
    pub async fn add_gate_account(&self, email: String, password: String) -> Result<i32> {
        let mut data = self.data.write().await;
        
        // Check if account already exists
        if data.gate_accounts.iter().any(|a| a.email == email) {
            return Err(AppError::BadRequest("Gate account already exists".to_string()));
        }

        let id = data.gate_accounts.len() as i32 + 1;
        let account = GateAccount {
            id,
            email,
            password,
            cookies: None,
            last_auth: None,
            balance: 0.0,
            status: AccountStatus::Inactive,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        data.gate_accounts.push(account);
        data.last_updated = Utc::now();
        drop(data);

        self.save_to_file().await?;
        Ok(id)
    }

    pub async fn get_gate_account(&self, id: i32) -> Result<Option<GateAccount>> {
        let data = self.data.read().await;
        Ok(data.gate_accounts.iter().find(|a| a.id == id).cloned())
    }

    pub async fn get_gate_account_by_email(&self, email: &str) -> Result<Option<GateAccount>> {
        let data = self.data.read().await;
        Ok(data.gate_accounts.iter().find(|a| a.email == email).cloned())
    }

    pub async fn update_gate_account_cookies(&self, id: i32, cookies: serde_json::Value) -> Result<()> {
        let mut data = self.data.write().await;
        
        if let Some(account) = data.gate_accounts.iter_mut().find(|a| a.id == id) {
            account.cookies = Some(cookies);
            account.last_auth = Some(Utc::now());
            account.status = AccountStatus::Active;
            account.updated_at = Utc::now();
            data.last_updated = Utc::now();
            drop(data);
            
            self.save_to_file().await?;
            Ok(())
        } else {
            Err(AppError::NotFound("Gate account not found".to_string()))
        }
    }

    pub async fn update_gate_balance(&self, id: i32, balance: f64) -> Result<()> {
        let mut data = self.data.write().await;
        
        if let Some(account) = data.gate_accounts.iter_mut().find(|a| a.id == id) {
            account.balance = balance;
            account.updated_at = Utc::now();
            data.last_updated = Utc::now();
            drop(data);
            
            self.save_to_file().await?;
            Ok(())
        } else {
            Err(AppError::NotFound("Gate account not found".to_string()))
        }
    }

    pub async fn get_active_gate_accounts(&self) -> Result<Vec<GateAccount>> {
        let data = self.data.read().await;
        Ok(data.gate_accounts
            .iter()
            .filter(|a| a.status == AccountStatus::Active)
            .cloned()
            .collect())
    }

    // Bybit account management
    pub async fn add_bybit_account(&self, account_name: String, api_key: String, api_secret: String) -> Result<i32> {
        let mut data = self.data.write().await;
        
        // Check if account already exists
        if data.bybit_accounts.iter().any(|a| a.account_name == account_name) {
            return Err(AppError::BadRequest("Bybit account already exists".to_string()));
        }

        let id = data.bybit_accounts.len() as i32 + 1;
        let account = BybitAccount {
            id,
            account_name,
            api_key,
            api_secret,
            active_ads: 0,
            status: BybitAccountStatus::Available,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        data.bybit_accounts.push(account);
        data.last_updated = Utc::now();
        drop(data);

        self.save_to_file().await?;
        Ok(id)
    }

    pub async fn get_bybit_account(&self, id: i32) -> Result<Option<BybitAccount>> {
        let data = self.data.read().await;
        Ok(data.bybit_accounts.iter().find(|a| a.id == id).cloned())
    }

    pub async fn get_available_bybit_account(&self) -> Result<Option<BybitAccount>> {
        let data = self.data.read().await;
        Ok(data.bybit_accounts
            .iter()
            .find(|a| a.status == BybitAccountStatus::Available && a.active_ads < 4)
            .cloned())
    }

    pub async fn update_bybit_active_ads(&self, id: i32, active_ads: i32) -> Result<()> {
        let mut data = self.data.write().await;
        
        if let Some(account) = data.bybit_accounts.iter_mut().find(|a| a.id == id) {
            account.active_ads = active_ads;
            account.status = if active_ads >= 4 {
                BybitAccountStatus::MaxAdsReached
            } else if active_ads > 0 {
                BybitAccountStatus::Busy
            } else {
                BybitAccountStatus::Available
            };
            account.updated_at = Utc::now();
            data.last_updated = Utc::now();
            drop(data);
            
            self.save_to_file().await?;
            Ok(())
        } else {
            Err(AppError::NotFound("Bybit account not found".to_string()))
        }
    }

    pub async fn get_all_bybit_accounts(&self) -> Result<Vec<BybitAccount>> {
        let data = self.data.read().await;
        Ok(data.bybit_accounts.clone())
    }

    // Utility methods
    pub async fn get_stats(&self) -> Result<AccountStats> {
        let data = self.data.read().await;
        
        let gate_active = data.gate_accounts.iter().filter(|a| a.status == AccountStatus::Active).count();
        let gate_total = data.gate_accounts.len();
        
        let bybit_available = data.bybit_accounts.iter().filter(|a| a.status == BybitAccountStatus::Available).count();
        let bybit_total = data.bybit_accounts.len();
        let total_active_ads: i32 = data.bybit_accounts.iter().map(|a| a.active_ads).sum();
        
        Ok(AccountStats {
            gate_active,
            gate_total,
            bybit_available,
            bybit_total,
            total_active_ads,
        })
    }

    async fn save_to_file(&self) -> Result<()> {
        let data = self.data.read().await;
        let json = serde_json::to_string_pretty(&*data)?;
        fs::write(&self.file_path, json).await?;
        Ok(())
    }

    pub async fn reload_from_file(&self) -> Result<()> {
        let content = fs::read_to_string(&self.file_path).await?;
        let new_data: AccountsData = serde_json::from_str(&content)?;
        
        let mut data = self.data.write().await;
        *data = new_data;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStats {
    pub gate_active: usize,
    pub gate_total: usize,
    pub bybit_available: usize,
    pub bybit_total: usize,
    pub total_active_ads: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_account_manager() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("accounts.json");
        
        let manager = AccountManager::new(&file_path).await?;
        
        // Test Gate account
        let gate_id = manager.add_gate_account("test@example.com".to_string(), "password123".to_string()).await?;
        assert_eq!(gate_id, 1);
        
        let account = manager.get_gate_account(gate_id).await?.unwrap();
        assert_eq!(account.email, "test@example.com");
        assert_eq!(account.status, AccountStatus::Inactive);
        
        // Update cookies
        let cookies = serde_json::json!([{"name": "session", "value": "123"}]);
        manager.update_gate_account_cookies(gate_id, cookies).await?;
        
        let account = manager.get_gate_account(gate_id).await?.unwrap();
        assert_eq!(account.status, AccountStatus::Active);
        assert!(account.cookies.is_some());
        
        // Test Bybit account
        let bybit_id = manager.add_bybit_account(
            "bybit1".to_string(),
            "api_key".to_string(),
            "api_secret".to_string()
        ).await?;
        assert_eq!(bybit_id, 1);
        
        let bybit_account = manager.get_bybit_account(bybit_id).await?.unwrap();
        assert_eq!(bybit_account.account_name, "bybit1");
        assert_eq!(bybit_account.status, BybitAccountStatus::Available);
        
        // Update active ads
        manager.update_bybit_active_ads(bybit_id, 4).await?;
        let bybit_account = manager.get_bybit_account(bybit_id).await?.unwrap();
        assert_eq!(bybit_account.status, BybitAccountStatus::MaxAdsReached);
        
        // Test stats
        let stats = manager.get_stats().await?;
        assert_eq!(stats.gate_active, 1);
        assert_eq!(stats.gate_total, 1);
        assert_eq!(stats.bybit_available, 0);
        assert_eq!(stats.bybit_total, 1);
        assert_eq!(stats.total_active_ads, 4);
        
        Ok(())
    }
}