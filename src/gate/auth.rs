use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};
use rust_decimal::{Decimal, prelude::*};
use std::str::FromStr;

use crate::core::rate_limiter::RateLimiter;
use crate::core::config::GateConfig;
use crate::core::account_storage::AccountStorage;
use crate::db::Repository;
use crate::utils::error::{AppError, Result};
use super::client::GateClient;
use super::models::{Cookie, Payout};

pub struct GateAccountManager {
    pub client: Arc<GateClient>,
    config: GateConfig,
    repository: Arc<Repository>,
    account_storage: Arc<AccountStorage>,
}

impl GateAccountManager {
    pub fn new(
        config: GateConfig,
        rate_limiter: Arc<RateLimiter>,
        repository: Arc<Repository>,
        account_storage: Arc<AccountStorage>,
    ) -> Result<Self> {
        let client = Arc::new(GateClient::new(config.base_url.clone(), rate_limiter)?);
        
        Ok(Self {
            client,
            config,
            repository,
            account_storage,
        })
    }

    pub async fn authenticate_all_accounts(&self) -> Result<()> {
        info!("Starting authentication for all Gate.io accounts");
        
        let accounts = self.account_storage.list_gate_accounts().await?;
        
        for account in accounts {
            match self.authenticate_account(&account.id).await {
                Ok(_) => info!("Successfully authenticated account: {}", account.login),
                Err(e) => error!("Failed to authenticate account {}: {}", account.login, e),
            }
        }
        
        Ok(())
    }

    pub async fn authenticate_account(&self, account_id: &str) -> Result<()> {
        // Load account data
        let account_data = self.account_storage.load_gate_account(account_id).await?
            .ok_or_else(|| AppError::NotFound(format!("Account {} not found", account_id)))?;
        
        let (email, password, _cookies) = account_data;
        
        // Try to login
        match self.client.login(&email, &password).await {
            Ok(_response) => {
                info!("Successfully logged in to account: {}", email);
                
                // Note: cookies are automatically stored in the client's cookie jar
                // We could extract them if needed, but for now we'll handle that separately
                
                // Set initial balance
                self.set_account_balance(&email, self.config.target_balance).await?;
                self.account_storage.update_gate_balance(account_id, self.config.target_balance).await?;
                
                Ok(())
            }
            Err(e) => {
                warn!("Failed to login to account {}: {}", email, e);
                Err(e)
            }
        }
    }

    pub async fn load_cookies_from_file(&self, cookies: Vec<Cookie>) -> Result<()> {
        self.client.set_cookies(cookies).await?;
        info!("Loaded cookies from file");
        Ok(())
    }

    pub async fn set_account_balance(&self, _email: &str, amount: f64) -> Result<()> {
        let decimal_amount = Decimal::from_str(&amount.to_string())
            .map_err(|_| AppError::InvalidInput("Invalid amount".to_string()))?;
        
        self.client.set_balance("RUB", decimal_amount.to_f64().unwrap_or(0.0)).await?;
        info!("Set balance to {} RUB", amount);
        Ok(())
    }

    pub async fn start_session_refresh_task(self: Arc<Self>) {
        let mut interval = interval(Duration::from_secs(self.config.session_refresh_interval));
        
        loop {
            interval.tick().await;
            
            info!("Refreshing Gate.io sessions");
            if let Err(e) = self.refresh_all_sessions().await {
                error!("Failed to refresh sessions: {}", e);
            }
        }
    }

    pub async fn start_balance_monitor_task(self: Arc<Self>) {
        let mut interval = interval(Duration::from_secs(self.config.balance_check_interval));
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.check_and_update_balances().await {
                error!("Failed to check balances: {}", e);
            }
        }
    }

    async fn refresh_all_sessions(&self) -> Result<()> {
        let accounts = self.account_storage.list_gate_accounts().await?;
        
        for account in accounts {
            match self.refresh_session(&account.login).await {
                Ok(_) => info!("Refreshed session for: {}", account.login),
                Err(e) => {
                    error!("Failed to refresh session for {}: {}", account.login, e);
                    // Try to re-authenticate
                    if let Err(auth_err) = self.authenticate_account(&account.id).await {
                        error!("Re-authentication failed for {}: {}", account.login, auth_err);
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn refresh_session(&self, _email: &str) -> Result<()> {
        // Make a simple API call to keep session alive
        self.client.get_balance("RUB").await?;
        // Note: We could update last_auth in account storage if needed
        Ok(())
    }

    async fn check_and_update_balances(&self) -> Result<()> {
        let balance_response = self.client.get_balance("RUB").await?;
        let current_balance = balance_response.available;
        
        if current_balance < Decimal::from_str(&self.config.min_balance.to_string()).unwrap() {
            warn!("Balance {} is below minimum {}, topping up", current_balance, self.config.min_balance);
            self.client.set_balance("RUB", self.config.target_balance).await?;
        }
        
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Gate.io account manager");
        
        // Set all balances to 0
        let accounts = self.account_storage.list_gate_accounts().await?;
        
        for account in accounts {
            match self.client.set_balance("RUB", 0.0).await {
                Ok(_) => info!("Set balance to 0 for account: {}", account.login),
                Err(e) => error!("Failed to reset balance for {}: {}", account.login, e),
            }
            
            // Update balance in storage
            if let Err(e) = self.account_storage.update_gate_balance(&account.id, 0.0).await {
                error!("Failed to update account balance in storage: {}", e);
            }
        }
        
        Ok(())
    }
    
    pub async fn set_balance(&self, email: &str, amount: f64) -> Result<f64> {
        self.client.set_balance(email, amount).await
    }
    
    pub async fn get_pending_transactions(&self, email: &str) -> Result<Vec<Payout>> {
        self.client.get_pending_transactions(email).await
    }
    
    pub async fn approve_transaction(&self, _email: &str, transaction_id: &str) -> Result<()> {
        self.client.approve_transaction(transaction_id).await?;
        Ok(())
    }
}