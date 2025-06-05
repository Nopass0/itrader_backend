use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::core::config::Config;
use crate::core::rate_limiter::RateLimiter;
use crate::core::accounts::AccountManager;
use crate::core::account_storage::AccountStorage;
use crate::core::db_account_manager::DbAccountManager;
use crate::core::db_account_storage::DbAccountStorage;
use crate::db::{Repository, AccountRepository};
use crate::gate::client::GateClient;
use crate::gate::GateAccountManager;
use crate::bybit::BybitP2PClient;

pub struct AppState {
    pub config: Config,
    pub repository: Arc<Repository>,
    pub account_repository: Arc<AccountRepository>,
    pub rate_limiter: Arc<RateLimiter>,
    pub account_manager: Arc<AccountManager>,
    pub account_storage: Arc<AccountStorage>,
    pub db_account_manager: Arc<DbAccountManager>,
    pub db_account_storage: Arc<DbAccountStorage>,
    pub gate_manager: Arc<GateAccountManager>,
    pub gate_client: Arc<GateClient>,
    pub bybit_clients: RwLock<HashMap<i32, Arc<BybitP2PClient>>>,
    pub start_time: DateTime<Utc>,
    pub shutdown_signal: Arc<tokio::sync::Notify>,
    pub auto_approve_mode: Arc<tokio::sync::RwLock<bool>>,
    pub last_check_time: Arc<tokio::sync::RwLock<DateTime<Utc>>>,
}

impl AppState {
    pub async fn new(config: Config) -> crate::utils::error::Result<Arc<Self>> {
        // Create repository
        let repository = Arc::new(Repository::new(&config.database).await?);
        
        // Create account repository
        let account_repository = Arc::new(AccountRepository::new(repository.pool.clone()));
        
        // Create rate limiter
        let rate_limiter = Arc::new(RateLimiter::new(&config.rate_limits));
        
        // Create database-backed managers
        let db_account_manager = Arc::new(DbAccountManager::new(repository.pool.clone()));
        let db_account_storage = Arc::new(DbAccountStorage::new(repository.pool.clone()));
        
        // Create file-based managers (for backward compatibility)
        let account_manager = Arc::new(AccountManager::new("data/accounts.json").await?);
        let account_storage = Arc::new(AccountStorage::new("db"));
        account_storage.init().await?;
        
        // Create Gate manager (without encryption)
        let gate_manager = Arc::new(GateAccountManager::new(
            config.gate.clone(),
            rate_limiter.clone(),
            repository.clone(),
            account_storage.clone(),
        )?);
        
        // Create Gate client
        let gate_client = Arc::new(GateClient::new(
            config.gate.base_url.clone(),
            rate_limiter.clone(),
        )?);
        
        let state = Arc::new(Self {
            config,
            repository,
            account_repository,
            rate_limiter,
            account_manager,
            account_storage,
            db_account_manager,
            db_account_storage,
            gate_manager,
            gate_client,
            bybit_clients: RwLock::new(HashMap::new()),
            start_time: Utc::now(),
            shutdown_signal: Arc::new(tokio::sync::Notify::new()),
            auto_approve_mode: Arc::new(tokio::sync::RwLock::new(true)),
            last_check_time: Arc::new(tokio::sync::RwLock::new(Utc::now())),
        });
        
        Ok(state)
    }

    pub async fn add_bybit_client(&self, account_id: i32, client: Arc<BybitP2PClient>) {
        self.bybit_clients.write().insert(account_id, client);
    }

    pub fn get_bybit_client(&self, account_id: i32) -> Option<Arc<BybitP2PClient>> {
        self.bybit_clients.read().get(&account_id).cloned()
    }

    pub fn trigger_shutdown(&self) {
        self.shutdown_signal.notify_waiters();
    }

    pub async fn wait_for_shutdown(&self) {
        self.shutdown_signal.notified().await;
    }
}

#[derive(Debug, Clone)]
pub struct OrderContext {
    pub order_id: Uuid,
    pub gate_transaction_id: String,
    pub bybit_order_id: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub total_orders: u64,
    pub active_orders: u64,
    pub completed_orders: u64,
    pub failed_orders: u64,
    pub uptime_seconds: u64,
}