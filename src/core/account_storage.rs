use std::path::{Path, PathBuf};
use tokio::fs;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::utils::error::{AppError, Result};
// use crate::utils::crypto::{encrypt_string, decrypt_string};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateAccountData {
    pub id: String,
    pub login: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    pub cookies: Option<serde_json::Value>,
    #[serde(alias = "last_login")]
    pub last_auth: Option<DateTime<Utc>>,
    pub balance: f64,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BybitAccountData {
    pub id: String,
    pub api_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_secret: Option<String>,
    #[serde(default)]
    pub active_ads: i32,
    #[serde(alias = "last_login")]
    pub last_used: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testnet: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

pub struct AccountStorage {
    base_path: PathBuf,
}

impl AccountStorage {
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    pub async fn init(&self) -> Result<()> {
        // Create directory structure
        fs::create_dir_all(&self.base_path).await?;
        fs::create_dir_all(self.base_path.join("gate")).await?;
        fs::create_dir_all(self.base_path.join("bybit")).await?;
        fs::create_dir_all(self.base_path.join("gmail")).await?;
        fs::create_dir_all(self.base_path.join("transactions")).await?;
        fs::create_dir_all(self.base_path.join("checks")).await?;
        Ok(())
    }

    // Gate account methods
    pub async fn save_gate_account(&self, login: &str, password: &str) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let filename = format!("{}.json", id);
        let file_path = self.base_path.join("gate").join(&filename);

        let account_data = GateAccountData {
            id: id.clone(),
            login: login.to_string(),
            password: Some(password.to_string()),
            status: Some("inactive".to_string()),
            cookies: None,
            last_auth: None,
            balance: 0.0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string_pretty(&account_data)?;
        fs::write(&file_path, json).await?;

        Ok(id)
    }

    pub async fn load_gate_account(&self, id: &str) -> Result<Option<(String, String, Option<serde_json::Value>)>> {
        let filename = format!("{}.json", id);
        let file_path = self.base_path.join("gate").join(&filename);

        if !file_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&file_path).await?;
        let account_data: GateAccountData = serde_json::from_str(&content)?;

        let password = account_data.password
            .ok_or_else(|| AppError::InternalError("Password not found".to_string()))?;

        Ok(Some((account_data.login, password, account_data.cookies)))
    }

    pub async fn update_gate_cookies(&self, id: &str, cookies: serde_json::Value) -> Result<()> {
        let filename = format!("{}.json", id);
        let file_path = self.base_path.join("gate").join(&filename);

        if !file_path.exists() {
            return Err(AppError::NotFound("Gate account not found".to_string()));
        }

        let content = fs::read_to_string(&file_path).await?;
        let mut account_data: GateAccountData = serde_json::from_str(&content)?;

        account_data.cookies = Some(cookies);
        account_data.last_auth = Some(Utc::now());
        account_data.updated_at = Utc::now();

        let json = serde_json::to_string_pretty(&account_data)?;
        fs::write(&file_path, json).await?;

        Ok(())
    }

    pub async fn update_gate_balance(&self, id: &str, balance: f64) -> Result<()> {
        let filename = format!("{}.json", id);
        let file_path = self.base_path.join("gate").join(&filename);

        if !file_path.exists() {
            return Err(AppError::NotFound("Gate account not found".to_string()));
        }

        let content = fs::read_to_string(&file_path).await?;
        let mut account_data: GateAccountData = serde_json::from_str(&content)?;

        account_data.balance = balance;
        account_data.updated_at = Utc::now();

        let json = serde_json::to_string_pretty(&account_data)?;
        fs::write(&file_path, json).await?;

        Ok(())
    }

    pub async fn list_gate_accounts(&self) -> Result<Vec<GateAccountData>> {
        let mut accounts = Vec::new();
        let gate_dir = self.base_path.join("gate");

        if !gate_dir.exists() {
            return Ok(accounts);
        }

        let mut entries = fs::read_dir(&gate_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(entry.path()).await?;
                if let Ok(account) = serde_json::from_str::<GateAccountData>(&content) {
                    accounts.push(account);
                }
            }
        }

        Ok(accounts)
    }

    // Bybit account methods
    pub async fn save_bybit_account(&self, api_key: &str, api_secret: &str) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let filename = format!("{}.json", id);
        let file_path = self.base_path.join("bybit").join(&filename);

        let account_data = BybitAccountData {
            id: id.clone(),
            api_key: api_key.to_string(),
            api_secret: Some(api_secret.to_string()),
            active_ads: 0,
            last_used: None,
            status: Some("active".to_string()),
            testnet: Some(false),
            last_error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string_pretty(&account_data)?;
        fs::write(&file_path, json).await?;

        Ok(id)
    }

    pub async fn load_bybit_account(&self, id: &str) -> Result<Option<(String, String)>> {
        let filename = format!("{}.json", id);
        let file_path = self.base_path.join("bybit").join(&filename);

        if !file_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&file_path).await?;
        let account_data: BybitAccountData = serde_json::from_str(&content)?;

        let api_secret = account_data.api_secret
            .ok_or_else(|| AppError::InternalError("API secret not found".to_string()))?;

        Ok(Some((account_data.api_key, api_secret)))
    }

    pub async fn update_bybit_active_ads(&self, id: &str, active_ads: i32) -> Result<()> {
        let filename = format!("{}.json", id);
        let file_path = self.base_path.join("bybit").join(&filename);

        if !file_path.exists() {
            return Err(AppError::NotFound("Bybit account not found".to_string()));
        }

        let content = fs::read_to_string(&file_path).await?;
        let mut account_data: BybitAccountData = serde_json::from_str(&content)?;

        account_data.active_ads = active_ads;
        account_data.last_used = Some(Utc::now());
        account_data.updated_at = Utc::now();

        let json = serde_json::to_string_pretty(&account_data)?;
        fs::write(&file_path, json).await?;

        Ok(())
    }

    pub async fn list_bybit_accounts(&self) -> Result<Vec<BybitAccountData>> {
        let mut accounts = Vec::new();
        let bybit_dir = self.base_path.join("bybit");

        if !bybit_dir.exists() {
            return Ok(accounts);
        }

        let mut entries = fs::read_dir(&bybit_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(entry.path()).await?;
                if let Ok(account) = serde_json::from_str::<BybitAccountData>(&content) {
                    accounts.push(account);
                }
            }
        }

        Ok(accounts)
    }

    // Transaction storage
    pub async fn save_transaction(&self, transaction_id: &str, data: &serde_json::Value) -> Result<()> {
        let filename = format!("{}.json", transaction_id);
        let file_path = self.base_path.join("transactions").join(&filename);

        let json = serde_json::to_string_pretty(data)?;
        fs::write(&file_path, json).await?;

        Ok(())
    }

    pub async fn load_transaction(&self, transaction_id: &str) -> Result<Option<serde_json::Value>> {
        let filename = format!("{}.json", transaction_id);
        let file_path = self.base_path.join("transactions").join(&filename);

        if !file_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&file_path).await?;
        let data: serde_json::Value = serde_json::from_str(&content)?;

        Ok(Some(data))
    }

    // Receipt storage
    pub async fn save_receipt(&self, transaction_id: &str, receipt_path: &str) -> Result<String> {
        let source_path = Path::new(receipt_path);
        if !source_path.exists() {
            return Err(AppError::NotFound("Receipt file not found".to_string()));
        }

        let extension = source_path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("pdf");
        
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.{}", transaction_id, timestamp, extension);
        let dest_path = self.base_path.join("checks").join(&filename);

        fs::copy(source_path, &dest_path).await?;

        Ok(filename)
    }

    pub async fn get_receipt_path(&self, filename: &str) -> PathBuf {
        self.base_path.join("checks").join(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_account_storage() -> Result<()> {
        let dir = tempdir()?;
        let storage = AccountStorage::new(dir.path());
        storage.init().await?;

        // Test Gate account
        let gate_id = storage.save_gate_account("test@example.com", "password123").await?;
        let loaded = storage.load_gate_account(&gate_id).await?.unwrap();
        assert_eq!(loaded.0, "test@example.com");
        assert_eq!(loaded.1, "password123");

        // Update cookies
        let cookies = serde_json::json!([{"name": "session", "value": "123"}]);
        storage.update_gate_cookies(&gate_id, cookies.clone()).await?;
        let loaded = storage.load_gate_account(&gate_id).await?.unwrap();
        assert_eq!(loaded.2, Some(cookies));

        // Test Bybit account
        let bybit_id = storage.save_bybit_account("api_key_123", "api_secret_456").await?;
        let loaded = storage.load_bybit_account(&bybit_id).await?.unwrap();
        assert_eq!(loaded.0, "api_key_123");
        assert_eq!(loaded.1, "api_secret_456");

        // Update active ads
        storage.update_bybit_active_ads(&bybit_id, 2).await?;

        // Test listing
        let gate_accounts = storage.list_gate_accounts().await?;
        assert_eq!(gate_accounts.len(), 1);

        let bybit_accounts = storage.list_bybit_accounts().await?;
        assert_eq!(bybit_accounts.len(), 1);
        assert_eq!(bybit_accounts[0].active_ads, 2);

        Ok(())
    }
}